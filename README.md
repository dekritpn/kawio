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

* Database Integration — SQLite-based storage for persistent sessions, player stats, and leaderboards.

* WebSocket Support — Real-time communication for multiplayer clients.

* REST API — For turn submissions, match creation, and querying stats.

* AI Opponent — Minimax algorithm with alpha-beta pruning for single-player mode.

* Authentication — JWT-based session tokens for secure play.

* Matchmaking — Automatic player pairing system.

* Leaderboard — ELO rating system with persistent storage.

* Web Frontend — Simple browser-based client for testing and playing.

## ⚙️ Architecture Overview

The Kawio server is structured into several core modules:

Module	Description
ai/	AI opponent using minimax with alpha-beta pruning.
auth/	JWT-based authentication for secure play.
game/	Implements game rules and move logic.
network/	REST API and WebSocket endpoints.
state/	Manages sessions and board representations.
storage/	SQLite database for persistence.

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
Configuration can be customized via environment variables (e.g., PORT=3000).

### Testing the Server

1. Start the server as above.
2. Open a web browser and navigate to http://localhost:8080.
3. Enter a player name and click "Login" to receive a JWT token.
4. Choose "Create Match vs AI" to play against the AI, or "Join Matchmaking" to wait for another player.
5. Click on the board to make moves. The AI will respond automatically.
6. View the leaderboard to see player stats.

For API testing, use tools like curl or Postman. All protected endpoints require an Authorization header: `Bearer <token>`.

## 🔌 REST API

The Kawio server provides a REST API for managing Othello matches. All endpoints return JSON responses. Protected endpoints require JWT authentication via `Authorization: Bearer <token>` header.

### Login
**POST /auth/login**

Authenticates a player and returns a JWT token.

**Request Body:**
```json
{
  "player": "Alice"
}
```

**Response (200 OK):**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
}
```

### Create a New Match
**POST /match/new** (requires auth)

Creates a new game between the authenticated player and another player (e.g., "AI").

**Request Body:**
```json
{
  "player2": "AI"
}
```

**Response (200 OK):**
```json
{
  "id": "abc123"
}
```

### Join Matchmaking
**POST /match/join** (requires auth)

Joins the matchmaking queue. If another player is waiting, a match is created automatically.

**Response (200 OK):**
```json
{
  "matched": true,
  "id": "abc123"
}
```
If no match is available, returns `{"matched": false, "id": null}`.

### Make a Move
**POST /match/{id}/move** (requires auth)

Makes a move in the specified game. Coordinates use standard notation (e.g., "D3").

**Request Body:**
```json
{
  "coord": "D3"
}
```

**Response (200 OK):** Empty body on success.

**Error Responses:**
- 400 Bad Request: Invalid coordinate or illegal move.
- 401 Unauthorized: Invalid or missing token.
- 404 Not Found: Game ID does not exist.

### Get Game State
**GET /match/{id}/state**

Retrieves the current state of the game.

**Response (200 OK):**
```json
{
  "board": [
    [".", ".", ".", ".", ".", ".", ".", "."],
    [".", ".", ".", ".", ".", ".", ".", "."],
    [".", ".", ".", ".", ".", ".", ".", "."],
    [".", ".", ".", "W", "B", ".", ".", "."],
    [".", ".", ".", "B", "W", ".", ".", "."],
    [".", ".", ".", ".", ".", ".", ".", "."],
    [".", ".", ".", ".", ".", ".", ".", "."],
    [".", ".", ".", ".", ".", ".", ".", "."]
  ],
  "current_player": "Black",
  "legal_moves": ["C4", "D3", "E6", "F5"],
  "game_over": false,
  "winner": null,
  "player1": "Alice",
  "player2": "Bob"
}
```

**Error Responses:**
- 404 Not Found: Game ID does not exist.

### WebSocket Connection
**GET /match/{id}/ws**

Establishes a WebSocket connection for real-time game updates. The server sends periodic JSON updates of the game state.

### Get Leaderboard
**GET /leaderboard**

Retrieves the current leaderboard with player statistics.

**Response (200 OK):**
```json
[
  {
    "name": "Alice",
    "wins": 10,
    "losses": 5,
    "elo": 1200
  },
  {
    "name": "Bob",
    "wins": 8,
    "losses": 7,
    "elo": 1150
  }
]
```

## 🧩 Development Notes

Tests are provided for all core rules under tests/.

Bitboard operations are benchmarked in benches/ to ensure sub-millisecond move validation.

To enable debug logs, run:

RUST_LOG=kawio=debug cargo run

Run benchmarks with: `cargo bench`

Run tests with: `cargo test`

## 📈 Roadmap

All core features implemented! Future enhancements may include:

 * Advanced AI algorithms (e.g., neural networks)
 * Tournament mode
 * Mobile app client
 * Multi-language support

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
