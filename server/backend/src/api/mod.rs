pub mod handlers {
    use axum::{
        extract::{Path, Query, State},
        http::StatusCode,
        response::Json,
    };
    use serde::{Deserialize, Serialize};

    use crate::{
        game::{self, GameState, GameStateJS, PlayerId},
        SharedGames,
    };

    // ============================================================================
    // REQUEST/RESPONSE TYPES
    // ============================================================================

    #[derive(Deserialize)]
    pub struct ShuffleAndDealRequest {
        pub players: u8,
        pub cards_per_player: u8,
    }

    #[derive(Serialize)]
    pub struct ShuffleAndDealResponse {
        pub success: bool,
        pub message: String,
        pub total_players: u8,
        pub cards_per_player: u8,
        pub cards_remaining_in_deck: usize,
        pub seed_used: u64, // for debugging/replay
    }

    #[derive(Serialize)]
    pub struct InitialHandsResponse {
        pub player_id: PlayerId,
        pub cards: Vec<u8>,        // Raw indices for internal use
        pub cards_js: Vec<String>, // JavaScript-friendly card names
        pub card_count: usize,
    }

    #[derive(Serialize)]
    pub struct InitialHandsJSResponse {
        pub player_id: PlayerId,
        pub cards: Vec<String>, // Only JS format
        pub card_count: usize,
    }

    #[derive(Deserialize)]
    pub struct DrawCardRequest {
        pub player_id: PlayerId,
    }

    #[derive(Deserialize)]
    pub struct DrawMultipleRequest {
        pub player_id: PlayerId,
        pub count: u8,
    }

    #[derive(Serialize)]
    pub struct DrawCardResponse {
        pub success: bool,
        pub card: Option<u8>,
        pub card_js: Option<String>, // Added JS format
        pub message: String,
        pub player_id: PlayerId,
        pub new_hand_size: usize,
    }

    #[derive(Serialize)]
    pub struct DrawMultipleResponse {
        pub success: bool,
        pub cards: Vec<u8>,
        pub cards_js: Vec<String>,
        pub message: String,
        pub player_id: PlayerId,
        pub new_hand_size: usize,
    }

    #[derive(Deserialize)]
    pub struct PlayCardRequest {
        pub player_id: PlayerId,
        pub card_index: usize,
    }

    #[derive(Serialize)]
    pub struct PlayCardResponse {
        pub success: bool,
        pub played_card: u8,
        pub played_card_js: String,
        pub message: String,
        pub player_id: PlayerId,
        pub new_hand_size: usize,
    }

    #[derive(Serialize)]
    pub struct GameStateResponse {
        pub game_state: GameStateJS,
        pub player_count: usize,
        pub total_cards: usize,
    }

    #[derive(Serialize)]
    pub struct GameStatusResponse {
        pub is_initialized: bool,
        pub is_shuffled: bool,
        pub player_count: usize,
        pub draw_pile_count: usize,
        pub discard_pile_count: usize,
        pub total_cards: usize,
    }

    #[derive(Serialize)]
    pub struct ErrorResponse {
        pub error: String,
        pub code: String,
    }

    #[derive(Deserialize)]
    pub struct GameStateQuery {
        pub include_all_hands: Option<bool>,
        pub format: Option<String>, // "js" or "raw"
    }

    fn game_not_found(game_id: &str) -> (StatusCode, Json<ErrorResponse>) {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Game {} not found", game_id),
                code: "GAME_NOT_FOUND".to_string(),
            }),
        )
    }

    // ============================================================================
    // CORE GAME ENDPOINTS
    // ============================================================================

    pub async fn shuffle_and_deal(
        State(games): State<SharedGames>,
        Path(game_id): Path<String>,
        Json(payload): Json<ShuffleAndDealRequest>,
    ) -> Result<Json<ShuffleAndDealResponse>, (StatusCode, Json<ErrorResponse>)> {
        match game::shuffle_and_deal(payload.players, payload.cards_per_player).await {
            Ok(new_game_state) => {
                let cards_remaining = new_game_state.draw_pile.len();
                let seed_used = new_game_state.seed_used;

                {
                    let mut games_guard = games.write().await;
                    games_guard.insert(game_id.clone(), new_game_state);
                }

                Ok(Json(ShuffleAndDealResponse {
                    success: true,
                    message: "Game successfully shuffled and dealt".to_string(),
                    total_players: payload.players,
                    cards_per_player: payload.cards_per_player,
                    cards_remaining_in_deck: cards_remaining,
                    seed_used,
                }))
            }
            Err(e) => Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                    code: "SHUFFLE_DEAL_FAILED".to_string(),
                }),
            )),
        }
    }

    pub async fn get_initial_hands(
        State(games): State<SharedGames>,
        Path((game_id, player_id)): Path<(String, PlayerId)>,
    ) -> Result<Json<InitialHandsResponse>, (StatusCode, Json<ErrorResponse>)> {
        let games_guard = games.read().await;
        let state = match games_guard.get(&game_id) {
            Some(state) => state,
            None => return Err(game_not_found(&game_id)),
        };

        match game::get_initial_hands(state, player_id) {
            Ok(cards) => {
                let cards_js = game::convert_indexes_to_js_cards(&cards);
                let card_count = cards.len();

                Ok(Json(InitialHandsResponse {
                    player_id,
                    cards,
                    cards_js,
                    card_count,
                }))
            }
            Err(e) => Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                    code: "GET_HAND_FAILED".to_string(),
                }),
            )),
        }
    }

    // JavaScript-optimized version (only returns JS format)
    pub async fn get_initial_hands_js(
        State(games): State<SharedGames>,
        Path((game_id, player_id)): Path<(String, PlayerId)>,
    ) -> Result<Json<InitialHandsJSResponse>, (StatusCode, Json<ErrorResponse>)> {
        let games_guard = games.read().await;
        let state = match games_guard.get(&game_id) {
            Some(state) => state,
            None => return Err(game_not_found(&game_id)),
        };

        match game::serialize_player_hand_js(state, player_id) {
            Ok(cards) => {
                let card_count = cards.len();
                Ok(Json(InitialHandsJSResponse {
                    player_id,
                    cards,
                    card_count,
                }))
            }
            Err(e) => Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                    code: "GET_HAND_JS_FAILED".to_string(),
                }),
            )),
        }
    }

    pub async fn draw_card(
        State(games): State<SharedGames>,
        Path(game_id): Path<String>,
        Json(payload): Json<DrawCardRequest>,
    ) -> Result<Json<DrawCardResponse>, (StatusCode, Json<ErrorResponse>)> {
        let mut games_guard = games.write().await;
        let state = match games_guard.get_mut(&game_id) {
            Some(state) => state,
            None => return Err(game_not_found(&game_id)),
        };

        match game::draw_card(state, payload.player_id) {
            Ok(card) => {
                let new_hand_size = state
                    .player_hands
                    .get(payload.player_id as usize)
                    .map(|hand| hand.len())
                    .unwrap_or(0);

                let card_js = game::convert_card_to_js(card);

                Ok(Json(DrawCardResponse {
                    success: true,
                    card: Some(card),
                    card_js: Some(card_js),
                    message: "Card drawn successfully".to_string(),
                    player_id: payload.player_id,
                    new_hand_size,
                }))
            }
            Err(e) => Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                    code: "DRAW_CARD_FAILED".to_string(),
                }),
            )),
        }
    }

    pub async fn draw_multiple_cards(
        State(games): State<SharedGames>,
        Path(game_id): Path<String>,
        Json(payload): Json<DrawMultipleRequest>,
    ) -> Result<Json<DrawMultipleResponse>, (StatusCode, Json<ErrorResponse>)> {
        let mut games_guard = games.write().await;
        let state = match games_guard.get_mut(&game_id) {
            Some(state) => state,
            None => return Err(game_not_found(&game_id)),
        };

        match game::draw_multiple_cards(state, payload.player_id, payload.count) {
            Ok(cards) => {
                let cards_js = game::convert_indexes_to_js_cards(&cards);
                let new_hand_size = state
                    .player_hands
                    .get(payload.player_id as usize)
                    .map(|hand| hand.len())
                    .unwrap_or(0);

                Ok(Json(DrawMultipleResponse {
                    success: true,
                    cards,
                    cards_js,
                    message: format!("Successfully drew {} cards", payload.count),
                    player_id: payload.player_id,
                    new_hand_size,
                }))
            }
            Err(e) => Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                    code: "DRAW_MULTIPLE_FAILED".to_string(),
                }),
            )),
        }
    }

    pub async fn play_card(
        State(games): State<SharedGames>,
        Path(game_id): Path<String>,
        Json(payload): Json<PlayCardRequest>,
    ) -> Result<Json<PlayCardResponse>, (StatusCode, Json<ErrorResponse>)> {
        let mut games_guard = games.write().await;
        let state = match games_guard.get_mut(&game_id) {
            Some(state) => state,
            None => return Err(game_not_found(&game_id)),
        };

        match game::play_card(state, payload.player_id, payload.card_index) {
            Ok(played_card) => {
                let played_card_js = game::convert_card_to_js(played_card);
                let new_hand_size = state
                    .player_hands
                    .get(payload.player_id as usize)
                    .map(|hand| hand.len())
                    .unwrap_or(0);

                Ok(Json(PlayCardResponse {
                    success: true,
                    played_card,
                    played_card_js,
                    message: "Card played successfully".to_string(),
                    player_id: payload.player_id,
                    new_hand_size,
                }))
            }
            Err(e) => Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                    code: "PLAY_CARD_FAILED".to_string(),
                }),
            )),
        }
    }

    pub async fn get_game_state(
        State(games): State<SharedGames>,
        Path(game_id): Path<String>,
        Query(params): Query<GameStateQuery>,
    ) -> Result<Json<GameStateResponse>, (StatusCode, Json<ErrorResponse>)> {
        let games_guard = games.read().await;
        let state = match games_guard.get(&game_id) {
            Some(state) => state,
            None => return Err(game_not_found(&game_id)),
        };

        let mut game_state_js = GameStateJS::from(state);

        if !params.include_all_hands.unwrap_or(false) {
            for hand in &mut game_state_js.player_hands {
                hand.clear();
                hand.push("HIDDEN".to_string());
            }
        }

        Ok(Json(GameStateResponse {
            game_state: game_state_js,
            player_count: state.player_count(),
            total_cards: state.total_cards(),
        }))
    }

    pub async fn get_game_status(
        State(games): State<SharedGames>,
        Path(game_id): Path<String>,
    ) -> Result<Json<GameStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
        let games_guard = games.read().await;
        let state = match games_guard.get(&game_id) {
            Some(state) => state,
            None => return Err(game_not_found(&game_id)),
        };

        Ok(Json(GameStatusResponse {
            is_initialized: state.is_initialized(),
            is_shuffled: state.is_shuffled,
            player_count: state.player_count(),
            draw_pile_count: state.draw_pile.len(),
            discard_pile_count: state.discard_pile.len(),
            total_cards: state.total_cards(),
        }))
    }

    pub async fn get_top_discard_card(
        State(games): State<SharedGames>,
        Path(game_id): Path<String>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
        let games_guard = games.read().await;
        let state = match games_guard.get(&game_id) {
            Some(state) => state,
            None => return Err(game_not_found(&game_id)),
        };

        if let Some(&top_card) = state.discard_pile.last() {
            let card_js = game::convert_card_to_js(top_card);
            Ok(Json(serde_json::json!({
                "card": top_card,
                "card_js": card_js,
                "has_card": true
            })))
        } else {
            Ok(Json(serde_json::json!({
                "card": null,
                "card_js": null,
                "has_card": false
            })))
        }
    }

    // ============================================================================
    // UTILITY ENDPOINTS
    // ============================================================================

    pub async fn reset_game(
        State(games): State<SharedGames>,
        Path(game_id): Path<String>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
        let mut games_guard = games.write().await;
        let was_existing = games_guard.contains_key(&game_id);
        games_guard.insert(game_id.clone(), GameState::new());

        Ok(Json(serde_json::json!({
            "success": true,
            "message": if was_existing {
                format!("Game {} reset successfully", game_id)
            } else {
                format!("Game {} created and reset", game_id)
            }
        })))
    }

    pub async fn get_player_hand_count(
        State(games): State<SharedGames>,
        Path((game_id, player_id)): Path<(String, PlayerId)>,
    ) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
        let games_guard = games.read().await;
        let state = match games_guard.get(&game_id) {
            Some(state) => state,
            None => return Err(game_not_found(&game_id)),
        };

        if state.is_valid_player(player_id) {
            let hand_size = state.player_hands[player_id as usize].len();
            Ok(Json(serde_json::json!({
                "game_id": game_id,
                "player_id": player_id,
                "hand_size": hand_size
            })))
        } else {
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Player {} not found in game {}", player_id, game_id),
                    code: "PLAYER_NOT_FOUND".to_string(),
                }),
            ))
        }
    }

    // Health check endpoint
    pub async fn health_check() -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "status": "healthy",
            "service": "uno-game-api",
            "timestamp": chrono::Utc::now().timestamp()
        }))
    }
}
