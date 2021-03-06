extern crate serde_yaml;
extern crate serde_json;

use std::collections::HashMap;
use robodrivers::Direction;


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum Resource {
    GAS(u32)
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Producer {
    pub resource: Resource,
    pub cooldown: u32,
    pub respawn_in: u32,
    pub on_cooldown: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Item {
    BASE,
    RESOURCE(Resource),
    PRODUCER(Producer),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Block {
    WALL,
    OPEN,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cell {
    pub block: Block,
    pub items: Vec<Item>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Map {
    pub cells: Vec<Vec<Cell>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    MOVING(Direction),
    STOPPED
}

impl Default for State {
    fn default() -> State { State::STOPPED }
}


#[derive(Default, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Car {
    pub x: i32,
    pub y: i32,
    pub next_x: i32,
    pub next_y: i32,
    pub collided: bool,
    pub killed: bool,
    pub health: i32,
    pub max_health: i32,
    pub resources: u32,
    pub state: State,
    pub team_id: u32,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Team {
    pub name: String,
    pub color: String,
    pub score: u32,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct GameState {
    pub id: u32,
    pub map: Map,
    pub teams: HashMap<u32, Team>,
    pub cars: HashMap<u32, Car>,
    pub tick: u32,
}

impl GameState {
    pub fn from_serialized(serialized: &str) -> GameState {
        serde_yaml::from_str::<GameState>(serialized).expect("Unable to deserialize game state")
    }

    pub fn to_serialized(self: &Self) -> String {
        serde_yaml::to_string(self).expect("Unable to serialize game state into YAML")
    }

    pub fn to_json(self: &Self) -> String {
        serde_json::to_string(self).expect("Unable to serialize game state into JSON")
    }
}

pub fn recreate_game_state() -> () {
    let mut game_state = GameState::default();
    {
        let nb_teams = 8;
        let max_health = 3;
        let team_colors = vec![ "#aa0000", "#00aa00", "#0000aa", "#aaaa00", "#00aaaa", "#aaaaaa", "#aa00aa", "#44aaff" ];
        let team_names  = vec![ "team 1", "team 2", "team 3", "team 4", "team 5", "team 6", "team 7", "team 8" ];
        game_state.id = 0;
        game_state.tick = 0;
        let map = &mut game_state.map;
        let teams = &mut game_state.teams;
        let cars = &mut game_state.cars;
        for i in 0..nb_teams {
            let team_id = i + 1;
            let car = Car {
                x: 0,
                y: 0,
                next_x: 0,
                next_y: 0,
                collided: false,
                killed: false,
                health: max_health,
                max_health: max_health,
                resources: 0,
                state: State::STOPPED,
                team_id: team_id,
            };
            teams.insert(team_id, Team {
                name: team_names[i as usize].to_string(),
                color: team_colors[i as usize].to_string(),
                score: 0,
            });
            cars.insert(team_id, car);
        }
        let level = vec![
            "WWWWWWWWWWWWWWWWWWWWWWWW",
            "W  9                   W",
            "W WWWWWWWWW WWWWWW W   W",
            "W W                W W W",
            "W W      5       W W W W",
            "W W             3W W W W",
            "W W  WWWWWWWWWWWWW W W W",
            "W W WWW          1 W   W",
            "W W                W   W",
            "W WWWWWW      W    W393W",
            "W             W    WWWWW",
            "W      W      W    W    ",
            "W      W  1        W    ",
            "W      WWWWWW      W    ",
            "W                  W    ",
            "W       WWWW       W    ",
            "W                  W    ",
            "WWWW            WWWW    ",
            "   WBBBBBBBBBBBBW       ",
            "   WWWWWWWWWWWWWW       ",
        ];
        for level_row in level.iter() {
            let mut row: Vec<Cell> = Vec::new();
            for c in level_row.chars() {
                let block = match c {
                    'W' => Block::WALL,
                    _ => Block::OPEN
                };
                let mut items: Vec<Item> = Vec::new();
                match c {
                    'B' => items.push(Item::BASE),
                     val @ '1' ... '9' => {
                        let value = val.to_digit(10).unwrap();
                        let cooldown = 10 + value*2;
                        let producer = Producer { resource: Resource::GAS(value), cooldown: cooldown, respawn_in: 1, on_cooldown: true };
                        items.push(Item::PRODUCER(producer));
                    },
                    _ => ()
                };
                let cell = Cell { block: block, items: items };
                row.push(cell);
            }
            map.cells.push(row);
        }
    }
    println!("{}", game_state.to_serialized());
}
