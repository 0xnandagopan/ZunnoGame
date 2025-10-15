// backend/src/lib.rs

pub mod api;
pub mod blockchain;
pub mod game;
pub mod orchestrator;
pub mod proof_management;

// Re-export commonly used types
pub use blockchain::BlockchainAdapter;
pub use game::GameState;
pub use orchestrator::GameOrchestrator;
