use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::state::Sessions;
use crate::game::Game;
use tracing;

#[derive(Deserialize)]
struct NewMatchRequest {
    player1: String,
    player2: String,
}

#[derive(Serialize)]
struct NewMatchResponse {
    id: String,
}

#[derive(Deserialize)]
struct MoveRequest {
    coord: String,
    player: String,
}

#[derive(Serialize)]
struct GameStateResponse {
    board: Vec<Vec<String>>,
    current_player: String,
    legal_moves: Vec<String>,
    game_over: bool,
    winner: Option<String>,
}

pub fn create_router(sessions: Arc<Mutex<Sessions>>) -> Router {
    Router::new()
        .route("/match/new", post(create_match))
        .route("/match/:id/move", post(make_move))
        .route("/match/:id/state", get(get_state))
        .with_state(sessions)
}

async fn create_match(
    State(sessions): State<Arc<Mutex<Sessions>>>,
    Json(_req): Json<NewMatchRequest>,
) -> Result<Json<NewMatchResponse>, StatusCode> {
    let mut sessions = sessions.lock().unwrap();
    let id = sessions.create_game();
    tracing::info!("Created game: {}", id);
    Ok(Json(NewMatchResponse { id }))
}

async fn make_move(
    State(sessions): State<Arc<Mutex<Sessions>>>,
    Path(id): Path<String>,
    Json(req): Json<MoveRequest>,
) -> Result<(), StatusCode> {
    let pos = Game::coord_to_pos(&req.coord).ok_or(StatusCode::BAD_REQUEST)?;
    let mut sessions = sessions.lock().unwrap();
    sessions.make_move(&id, pos).map_err(|_| StatusCode::BAD_REQUEST)?;
    tracing::info!("Move made in game {}: {}", id, req.coord);
    Ok(())
}

async fn get_state(
    State(sessions): State<Arc<Mutex<Sessions>>>,
    Path(id): Path<String>,
) -> Result<Json<GameStateResponse>, StatusCode> {
    let sessions = sessions.lock().unwrap();
    let game = sessions.get_game(&id).ok_or(StatusCode::NOT_FOUND)?;
    let board = game_to_board(game);
    let legal_moves = game.legal_moves().iter().map(|&p| Game::pos_to_coord(p)).collect();
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
    }))
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