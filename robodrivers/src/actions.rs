use rand::prelude::*;


#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    NORTH,
    SOUTH,
    EAST,
    WEST,
}

impl Direction {
    pub fn random_direction() -> Direction {
        match thread_rng().gen_range(0, 4) as u32 {
            0 => Direction::NORTH,
            1 => Direction::SOUTH,
            2 => Direction::EAST,
            _ => Direction::WEST,
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    STOP,
    MOVE(Direction),
    SUICIDE,
}

impl Action {
    pub fn random_action() -> Action {
        match thread_rng().gen_range(0, 1000) {
            0...1 => Action::SUICIDE,
            1...100 => Action::STOP,
            _ => Action::MOVE(Direction::random_direction()) ,
        }
    }
}

