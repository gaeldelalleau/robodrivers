use std::thread;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::time;
use std::sync::Mutex;
use std::collections::HashMap;

use game_state::GameState;
use game_state::Map;
use game_state::Cell;
use game_state::Item;
use game_state::Resource;
use game_state::Producer;
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

pub struct GameEngine {
    game_id: u32,
    current_game_state_file: File,
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

    pub fn new() -> GameEngine {
        let game_id = 0;
        let mut file = GameEngine::get_game_state_file(game_id);

        let mut serialized = String::new();
        file.read_to_string(&mut serialized).expect("Unable to read game state file");
        let mut game_state = GameState::from_serialized(&serialized);
        game_state.id = game_id;

        // TODO: parse map to get list of cells holding: base items, producers items

        game_state_map.lock().expect("Unable to acquire game_state_map lock").insert(game_id, game_state);

        GameEngine { game_id: game_id, current_game_state_file: file }
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

    fn spawn(self: &Self, map: &mut Map, car: &mut Car) {
        // TODO XXX
        self.restore_health(car);
        car.state = State::STOPPED;
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

    fn kill(self: &Self, map: &mut Map, car: &mut Car) {
        self.drop_resources(map, car);
        self.spawn(map, car);
    }

    fn move_car(self: &Self, map: &mut Map, car: &mut Car, direction: &Direction) {
        let mut x = car.x;
        let mut y = car.y;
        match direction {
            Direction::NORTH => y -= 1,
            Direction::SOUTH => y += 1,
            Direction::EAST => x += 1,
            Direction::WEST => x -= 1,
        }
        match self.cell_at(map, x, y).block {
            Block::WALL => (),
            Block::OPEN => {
                car.x = x;
                car.y = y;
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

    fn apply_action(self: &Self, game_state: &mut GameState, team_id: u32, action: &Action) {
        let car = &mut game_state.cars.get_mut(&team_id).expect(&format!("Team {} has no car!", team_id));
        let team = &mut game_state.teams.get_mut(&team_id).expect(&format!("Team {} does not exists!", team_id));
        let map = &mut game_state.map;

        match action {
            Action::STOP => car.state = State::STOPPED,
            Action::MOVE(direction) => self.move_car(map, car, direction),
            Action::SUICIDE => self.kill(map, car),
        }

        self.collect(map, car);

        if self.items_at(map, car.x, car.y).contains(&Item::BASE) {
            self.at_base(car, team);
        }
    }

    fn step(self: &mut Self) -> () {
        let mut game_state_guard = game_state_guard!();
        let game_state = game_state!(game_state_guard, self.game_id);
        let mut actions = actions!();

        // TODO: let producers produce!
        // aka: for each producer on cooldown, reduce the respawn_in by 1, and drop resources on the
        // cell if ok

        let team_ids = game_state.teams.keys().map(|k| *k).collect::<Vec<u32>>();
        for team_id in team_ids {
            let action = match actions.get(&team_id) {
                Some(a) => a,
                None => &Action::STOP,
            };
            self.apply_action(game_state, team_id, action);
        }
        actions.clear();

        game_state.tick += 1;
    }

    pub fn start(self: &mut Self) -> () {
        trace!(logger!(), "Starting game loop");
        loop {
            thread::sleep(time::Duration::from_millis(200));
            self.step();
            if game_state!(game_state_guard!(), &self.game_id).tick % 10 == 0 {
                self.save_game_state();
            }
        }
    }
}
