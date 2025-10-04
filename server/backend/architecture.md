# Backend Architecture (Rust / Axum)

## Overview

The backend is a lightweight Axum service that orchestrates UNO game sessions. It exposes REST endpoints for shuffling, dealing, and playing cards, while optionally integrating with an Ethereum-compatible smart contract to obtain randomness. The service is designed to host many concurrent games on a single instance by namespacing every route with a `game_id`.

## Component map

```
┌──────────────────────────────┐
│ HTTP Client (curl / UI)      │
└──────────────┬───────────────┘
							 │
			Axum Router (per game_id)
							 │
			┌────────┴────────┐
			│                 │
 API Handlers     SharedGames (Arc<RwLock<HashMap>>)
			│                 │
			│          ┌──────┴──────┐
			│          │             │
			▼          ▼             ▼
	game::API   game::GameState  blockchain::Adapter
										 │                │
										 │     (optional) │
										 ▼                ▼
							zunnogame-lib     Base / RPC node
```

## Request lifecycle

1. **Routing (`lib.rs`)** – Requests hit `/game/{game_id}/…` endpoints, ensuring isolation between rooms.
2. **Handler extraction (`api::handlers`)** – Path parameters, query strings, and JSON payloads are parsed. The handler looks up or mutates the appropriate `GameState` inside `SharedGames` (`Arc<RwLock<HashMap<String, GameState>>>`).
3. **Game logic (`game` module)** – Business rules validate player actions, modify card piles, and serialize responses for both Rust and JavaScript clients.
4. **Randomness provider (`blockchain` module)** – When shuffling, the handler calls `blockchain::get_random_seed()`. If RPC credentials exist, it requests randomness from the Uno smart contract; otherwise it falls back to a cryptographically secure local RNG.
5. **Response** – Data is returned as JSON, with cards represented in both raw indices and JS-friendly codes when appropriate.

## Key modules

### `src/lib.rs`

- Boots Axum using `#[tokio::main]`.
- Initializes `SharedGames` (thread-safe map of `GameState`s keyed by `game_id`).
- Registers REST routes for core gameplay, state inspection, and health checks.
- Applies permissive CORS via `tower_http::cors::CorsLayer` to simplify local testing.

### `src/api/mod.rs`

- Contains all HTTP handlers and DTOs.
- Each handler extracts `State<SharedGames>` plus path parameters (`game_id`, `player_id`).
- Centralizes error responses with structured JSON payloads.
- Implements helper utilities (e.g., `game_not_found`) to keep error handling consistent.

### `src/game/mod.rs`

- Defines the `GameState` struct and helper methods such as `player_count()` and `total_cards()`.
- Implements asynchronous `shuffle_and_deal` that sources randomness via the blockchain adapter, then delegates to `zunnogame_lib` for Fisher–Yates shuffling and distribution.
- Provides operations for drawing, playing cards, and serializing hands in both raw/indexed and JavaScript string formats.

### `src/blockchain/mod.rs`

- Wraps Alloy provider bindings for the Uno smart contract.
- Attempts to read `RPC_URL` and `CONTRACT_ADDRESS` (from environment or `.env`).
- If credentials are missing, malformed, or the contract call fails, logs a warning and returns a locally generated seed (`rand::thread_rng()`), guaranteeing the backend remains usable in development.

## State management

- `SharedGames` allows the server to host many rooms simultaneously without cross-talk.
- The `RwLock` gives fast reads for state inspection endpoints and exclusive writes for mutations (shuffle, draw, play).
- Rooms are implicitly created the first time they are accessed; a `reset` call simply replaces the entry with a fresh `GameState`.
- Consider periodic cleanup (evicting inactive games) if long-lived cats accumulate.

## Configuration & environment

| Variable           | Description                            | Default behaviour                      |
| ------------------ | -------------------------------------- | -------------------------------------- |
| `RPC_URL`          | HTTP RPC endpoint for the Uno contract | Optional – fallback RNG used if absent |
| `CONTRACT_ADDRESS` | Checksummed contract address           | Optional – fallback RNG used if absent |

Create a `.env` file inside `server/backend/` or export the variables when you need on-chain randomness.

## Testing & tooling

- Start the backend: `cargo run --manifest-path server/backend/Cargo.toml`.
- Example workflow (from repo root):
  ```bash
  BASE=http://127.0.0.1:3001
  curl -sS "$BASE/health" | jq
  curl -sS -X POST "$BASE/game/room-a/shuffle-and-deal" \
  	-H "Content-Type: application/json" \
  	-d '{"players":4,"cards_per_player":7}' | jq
  curl -sS "$BASE/game/room-a/player/0/hand" | jq
  ```
- That same pattern can be scripted (Node, Rust integration tests, Postman collections) to automate regression coverage.

## Future enhancements

- **Game lifecycle management** – Add TTL-based eviction or `/games` admin endpoints to inspect/remove rooms.
- **Persistence** – Introduce a database or Redis cache if you need durable game history or horizontal scaling.
- **Observability** – Integrate structured logging sinks (OpenTelemetry) and metrics for production deployments.
- **Proof integration** – Once ZK proofs land, extend the blockchain adapter to submit/verify proofs alongside shuffle seeds.
