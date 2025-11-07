# TODO List - Code Review Recommendations

This TODO list contains recommendations from the code review to improve the Kawio Othello game server project.

## High Priority
- [ ] **fix_game_over_logic**: Fix game-over logic in game.rs: Change is_game_over() to check passes == 2 instead of >=1, and ensure proper pass handling
- [ ] **add_authentication**: Implement authentication and session validation in network.rs to prevent unauthorized moves

## Medium Priority
- [ ] **improve_error_handling**: Replace unwrap() calls in main.rs with proper error handling and propagation
- [ ] **fix_storage_casting**: Add bounds checking in storage.rs for i64 to u64 casting to prevent overflow
- [ ] **add_integration_tests**: Expand testing: Add integration tests for API endpoints, state management, and storage
- [ ] **enhance_ai**: Enhance AI in ai.rs with minimax algorithm and alpha-beta pruning as planned in README

## Low Priority
- [ ] **add_documentation**: Add doc comments (///) to public functions, especially complex logic in game.rs like flips()
- [ ] **add_configuration**: Add environment variable support for configuration (port, DB path) as hinted in README
- [ ] **implement_planned_features**: Implement planned features: WebSocket support, matchmaking system, leaderboards, and ELO rating
- [ ] **run_linting_formatting**: Run cargo clippy and cargo fmt for linting and style consistency
- [ ] **benchmark_performance**: Add benchmarking with criterion for bitboard operations as mentioned in README