use kawio::game::Game;
use kawio::state::Sessions;
use kawio::storage::Storage;

#[test]
fn test_sessions_create_game() {
    let mut sessions = Sessions::new();
    let id = sessions.create_game("Alice".to_string(), "Bob".to_string());
    assert!(sessions.has_game(&id));
    assert!(sessions.has_player(&id));
}

#[test]
fn test_sessions_make_move() {
    let mut sessions = Sessions::new();
    let id = sessions.create_game("Alice".to_string(), "Bob".to_string());
    let game = sessions.get_game(&id).unwrap();
    let moves = game.legal_moves();
    assert!(!moves.is_empty());
    let pos = moves[0];
    assert!(sessions.make_move(&id, pos, "Alice").is_ok());
    let game = sessions.get_game(&id).unwrap();
    assert_eq!(game.current_player, kawio::game::Player::White);
}

#[test]
fn test_storage_save_load() {
    let storage = Storage::new(":memory:").unwrap(); // In-memory DB for test
    let game = Game::new();
    storage.save_game("test", &game, "Alice", "Bob").unwrap();
    let loaded = storage.load_game("test").unwrap();
    assert!(loaded.is_some());
    let (loaded_game, p1, p2) = loaded.unwrap();
    assert_eq!(loaded_game.black, game.black);
    assert_eq!(p1, "Alice");
    assert_eq!(p2, "Bob");
}

#[test]
fn test_storage_load_all() {
    let storage = Storage::new(":memory:").unwrap();
    let game = Game::new();
    storage.save_game("test1", &game, "Alice", "Bob").unwrap();
    let (games, players) = storage.load_all_games().unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(players.len(), 1);
}
