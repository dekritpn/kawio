use kawio::*;

use clap::Parser;
use std::env;
use std::fs;
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;

use crate::game::{Game, Move, Player};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run in training mode
    #[arg(long)]
    train: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    if args.train {
        run_training();
    } else {
        run_server().await?;
    }
    Ok(())
}

fn run_training() {
    let num_games = 1000;
    let stats_file = "training_stats.txt";
    let mut start_game = 1;
    let mut black_wins = 0;
    let mut white_wins = 0;
    let mut draws = 0;
    let mut total_moves = 0;

    if let Ok(content) = fs::read_to_string(stats_file) {
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() >= 5 {
            start_game = lines[0].parse().unwrap_or(1) + 1; // start from next
            black_wins = lines[1].parse().unwrap_or(0);
            white_wins = lines[2].parse().unwrap_or(0);
            draws = lines[3].parse().unwrap_or(0);
            total_moves = lines[4].parse().unwrap_or(0);
        }
    }

    for game_num in start_game..=num_games {
        let mut game = Game::new();
        let mut moves_count = 0;

        while !game.is_game_over() {
            match ai::AI::get_move(&game) {
                Some(Move::Place(pos)) => {
                    game.make_move(pos).unwrap();
                    moves_count += 1;
                }
                Some(Move::Pass) => {
                    game.pass();
                }
                None => {
                    game.pass();
                }
            }
        }

        total_moves += moves_count;

        match game.winner() {
            Some(Player::Black) => black_wins += 1,
            Some(Player::White) => white_wins += 1,
            None => draws += 1,
        }

        let content = format!("{}\n{}\n{}\n{}\n{}", game_num, black_wins, white_wins, draws, total_moves);
        fs::write(stats_file, content).unwrap();

        if game_num % 100 == 0 {
            let avg_moves = total_moves as f64 / game_num as f64;
            let black_win_rate = black_wins as f64 / game_num as f64;
            let white_win_rate = white_wins as f64 / game_num as f64;
            let draw_rate = draws as f64 / game_num as f64;
            println!("Games: {}, Black wins: {:.2}%, White wins: {:.2}%, Draws: {:.2}%, Avg moves: {:.2}",
                     game_num, black_win_rate * 100.0, white_win_rate * 100.0, draw_rate * 100.0, avg_moves);
        }
    }

    println!("Training complete. Total games: {}", num_games);
    let _ = fs::remove_file(stats_file);
}

async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("0.0.0.0:{}", port);

    let sessions = Arc::new(Mutex::new(state::Sessions::new()));
    let api_router = network::create_router(sessions);
    let app = api_router.fallback_service(ServeDir::new("web"));

    let listener = tokio::net::TcpListener::bind(&address).await?;
    tracing::info!("Server running on http://{}", address);
    axum::serve(listener, app).await?;
    Ok(())
}
