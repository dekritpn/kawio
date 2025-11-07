# Kawio: An Othello Game Server

Kawio is a modern Othello game server written in Rust, designed for performance, reliability, and extensibility. It provides complete game logic, session handling, and optional database integration for persistent multiplayer play.

## 🕹️ About Othello

Othello is a classic two-player strategy game played on an 8×8 board. Players alternate turns placing discs, flipping their opponent’s pieces to their own color when trapped between two of theirs. The goal is to finish the game with the majority of discs in your color.

See the Wikipedia page on [Othello https://en.wikipedia.org/wiki/Reversi]
 for a complete description of the rules and strategies.

## ✨ Features

* Full Game Logic — Implements the complete Othello ruleset, including legal move detection, pass logic, and game-over conditions.

* Session Management — Tracks multiple simultaneous games efficiently.

* Compact Game Representation — Board state stored in 64-bit integers for maximum performance.

* In-Game Timers — Optional countdowns per turn or total match duration.

* Database Integration — Pluggable storage layer (e.g., PostgreSQL, SQLite) for persistent sessions or leaderboards.

* WebSocket Support (optional) — Real-time communication for multiplayer clients.

* REST API (planned) — For turn submissions, match creation, and querying stats.

* AI Player Interface (planned) — Standardized interface for bot or AI opponents.

## ⚙️ Architecture Overview

The Kawio server is structured into several core modules:

Module	Description
game/	Implements game rules and move logic.
state/	Manages sessions and board representations.
network/	Handles client connections and message routing.
storage/	Abstract database layer (can be disabled for local play).
timer/	Provides countdown timers and turn expiration logic.

The game board is encoded as two 64-bit bitboards (one per player). This allows fast bitwise operations for move validation and flipping discs.

## 🧭 Coordinate System

Coordinates follow the standard notation:
Columns A–H and rows 1–8.
Example: top-left = A1, bottom-right = H8.

## 🚀 Getting Started
### Prerequisites
* Rust (latest stable)
* Cargo build system
* (optional) PostgreSQL or SQLite if database features are enabled.

### Build and Run
git clone https://github.com/dekritpn/kawio.git
cd kawio
cargo run --release


The server will start on port 8080 by default.
Configuration can be customized via .env or CLI arguments.

## 🔌 Example API (coming soon)
### Create a new match
POST /match/new
{
  "player1": "Alice",
  "player2": "Bob"
}

### Make a move
POST /match/{id}/move
{
  "coord": "D3",
  "player": "Alice"
}

### Get current board
GET /match/{id}/state

## 🧩 Development Notes

Tests are provided for all core rules under tests/.

Bitboard operations are benchmarked to ensure sub-millisecond move validation.

To enable debug logs, run:

RUST_LOG=kawio=debug cargo run

## 📈 Roadmap

 * REST API endpoints
 * WebSocket real-time updates
 * AI opponent module (minimax + alpha-beta pruning)
 * Leaderboard and ELO rating system
 * Frontend example (Web + CLI)

## 🤝 Contributing

Pull requests, bug reports, and suggestions are welcome!
Please open an issue first to discuss major changes.

## 📜 License

This project is licensed under the MIT License — see the LICENSE file for details.

## 🔧 A Functional Game Server

To make Kawio a complete, usable game server, these are essential features we are trying to achieve

1. Networking Layer — REST or WebSocket endpoints to handle moves, matchmaking, and game state queries.
2. Authentication/Session Tokens — To identify players and ensure fair play.
3. Matchmaking System — To pair players automatically or manage game invitations.
4. Persistence Layer — For storing player data, results, and leaderboards.
5. Logging and Error Handling — Structured logs (e.g., via tracing crate).
6. Testing and Benchmarking — For core rules and performance-critical paths.
7. AI Interface — Optional bot integration for single-player or training.
