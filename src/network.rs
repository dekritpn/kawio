use crate::ai::AI;
use crate::auth::Auth;
use crate::game::Game;
use crate::state::Sessions;
use crate::storage::PlayerStats;
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::{
    async_trait,
    extract::{FromRequestParts, Path, State},
    http::{header, request::Parts, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct AuthenticatedPlayer(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedPlayer
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "));

        if let Some(token) = auth_header {
            match Auth::validate_token(token) {
                Ok(claims) => Ok(AuthenticatedPlayer(claims.sub)),
                Err(_) => Err(StatusCode::UNAUTHORIZED),
            }
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[derive(Deserialize)]
struct LoginRequest {
    player: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

#[derive(Deserialize)]
struct NewMatchRequest {
    player2: String,
}

#[derive(Serialize)]
struct NewMatchResponse {
    id: String,
}

#[derive(Deserialize)]
struct MoveRequest {
    coord: String,
}

#[derive(Serialize)]
struct GameStateResponse {
    board: Vec<Vec<String>>,
    current_player: String,
    legal_moves: Vec<String>,
    game_over: bool,
    winner: Option<String>,
    player1: String,
    player2: String,
    scores: HashMap<String, u32>,
}

#[derive(Deserialize)]
struct JoinRequest {
    player: String,
}

#[derive(Serialize)]
struct JoinResponse {
    matched: bool,
    id: Option<String>,
}

pub fn create_router(sessions: Arc<Mutex<Sessions>>) -> Router {
    Router::new()
        .route("/auth/login", post(login))
        .route("/match/new", post(create_match))
        .route("/match/join", post(join_matchmaking))
        .route("/match/:id/move", post(make_move))
        .route("/match/:id/state", get(get_state))
        .route("/match/:id/ws", get(ws_handler))
        .route("/leaderboard", get(get_leaderboard))
        .with_state(sessions)
}

async fn login(Json(req): Json<LoginRequest>) -> Result<Json<LoginResponse>, StatusCode> {
    match Auth::generate_token(&req.player) {
        Ok(token) => Ok(Json(LoginResponse { token })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_match(
    State(sessions): State<Arc<Mutex<Sessions>>>,
    AuthenticatedPlayer(player1): AuthenticatedPlayer,
    Json(req): Json<NewMatchRequest>,
) -> Result<Json<NewMatchResponse>, StatusCode> {
    if (player1 == "AI" && req.player2 != "AI") || (player1 != "AI" && req.player2 == "AI") {
        let mut sessions = sessions.lock().unwrap();
        let id = sessions.create_game(player1, &req.player2);
        tracing::info!("Created game: {}", id);
        return Ok(Json(NewMatchResponse { id }));
    }
    Err(StatusCode::BAD_REQUEST)
}

async fn make_move(
    State(sessions): State<Arc<Mutex<Sessions>>>,
    Path(id): Path<String>,
    AuthenticatedPlayer(player): AuthenticatedPlayer,
    Json(req): Json<MoveRequest>,
) -> Result<(), StatusCode> {
    let Ok(pos) = Game::coord_to_pos(&req.coord) else {
        return Err(StatusCode::BAD_REQUEST);
    };
    let mut sessions = sessions.lock().unwrap();
    sessions.make_move(&id, pos, &player).map_err(|_| StatusCode::BAD_REQUEST)?;
    let (p1, p2) = sessions.get_players(&id).unwrap();
    let game = sessions.get_game(&id).unwrap();
    let current_player_name = match game.current_player {
        crate::game::Player::Black => p1,
        crate::game::Player::White => p2,
    };
    if current_player_name == "AI" {
        if let Some(ai_move) = AI::get_move(game) {
            sessions.make_move(&id, ai_move, "AI").map_err(|_| StatusCode::BAD_REQUEST)?;
        } else {
            // AI has no moves, pass
            sessions.pass(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
        }
    }
    Ok(())
}

async fn get_state(
    State(sessions): State<Arc<Mutex<Sessions>>>,
    Path(id): Path<String>,
) -> Result<Json<GameStateResponse>, StatusCode> {
    let sessions = sessions.lock().unwrap();
    let game = sessions.get_game(&id).ok_or(StatusCode::NOT_FOUND)?;
    let (player1, player2) = sessions.get_players(&id).ok_or(StatusCode::NOT_FOUND)?;
    let board = game_to_board(game);
    let legal_moves = game
        .legal_moves()
        .iter()
        .map(|p| Game::pos_to_coord(*p))
        .collect();
    let current_player = match game.current_player {
        crate::game::Player::Black => "Black".to_string(),
        crate::game::Player::White => "White".to_string(),
    };
    let winner = game.winner().map(|p| match p {
        crate::game::Player::Black => "Black".to_string(),
        crate::game::Player::White => "White".to_string(),
    });
    let scores = game.scores();
    let mut scores_map = HashMap::new();
    scores_map.insert("B".to_string(), scores.0);
    scores_map.insert("W".to_string(), scores.1);
    Ok(Json(GameStateResponse {
        board,
        current_player,
        legal_moves,
        game_over: game.is_game_over(),
        winner,
        player1: player1.clone(),
        player2: player2.clone(),
        scores: scores_map,
    }))
}

async fn join_matchmaking(
    State(sessions): State<Arc<Mutex<Sessions>>>,
    AuthenticatedPlayer(player): AuthenticatedPlayer,
) -> Result<Json<JoinResponse>, StatusCode> {
    let mut sessions = sessions.lock().unwrap();
    if let Some(id) = sessions.join_matchmaking(player) {
        Ok(Json(JoinResponse {
            matched: true,
            id: Some(id),
        }))
    } else {
        Ok(Json(JoinResponse {
            matched: false,
            id: None,
        }))
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(sessions): State<Arc<Mutex<Sessions>>>,
    Path(id): Path<String>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, sessions, id))
}

async fn handle_socket(mut socket: WebSocket, sessions: Arc<Mutex<Sessions>>, id: String) {
    // Send initial state right after connection
    send_state(&mut socket, &sessions, &id).await;

    while let Some(Ok(msg)) = socket.recv().await {
        if let axum::extract::ws::Message::Text(text) = msg {
            #[derive(Deserialize)]
            struct ClientMessage {
                r#type: String,
                coord: Option<String>,
            }

            if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                {
                    let mut sessions_guard = sessions.lock().unwrap();
                    let (p1, p2) = sessions_guard.get_players(&id).unwrap().clone();

                    let player_name = if sessions_guard.get_game(&id).unwrap().current_player == crate::game::Player::Black {
                        p1.clone()
                    } else {
                        p2.clone()
                    };

                    if client_msg.r#type == "move" {
                        if let Some(coord) = client_msg.coord {
                            let Ok(pos) = Game::coord_to_pos(&coord) else {
                                continue; // Invalid coord
                            };

                            if sessions_guard.make_move(&id, pos, &player_name).is_ok() {
                                let game = sessions_guard.get_game(&id).unwrap();
                                let current_player_name = if game.current_player == crate::game::Player::Black {
                                    &p1
                                } else {
                                    &p2
                                };

                                if current_player_name == "AI" {
                                    if let Some(ai_move) = AI::get_move(game) {
                                        sessions_guard.make_move(&id, ai_move, "AI").unwrap();
                                    } else {
                                        sessions_guard.pass(&id).unwrap();
                                    }
                                }
                            }
                        }
                    } else if client_msg.r#type == "pass" {
                        if sessions_guard.pass(&id).is_ok() {
                            let game = sessions_guard.get_game(&id).unwrap();
                            let current_player_name = if game.current_player == crate::game::Player::Black {
                                &p1
                            } else {
                                &p2
                            };

                            if current_player_name == "AI" {
                                if let Some(ai_move) = AI::get_move(game) {
                                    sessions_guard.make_move(&id, ai_move, "AI").unwrap();
                                } else {
                                    sessions_guard.pass(&id).unwrap();
                                }
                            }
                        }
                    }
                }
                send_state(&mut socket, &sessions, &id).await;
            }
        }
    }
}
async fn send_state(socket: &mut WebSocket, sessions: &Arc<Mutex<Sessions>>, id: &str) {
    let (state, legal_moves_empty) = {
        let sessions = sessions.lock().unwrap();
        let mut data = None;
        let mut legal_moves: Vec<String> = Vec::new();
        if let Some(game) = sessions.get_game(id) {
            legal_moves = game.legal_moves().iter().map(|p| Game::pos_to_coord(*p)).collect();
            let (player1, player2) = sessions.get_players(id).unwrap();
            let board = game_to_board(game);
            let current_player = match game.current_player {
                crate::game::Player::Black => "Black".to_string(),
                crate::game::Player::White => "White".to_string(),
            };
            let winner = game.winner().map(|p| match p {
                crate::game::Player::Black => "Black".to_string(),
                crate::game::Player::White => "White".to_string(),
            });
            data = Some(serde_json::json!({
                "board": board,
                "current_player": current_player,
                "legal_moves": legal_moves,
                "game_over": game.is_game_over(),
                "winner": winner,
                "player1": player1.clone(),
                "player2": player2.clone(),
                "scores": { "B": game.scores().0, "W": game.scores().1 }
            }));
        }
        (data, legal_moves.is_empty())
    };

    if let Some(state) = state {
        if socket.send(axum::extract::ws::Message::Text(state.to_string())).await.is_err() {
            return;
        }
        if legal_moves_empty
            && socket
                .send(axum::extract::ws::Message::Text(
                    serde_json::json!({
                        "type": "status",
                        "message": "No legal moves available, you must pass."
                    })
                    .to_string(),
                ))
                .await
                .is_err()
        {
        }
    }
}

async fn get_leaderboard(
    State(sessions): State<Arc<Mutex<Sessions>>>,
) -> Result<Json<Vec<PlayerStats>>, StatusCode> {
    let sessions = sessions.lock().unwrap();
    let stats = sessions
        .storage
        .get_leaderboard()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(stats))
}

fn game_to_board(game: &Game) -> Vec<Vec<String>> {
    let mut board = vec![vec![".".to_string(); 8]; 8];
    for (row_idx, row) in board.iter_mut().enumerate().take(8) {
        for (col_idx, col) in row.iter_mut().enumerate().take(8) {
            let pos = row_idx * 8 + col_idx;
            let bit = 1u64 << pos;
            if (game.black & bit) != 0 {
                *col = "B".to_string();
            } else if (game.white & bit) != 0 {
                *col = "W".to_string();
            }
        }
    }
    board
}
