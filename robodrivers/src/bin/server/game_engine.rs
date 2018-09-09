use std::thread;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::time;
use std::sync::Mutex;
use std::collections::HashMap;
use std::collections::HashSet;

use rand::prelude::*;
use ws;

use game_state::GameState;
use game_state::Map;
use game_state::Cell;
use game_state::Item;
use game_state::Resource;
use game_state::Block;
use game_state::Direction;
use game_state::Team;
use game_state::Car;
use game_state::State;

use config::WORKING_DIRECTORY;
use config::SERIALIZED_FILES_EXTENSION;

use logging::LOGGER;


const BASE_GAME_STATE_FILENAME: &str = "game_state_base";
const CURRENT_GAME_STATE_FILENAME: &str = "game_state_{}";


lazy_static! {
    pub static ref game_state_map: Mutex<HashMap<u32, GameState>> = Mutex::new(HashMap::new());
}

macro_rules! game_state_guard {
    () => { game_state_map.lock().expect("Unable to acquire lock for game_state_map") };
}

macro_rules! game_state {
    ($guard:expr) => { $guard.get_mut(&0).expect("Unable to find a game state with id 0") };
    ($guard:expr, $game_id:expr) => { $guard.get_mut(&$game_id).expect("Unable to find a game state with this id") };
}

lazy_static! {
    pub static ref actions_map: Mutex<HashMap<u32, Action>> = Mutex::new(HashMap::new());
}

macro_rules! actions {
    () => { actions_map.lock().expect("Unable to acquire lock for actions_map") }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    STOP,
    MOVE(Direction),
    SUICIDE,
}

