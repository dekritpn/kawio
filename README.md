# Kawio: An Othello Game Server

Kawio is a modern Othello game server written in Rust, designed for performance, reliability, and extensibility. It provides complete game logic, session handling, and optional database integration for persistent multiplayer play.

## 🕹️ About Othello

Othello is a classic two-player strategy game played on an 8×8 board. Players alternate turns placing discs, flipping their opponent’s pieces to their own color when trapped between two of theirs. The goal is to finish the game with the majority of discs in your color.

See the Wikipedia page on [Othello](https://en.wikipedia.org/wiki/Reversi) for a complete description of the rules and strategies.

## ✨ Features

* Full Game Logic — Implements the complete Othello ruleset.
* Session Management — Tracks multiple simultaneous games.
* High-Performance Board — Board state stored in 64-bit integers.
* Database Integration — SQLite-based storage for persistence.
* Real-Time Communication — WebSocket and REST API support.
* AI Opponent — Monte Carlo Tree Search (MCTS) for single-player mode.
* Secure Authentication — JWT-based session tokens.
* Matchmaking & Leaderboard — Automatic player pairing and ELO rating system.
* Web Frontend — Simple browser-based client for testing and playing.

## ⚙️ Architecture Overview

The Kawio server is structured into several core modules: `ai`, `auth`, `game`, `network`, `state`, and `storage`. The game board is encoded as two 64-bit bitboards for high-performance bitwise operations.

## 🚀 Getting Started

### Prerequisites
* Rust (latest stable)
* Cargo build system

### Build and Run
```bash
git clone https://github.com/dekritpn/kawio.git
cd kawio
cargo run --release
```

The server will start on port `8080`. Open a web browser and navigate to `http://localhost:8080` to play.

## 🔌 API Documentation

The server provides a REST API for managing matches, players, and game state. For detailed information on endpoints and usage, see the [API Documentation](./docs/api.md).

## 🧩 Development

Run tests with `cargo test` and benchmarks with `cargo bench`. To enable debug logs, run:
```bash
RUST_LOG=kawio=debug cargo run
```

## 📈 Roadmap

Future enhancements may include a tournament mode, a mobile client, and multi-language support.

## 🤝 Contributing

Pull requests, bug reports, and suggestions are welcome! Please see our [Contributing Guidelines](./CONTRIBUTING.md) for more information.

## 📜 License

This project is licensed under the MIT License — see the [LICENSE](./LICENSE) file for details.
