# 🔌 REST API

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
