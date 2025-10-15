// backend/src/lib.rs

pub mod api;
pub mod blockchain;
pub mod game;
pub mod orchestrator;

// Re-export commonly used types
pub use orchestrator::GameOrchestrator;
pub use game::GameState;
pub use blockchain::BlockchainAdapter;
