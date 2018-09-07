extern crate serde_yaml;


#[derive(Serialize, Deserialize, Debug)]
pub enum Resource {
    GAS(u32)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Producer {
    resource: Resource,
    cooldown: u32,
}

#[derive(Serialize, Deserialize, Debug)]
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
    block: Block,
    items: Vec<Item>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Map {
    cells: Vec<Vec<Cell>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Direction {
    NORTH,
    SOUTH,
    EAST,
    WEST,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum State {
    MOVING(Direction),
    STOPPED
}

impl Default for State {
    fn default() -> State { State::STOPPED }
}


#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Car {
    pub x: i32,
    pub y: i32,
    pub health: i32,
    pub resources: i32,
    pub state: State,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Team {
    pub id: u32,
    pub name: String,
    pub color: String,
    pub score: i32,
    pub token: String,
    pub car: Car,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct GameState {
    pub id: u32,
    pub map: Map,
    pub teams: Vec<Team>,
    pub tick: u32,
}

impl GameState {
    pub fn new() -> GameState {
        GameState::default()
    }

    pub fn from_serialized(serialized: &str) -> GameState {
        serde_yaml::from_str::<GameState>(serialized).expect("Unable to deserialize game state")
    }

    pub fn to_serialized(self: &Self) -> String {
        serde_yaml::to_string(self).expect("Unable to serialize game state")
    }
}

pub fn recreate_game_state() -> () {
    let mut game_state = GameState::default();
    {
        let nb_teams = 8;
        let team_colors = vec![ "#aa0000", "#00aa00", "#0000aa", "#aaaa00", "#00aaaa", "#aaaaaa", "#aa00aa", "#44aaff" ];
        let team_names  = vec![ "team 1", "team 2", "team 3", "team 4", "team 5", "team 6", "team 7", "team 8" ];
        let team_tokens  = vec![ "XXX1", "XXX2", "XXX3", "XXX4", "XXX5", "XXX6", "XXX7", "XXX8" ];
        game_state.id = 0;
        game_state.tick = 0;
        let map = &mut game_state.map;
        let teams = &mut game_state.teams;
        for i in 0..nb_teams {
            let car = Car {
                x: 0,
                y: 0,
                health: 3,
                resources: 0,
                state: State::STOPPED,
            };
            teams.push(Team {
                id: i+1,
                name: team_names[i as usize].to_string(),
                color: team_colors[i as usize].to_string(),
                score: 0,
                token: team_tokens[i as usize].to_string(),
                car: car,
            });
        }
        let level = vec![
            "WWWWWWWWWWWWWWWWWWWW",
            "W  9               W",
            "W WWWWWWWWW WWWWWW W",
            "W W              W W",
            "W W      5       W W",
            "W W             3W W",
            "W W  WWWWWWWWWWWWW W",
            "W W              1 W",
            "W W                W",
            "W WWWWWW      W    W",
            "W             W    W",
            "W      W      W    W",
            "W      W  1        W",
            "W      W           W",
            "W      WWWWWW      W",
            "W                  W",
            "W                  W",
            "W       WWWW       W",
            "W                  W",
            "W                  W",
            "WBBBBBBBBBBBBBBBBBBW",
            "WWWWWWWWWWWWWWWWWWWW",
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
                        let producer = Producer { resource: Resource::GAS(value), cooldown: cooldown };
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
