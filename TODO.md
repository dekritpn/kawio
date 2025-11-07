# TODO List - Code Review Recommendations

This TODO list contains recommendations from the code review to improve the Kawio Othello game server project.

## High Priority
- [x] **fix_game_over_logic**: Fix game-over logic in game.rs: Change is_game_over() to check passes == 2 instead of >=1, and ensure proper pass handling
- [x] **add_authentication**: Implement authentication and session validation in network.rs to prevent unauthorized moves

## Medium Priority
- [x] **improve_error_handling**: Replace unwrap() calls in main.rs with proper error handling and propagation
- [x] **fix_storage_casting**: Add bounds checking in storage.rs for i64 to u64 casting to prevent overflow
- [x] **add_integration_tests**: Expand testing: Add integration tests for API endpoints, state management, and storage
- [x] **enhance_ai**: Enhance AI in ai.rs with minimax algorithm and alpha-beta pruning as planned in README

## Low Priority
- [x] **add_documentation**: Add doc comments (///) to public functions, especially complex logic in game.rs like flips()
- [x] **add_configuration**: Add environment variable support for configuration (port, DB path) as hinted in README
- [x] **implement_planned_features**: Implement planned features: WebSocket support, matchmaking system, leaderboards, and ELO rating
- [x] **run_linting_formatting**: Run cargo clippy and cargo fmt for linting and style consistency
- [x] **benchmark_performance**: Add benchmarking with criterion for bitboard operations as mentioned in README
- [x] **align_initial_setup**: Align the initial setup to the standard Othello start (Black: E4 & D5, White: D4 & E5) and update any tests/fixtures accordingly.
- [x] **enforce_move_legality**: Make `make_move` enforce legality: reject occupied squares or moves that flip zero discs; return a `Result` (or bool) instead of silently mutating state.
- [x] **add_has_legal_move**: Add `has_legal_move(player)` and implement auto-pass when a player has no legal moves; update `is_game_over` to check both players’ legal moves rather than relying solely on a `passes` counter.
- [x] **normalize_coordinates**: Normalize coordinate input to uppercase (accept “e3” as well as “E3”); validate and surface clear errors for malformed inputs.
- [x] **consistent_notation**: Decide on a consistent board/notation convention (e.g., `A1` bottom-left as in common Othello diagrams) and update rendering/coordinate mapping or document the current choice prominently.
- [x] **constrain_empty**: Constrain `empty()` with an explicit 64-square mask (e.g., `const ALL: u64 = 0xFFFF_FFFF_FFFF_FFFF;` then `!occupied() & ALL`) to future-proof against sentinel bits.
- [x] **expand_tests**: Expand tests: verify the four standard opening legal moves; add edge/corner cases, no-flip rejection, symmetry/capture tests for all 8 directions, and game-over detection with auto-pass.
- [x] **preview_move_helper**: Consider exposing a pure “try move” helper (`preview_move`) that returns the flip mask without mutating state to simplify testing and UIs.
- [x] **add_more_documentation**: Add documentation comments (`///`) for public functions and clarify invariants (bitboard layout, indexing, coordinate system).
- [x] **enable_stricter_lints**: Run `clippy` and enable stricter lints in `Cargo.toml` (e.g., `#![deny(clippy::all, clippy::pedantic)]` selectively) to catch subtle issues early.
