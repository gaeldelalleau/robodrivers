use std::thread;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::time;

use game_state::GameState;

use config::WORKING_DIRECTORY;
use config::SERIALIZED_FILES_EXTENSION;

use logging::LOGGER;


const BASE_GAME_STATE_FILENAME: &str = "game_state_base";
const CURRENT_GAME_STATE_FILENAME: &str = "game_state_{}";


pub struct GameEngine {
    game_state: GameState,
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

        GameEngine { game_state: game_state, current_game_state_file: file }
    }

    fn save_game_state(self: &mut Self) -> () {
        // TODO: lock gamestate mutex for the duration of this function
        trace!(logger!(), "Saving game state to file");
        let serialized = self.game_state.to_serialized();
        self.current_game_state_file.seek(SeekFrom::Start(0)).expect("Unable to seek at start of game state file");
        self.current_game_state_file.write_all(serialized.as_bytes()).expect("Unable to write to game state file");
        let cursor_pos = self.current_game_state_file.seek(SeekFrom::Current(0)).expect("Unable to get cursor position of game state file");
        self.current_game_state_file.set_len(cursor_pos).expect("Unable to truncate game state file at cursor position");
        self.current_game_state_file.flush().expect("Unable to flush game state file");
        trace!(logger!(), "Game state saved to file");
    }

        fn step(self: &mut Self) -> () {
        // TODO: lock gamestate mutex for the duration of this function
        // TODO: simulate game: update game state from team actions, and delete all
        //       team actions
        self.game_state.tick += 1;
    }

    pub fn start(self: &mut Self) -> () {
        trace!(logger!(), "Starting game loop");
        loop {
            thread::sleep(time::Duration::from_millis(200));
            self.step();
            if self.game_state.tick % 10 == 0 {
                self.save_game_state();
            }
        }
    }
}
