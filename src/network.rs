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
use serde_json;
use std::sync::{Arc, Mutex};
use tracing;

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
        let id = sessions.create_game(player1.clone(), req.player2.clone());
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
    let pos = match Game::coord_to_pos(&req.coord) {
        Ok(p) => p,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
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
        if let Some(ai_move) = AI::get_move(&game) {
            sessions.make_move(&id, ai_move, "AI").map_err(|_| StatusCode::BAD_REQUEST)?;
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
        .map(|&p| Game::pos_to_coord(p))
        .collect();
    let current_player = match game.current_player {
        crate::game::Player::Black => "Black".to_string(),
        crate::game::Player::White => "White".to_string(),
    };
    let winner = game.winner().map(|p| match p {
        crate::game::Player::Black => "Black".to_string(),
        crate::game::Player::White => "White".to_string(),
    });
    Ok(Json(GameStateResponse {
        board,
        current_player,
        legal_moves,
        game_over: game.is_game_over(),
        winner,
        player1: player1.clone(),
        player2: player2.clone(),
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
    loop {
        let data = {
            let sessions = sessions.lock().unwrap();
            if let Some(game) = sessions.get_game(&id) {
                let (player1, player2) = sessions.get_players(&id).unwrap();
                let board = game_to_board(game);
                let legal_moves: Vec<String> = game
                    .legal_moves()
                    .iter()
                    .map(|&p| Game::pos_to_coord(p))
                    .collect();
                let current_player = match game.current_player {
                    crate::game::Player::Black => "Black".to_string(),
                    crate::game::Player::White => "White".to_string(),
                };
                let winner = game.winner().map(|p| match p {
                    crate::game::Player::Black => "Black".to_string(),
                    crate::game::Player::White => "White".to_string(),
                });
                Some((
                    board,
                    legal_moves,
                    current_player,
                    winner,
                    player1.clone(),
                    player2.clone(),
                    game.is_game_over(),
                ))
            } else {
                None
            }
        };
        if let Some((board, legal_moves, current_player, winner, player1, player2, game_over)) =
            data
        {
            let state = serde_json::json!({
                "board": board,
                "current_player": current_player,
                "legal_moves": legal_moves,
                "game_over": game_over,
                "winner": winner,
                "player1": player1,
                "player2": player2,
            });
            if socket
                .send(axum::extract::ws::Message::Text(state.to_string()))
                .await
                .is_err()
            {
                break;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
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
    for row in 0..8 {
        for col in 0..8 {
            let pos = row * 8 + col;
            let bit = 1u64 << pos;
            if (game.black & bit) != 0 {
                board[row][col] = "B".to_string();
            } else if (game.white & bit) != 0 {
                board[row][col] = "W".to_string();
            }
        }
    }
    board
}