impl Action {
    fn random_action() -> Action {
        match thread_rng().gen_range(0, 100) {
            0...3 => Action::SUICIDE,
            3...10 => Action::STOP,
            _ => Action::MOVE(Direction::random_direction()) ,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct Coord {
    x: i32,
    y: i32,
}

pub struct GameEngine {
    game_id: u32,
    current_game_state_file: File,
    bases: Vec<Coord>,
    producers: Vec<Coord>,
    simulate: bool,
}

impl GameEngine {

    fn get_game_state_file(game_id: u32) -> File {
        let mut base_game_state_file = WORKING_DIRECTORY.clone();
        base_game_state_file.push(BASE_GAME_STATE_FILENAME);
        base_game_state_file.set_extension(SERIALIZED_FILES_EXTENSION);

        let mut current_game_state_file = WORKING_DIRECTORY.clone();
        current_game_state_file.push(CURRENT_GAME_STATE_FILENAME.replace("{}", &game_id.to_string()));
        current_game_state_file.set_extension(SERIALIZED_FILES_EXTENSION);

        debug!(logger!(), "Reading config files from {}", WORKING_DIRECTORY.clone().display());
        debug!(logger!(), "Current game state file is: {}", current_game_state_file.clone().display());

        match OpenOptions::new().read(true).write(true).open(&current_game_state_file) {
            Ok(file) => file,
            Err(_) => {
                info!(logger!(), "No game state file found for game {}, will create a fresh one", game_id);
                fs::copy(&base_game_state_file, &current_game_state_file).expect("Unable to copy base game state file to current game state file");
                match OpenOptions::new().read(true).write(true).open(&current_game_state_file) {
                    Ok(file) => file,
                    Err(_) => panic!("Couldn't copy base game state file {} to file {}", base_game_state_file.display(), current_game_state_file.display()),
                }
            }
        }
    }

    pub fn new(simulate: bool) -> GameEngine {
        let game_id = 0;
        let mut file = GameEngine::get_game_state_file(game_id);

        let mut serialized = String::new();
        file.read_to_string(&mut serialized).expect("Unable to read game state file");
        let mut game_state = GameState::from_serialized(&serialized);
        game_state.id = game_id;

        let mut bases: Vec<Coord> = Vec::new();
        let mut producers: Vec<Coord> = Vec::new();
        for (y, row) in game_state.map.cells.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                for item in cell.items.iter() {
                    match item {
                        Item::BASE => bases.push(Coord { x: x as i32, y: y as i32 }),
                        Item::PRODUCER(_producer) => producers.push(Coord { x: x as i32, y: y as i32 }),
                        _ => (),
                    }
                }
            }
        }

        game_state_map.lock().expect("Unable to acquire game_state_map lock").insert(game_id, game_state);

        GameEngine { game_id: game_id, current_game_state_file: file, bases: bases, producers: producers, simulate: simulate }
    }

    fn save_game_state(self: &mut Self) -> () {
        trace!(logger!(), "Saving game state to file");
        let serialized = game_state!(game_state_guard!(), &self.game_id).to_serialized();
        self.current_game_state_file.seek(SeekFrom::Start(0)).expect("Unable to seek at start of game state file");
        self.current_game_state_file.write_all(serialized.as_bytes()).expect("Unable to write to game state file");
        let cursor_pos = self.current_game_state_file.seek(SeekFrom::Current(0)).expect("Unable to get cursor position of game state file");
        self.current_game_state_file.set_len(cursor_pos).expect("Unable to truncate game state file at cursor position");
        self.current_game_state_file.flush().expect("Unable to flush game state file");
        trace!(logger!(), "Game state saved to file");
    }

    fn cell_at<'a>(self: &Self, map: &'a Map, x: i32, y: i32) -> &'a Cell {
        &map.cells[y as usize][x as usize]
    }

    fn mut_cell_at<'a>(self: &Self, map: &'a mut Map, x: i32, y: i32) -> &'a mut Cell {
        &mut map.cells[y as usize][x as usize]
    }

    fn mut_items_at<'a>(self: &Self, map: &'a mut Map, x: i32, y: i32) -> &'a mut Vec<Item> {
        &mut self.mut_cell_at(map, x, y).items
    }

    fn items_at<'a>(self: &Self, map: &'a Map, x: i32, y: i32) -> &'a Vec<Item> {
        &self.cell_at(map, x, y).items
    }

    fn spawn(self: &Self, cars: &mut HashMap<u32, Car>, team_id: u32) {
        let bases = self.bases.iter().filter(|coord| {
            for car in cars.values() {
                if (car.x == coord.x) && (car.y == coord.y) { return false; }
            }
            true
        }).collect::<Vec<&Coord>>();

        let random_base = bases[thread_rng().gen_range(0, bases.len())];
        let car = self.mut_get_car(cars, team_id);
        car.x = random_base.x;
        car.y = random_base.y;
        car.state = State::STOPPED;
        self.restore_health(car);
    }

    fn drop_resources(self: &Self, map: &mut Map, car: &mut Car) {
        let items = self.mut_items_at(map, car.x, car.y);
        items.push(Item::RESOURCE(Resource::GAS(car.resources)));
        car.resources = 0;
    }

    fn at_base(self: &Self, car: &mut Car, team: &mut Team) {
        team.score += car.resources;
        car.resources = 0;
        self.restore_health(car);
    }

    fn restore_health(self: &Self, car: &mut Car) {
        car.health = car.max_health;
    }

    fn kill(self: &Self, map: &mut Map, cars: &mut HashMap<u32, Car>, team_id: u32) {
        self.drop_resources(map, self.mut_get_car(cars, team_id));
    }

    fn tentative_move_car(self: &Self, map: &mut Map, cars: &mut HashMap<u32, Car>, team_id: u32, direction: &Direction) {
        let car = cars.get_mut(&team_id).unwrap();
        let mut x = car.x;
        let mut y = car.y;
        match direction {
            Direction::NORTH => y -= 1,
            Direction::SOUTH => y += 1,
            Direction::EAST => x += 1,
            Direction::WEST => x -= 1,
        }
        match self.cell_at(map, x, y).block {
            Block::WALL => car.state = State::STOPPED,
            Block::OPEN => {
                car.next_x = x;
                car.next_y = y;
                car.state = State::MOVING(*direction);
            },
        }
    }

    fn collect(self: &Self, map: &mut Map, car: &mut Car) {
        let items = self.mut_items_at(map, car.x, car.y);
        items.retain(|item| {
            match item {
                Item::RESOURCE(resource) => {
                    match resource {
                        Resource::GAS(value) => car.resources += value,
                    }
                    false
                },
                _ => true,
            }
        });
        for item in items {
            match item {
                Item::PRODUCER(producer) => {
                    if !producer.on_cooldown {
                        producer.on_cooldown = true;
                        producer.respawn_in = producer.cooldown;
                    }
                },
                _ => (),
            }
        }
    }

    fn get_car<'a>(self: &Self, cars: &'a HashMap<u32, Car>, team_id: u32) -> &'a Car {
        cars.get(&team_id).expect(&format!("Team {} has no car!", team_id))
    }

    fn mut_get_car<'a>(self: &Self, cars: &'a mut HashMap<u32, Car>, team_id: u32) -> &'a mut Car {
        cars.get_mut(&team_id).expect(&format!("Team {} has no car!", team_id))
    }

    fn stop_car(self: &Self, cars: &mut HashMap<u32, Car>, team_id: u32) {
        let car = self.mut_get_car(cars, team_id);
        car.state = State::STOPPED;
    }

    fn mark_collision(self: &Self, cars: &mut HashMap<u32, Car>, team_id: u32) -> bool {
        let car = self.mut_get_car(cars, team_id);
        car.collided = true;
        if car.health > 0 {
            car.health -= 1;
            if car.health == 0 {
                return true; // killed
            }
        }
        false // still alive
    }

    fn opposite_directions(self: &Self, direction: &Direction, other_direction: &Direction) -> bool {
        match direction {
            Direction::NORTH => match other_direction {
                Direction::SOUTH => true,
                _ => false,
            },
            Direction::SOUTH => match other_direction {
                Direction::NORTH => true,
                _ => false,
            },
            Direction::EAST => match other_direction {
                Direction::WEST => true,
                _ => false,
            },
            Direction::WEST => match other_direction {
                Direction::EAST => true,
                _ => false,
            },
        }
    }

    fn solver(self: &Self, cars: &mut HashMap<u32, Car>, killed_teams: &mut HashSet<u32>) {

        for car in &mut cars.values_mut() {
            car.collided = false;
        }

        let mut free_set: HashSet<u32> = HashSet::new();
        for &id in cars.keys().filter(|id| !killed_teams.contains(id)) {
            free_set.insert(id);
        }

        let all_cars = cars.clone();

        while free_set.len() > 0 {
            let mut solved: Vec<u32> = Vec::new();
            for car_id in &free_set {
                let car = self.get_car(cars, *car_id).clone();
                let direction: Direction;
                let mut free_to_go: bool = true;

                match car.state {
                    State::MOVING(d) => direction = d,
                    _ => {
                        solved.push(*car_id);
                        continue;
                    },
                }
                let next_coord = Coord { x: car.next_x, y: car.next_y };

                for (other_id, other_car) in all_cars.iter().filter(|t| t.0 != car_id) {
                    let mut collisions_coords: Vec<Coord> = Vec::new();
                    let mut potential_collisions_coords: Vec<Coord> = Vec::new();
                    let other_coord = Coord { x: other_car.x, y: other_car.y };
                    let other_direction: Direction;
                    let mut collision: bool = false;

                    match other_car.state {
                        State::MOVING(d) => {
                            other_direction = d;
                            collisions_coords.push(Coord { x: other_car.next_x, y: other_car.next_y });
                            if self.get_car(cars, *other_id).collided {
                                collisions_coords.push(other_coord);
                            } else if free_set.contains(other_id) {  // aka "if this car is still unresolved"
                                potential_collisions_coords.push(other_coord);
                            }
                            if self.opposite_directions(&direction, &other_direction) && (next_coord == other_coord) {
                                collision = true;
                            }
                        },
                        State::STOPPED => {
                            collisions_coords.push(other_coord);
                        },
                    }

                    if potential_collisions_coords.contains(&next_coord) {
                        free_to_go = false;
                    }
                    if collisions_coords.contains(&next_coord) {
                        collision = true;
                    }

                    if collision {
                        free_to_go = false;
                        for &id in vec!(car_id, other_id) {
                            if !self.get_car(cars, id).collided {
                                if self.mark_collision(cars, id) {
                                    killed_teams.insert(id);
                                }
                                solved.push(id);
                            }
                        }
                    }
                }
                if free_to_go {
                    let car = self.mut_get_car(cars, *car_id);
                    car.x = car.next_x;
                    car.y = car.next_y;
                    solved.push(*car_id);
                }
            }
            for id in &solved {
                free_set.remove(id);
            }
        }
    }

    fn apply_action(self: &Self, game_state: &mut GameState, team_id: u32, action: &Action) -> bool {
        let cars = &mut game_state.cars;
        let map = &mut game_state.map;

        match action {
            Action::STOP => self.stop_car(cars, team_id),
            Action::MOVE(direction) => self.tentative_move_car(map, cars, team_id, direction),
            Action::SUICIDE => {
                self.stop_car(cars, team_id);
                return true; // killed
            },
        }
        false // not killed
    }

    fn post_events(self: &Self, game_state: &mut GameState, team_id: u32) {
        let cars = &mut game_state.cars;
        let team = &mut game_state.teams.get_mut(&team_id).expect(&format!("Team {} does not exists!", team_id));
        let map = &mut game_state.map;

        let car = self.mut_get_car(cars, team_id);
        self.collect(map, car);
        if self.items_at(map, car.x, car.y).contains(&Item::BASE) {
            self.at_base(car, team);
        };
    }

    fn act(self: &Self, game_state: &mut GameState) {
        let mut actions = actions!();
        let team_ids = game_state.teams.keys().map(|k| *k).collect::<Vec<u32>>();
        let mut killed_teams: HashSet<u32> = HashSet::new();

        for team_id in &team_ids {
            let action = match actions.get(&team_id) {
                Some(a) => a,
                None => &Action::STOP,
            };
            let killed = self.apply_action(game_state, *team_id, action);
            if killed {
                killed_teams.insert(*team_id);
            }
        }

        self.solver(&mut game_state.cars, &mut killed_teams);

        for team_id in team_ids.iter().filter(|id| !killed_teams.contains(id)) {
            self.post_events(game_state, *team_id);
        }

        for team_id in killed_teams {
            self.kill(&mut game_state.map, &mut game_state.cars, team_id);
            self.spawn(&mut game_state.cars, team_id);
        }

        actions.clear();
    }

    fn produce(self: &mut Self, map: &mut Map) -> () {
        for coord in &self.producers {
            let mut resource: Option<Resource> = None;
            let mut items = self.mut_items_at(map, coord.x, coord.y);
            for item in items.iter_mut() {
                match item {
                    Item::PRODUCER(producer) => {
                        if producer.on_cooldown {
                            if producer.respawn_in == 0 {
                                producer.on_cooldown = false;
                                resource = Some(producer.resource);
                            } else {
                                producer.respawn_in -= 1;
                            }
                        }
                    },
                    _ => (),
                }
            }
            match resource {
                Some(r) => items.push(Item::RESOURCE(r)),
                None => (),
            }
        }
    }

    fn simulation(self: &Self, game_state: &GameState) {
        let mut actions = actions!();
        for team_id in game_state.teams.keys() {
            actions.insert(*team_id, Action::random_action());
        }
    }

    fn step(self: &mut Self) -> () {
        let mut game_state_guard = game_state_guard!();
        let game_state = game_state!(game_state_guard, self.game_id);

        self.produce(&mut game_state.map);
        if self.simulate {
            self.simulation(game_state);
        }
        self.act(game_state);

        game_state.tick += 1;
    }

    pub fn start(self: &mut Self, ws_broadcaster: ws::Sender) -> () {
        trace!(logger!(), "Initializing map objects");
        {
            let mut game_state_guard = game_state_guard!();
            let game_state = game_state!(game_state_guard, &self.game_id);

            for &team_id in game_state.teams.keys() {
                self.spawn(&mut game_state.cars, team_id);
            }
        }

        trace!(logger!(), "Starting game loop");
        loop {
            thread::sleep(time::Duration::from_millis(200));
            self.step();
            ws_broadcaster.send(game_state!(game_state_guard!(), &self.game_id).to_json()).expect("Broadcast to WebSocket failed");
            if game_state!(game_state_guard!(), &self.game_id).tick % 10 == 0 {
                self.save_game_state();
            }
        }
    }
}
