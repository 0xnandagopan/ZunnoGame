// backend/src/api/mod.rs

pub mod game_routes;

pub use game_routes::{get_game_state, get_game_status, start_game};
