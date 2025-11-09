document.addEventListener('DOMContentLoaded', () => {
    const boardElement = document.getElementById('game-board');
    const scoreBlackElement = document.getElementById('score-black');
    const scoreWhiteElement = document.getElementById('score-white');
    const playerBlackElement = document.getElementById('player-black');
    const playerWhiteElement = document.getElementById('player-white');
    const newGameBtn = document.getElementById('new-game-btn');
    const loginBtn = document.getElementById('login-btn');
    const submitLoginBtn = document.getElementById('submit-login-btn');
    const loginModal = document.getElementById('login-modal');
    const playerNameInput = document.getElementById('player-name'); // Added for focus
    const gameOverModal = document.getElementById('game-over-modal');
    const gameOverMessage = document.getElementById('game-over-message');
    const playAgainBtn = document.getElementById('play-again-btn');
    const gameStatus = document.getElementById('game-status');
    const logoutBtn = document.getElementById('logout-btn');
    const passMoveModal = document.getElementById('pass-move-modal'); // New modal for pass
    const passMoveBtn = document.getElementById('pass-move-btn'); // New button for pass

    let token = null;
    let currentGameId = null;
    let ws = null;
    let loggedInPlayerName = ''; // To store the logged-in player's name

    // --- Event Listeners ---
    newGameBtn.addEventListener('click', createMatch);
    loginBtn.addEventListener('click', () => {
        loginModal.classList.remove('hidden');
        playerNameInput.focus(); // Set focus on textbox
    });
    playerNameInput.addEventListener('keydown', (event) => {
        if (event.key === 'Enter') {
            submitLoginBtn.click();
        }
    });
    submitLoginBtn.addEventListener('click', login);
    logoutBtn.addEventListener('click', logout);
    playAgainBtn.addEventListener('click', () => {
        gameOverModal.classList.add('hidden');
        createMatch();
    });
    passMoveBtn.addEventListener('click', () => {
        passMoveModal.classList.add('hidden');
        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({type: "pass"}));
        }
    });

    // --- API & WebSocket Functions ---
    async function login() {
        const playerName = playerNameInput.value;
        if (!playerName) return;

        try {
            const response = await fetch('/auth/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ player: playerName })
            });

            if (response.ok) {
                const data = await response.json();
                token = data.token;
                loggedInPlayerName = playerName; // Store player name
                loginModal.classList.add('hidden');
                loginBtn.textContent = `Logged in as ${playerName}`;
                loginBtn.disabled = true;
                logoutBtn.style.display = 'inline-block';
            } else {
                alert('Login failed');
            }
        } catch (error) {
            console.error('Login error:', error);
            alert('Login failed due to a network error.');
        }
    }

    function logout() {
        token = null;
        currentGameId = null;
        loggedInPlayerName = '';
        if (ws) {
            ws.close();
            ws = null;
        }
        loginBtn.textContent = 'Login';
        loginBtn.disabled = false;
        logoutBtn.style.display = 'none';
        gameStatus.textContent = '';
        // Clear board or reset UI
        boardElement.innerHTML = '';
        scoreBlackElement.textContent = '2';
        scoreWhiteElement.textContent = '2';
        playerBlackElement.textContent = 'Black';
        playerWhiteElement.textContent = 'White';
        playerBlackElement.classList.remove('active');
        playerWhiteElement.classList.remove('active');
    }

    async function createMatch() {
        if (!token) {
            alert('You must be logged in to start a new game.');
            loginModal.classList.remove('hidden');
            playerNameInput.focus(); // Set focus on textbox
            return;
        }

        try {
            const response = await fetch('/match/new', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${token}`
                },
                body: JSON.stringify({ player2: 'AI' }) // Or implement player selection
            });

            if (response.ok) {
                const data = await response.json();
                currentGameId = data.id;
                gameStatus.textContent = '';
                startGame();
            } else {
                alert('Failed to create match');
            }
        } catch (error) {
            console.error('Create match error:', error);
            alert('Failed to create match due to a network error.');
        }
    }

    function startGame() {
        connectWebSocket();
    }

    function connectWebSocket() {
        if (ws) {
            ws.close();
        }
        
        const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        ws = new WebSocket(`${wsProtocol}//${window.location.host}/match/${currentGameId}/ws`);

        ws.onopen = () => {
            console.log('WebSocket connection established.');
        };
        
        ws.onmessage = (event) => {
            console.log("Received message from server:", event.data);
            const message = JSON.parse(event.data);
            if (message.type === 'status') {
                gameStatus.textContent = message.message;
            } else {
                updateUI(message);
            }
        };

        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };

        ws.onclose = () => {
            console.log('WebSocket connection closed.');
        };
    }

    async function makeMove(row, col) {
        console.log(`makeMove called for row: ${row}, col: ${col}`);
        if (!ws || ws.readyState !== WebSocket.OPEN) {
            console.error("WebSocket is not connected.");
            return;
        }

        const coord = String.fromCharCode(65 + col) + (8 - row);
        const moveMessage = {
            type: 'move',
            coord: coord,
        };
        
        console.log("Sending move message:", moveMessage);
        ws.send(JSON.stringify(moveMessage));
    }

    // --- UI Update Functions ---
    function updateUI(state) {
        console.log("Updating UI with new state:", state);
        renderBoard(state.board, state.legal_moves);
        updateScores(state.scores);
        updateTurnIndicator(state.current_player, state.player1, state.player2);

        if (state.game_over) {
            showGameOver(state.winner);
            gameStatus.textContent = '';
        } else if (state.legal_moves.length === 0 && 
                   ((state.current_player === 'Black' && state.player1 === loggedInPlayerName) ||
                    (state.current_player === 'White' && state.player2 === loggedInPlayerName))) {
            // Human has no moves, show pass dialog
            passMoveModal.classList.remove('hidden');
        } else {
            gameStatus.textContent = '';
        }
    }

    function renderBoard(board, legalMoves) {
        console.log("Rendering board. Legal moves:", legalMoves);
        boardElement.innerHTML = '';
        const legalMoveSet = new Set(legalMoves);
        console.log("Legal moves set:", legalMoveSet);

        for (let i = 0; i < 8; i++) {
            for (let j = 0; j < 8; j++) {
                const cell = document.createElement('div');
                cell.className = 'cell';
                cell.dataset.row = i;
                cell.dataset.col = j;

                const piece = board[i][j];
                if (piece === 'B' || piece === 'W') {
                    const pieceElement = document.createElement('div');
                    pieceElement.className = `piece ${piece === 'B' ? 'black' : 'white'}`;
                    cell.appendChild(pieceElement);
                }

                const moveCoord = String.fromCharCode(65 + j) + (8 - i);
                if (legalMoveSet.has(moveCoord)) {
                    console.log(`Attaching click listener to cell ${i}, ${j} (${moveCoord})`);
                    const legalIndicator = document.createElement('div');
                    legalIndicator.className = 'legal-move-indicator';
                    cell.appendChild(legalIndicator);
                    cell.addEventListener('click', () => makeMove(i, j));
                }
                
                boardElement.appendChild(cell);
            }
        }
    }

    function updateScores(scores) {
        scoreBlackElement.textContent = scores.B;
        scoreWhiteElement.textContent = scores.W;
    }

    function updateTurnIndicator(currentPlayer, player1Name, player2Name) {
        playerBlackElement.classList.toggle('active', currentPlayer === 'Black');
        playerWhiteElement.classList.toggle('active', currentPlayer === 'White');

        // Update player names displayed
        playerBlackElement.textContent = player1Name;
        playerWhiteElement.textContent = player2Name;
    }

    function showGameOver(winner) {
        let message = "It's a draw!";
        if (winner) {
            message = `${winner} wins!`;
        }
        gameOverMessage.textContent = message;
        gameOverModal.classList.remove('hidden');
    }

    // --- Initial Load ---
    function initialize() {
        // On load, we can just show the login button and wait for user action.
        // Or attempt to validate an existing token from localStorage.
        console.log('Othello UI Initialized.');
    }

    initialize();
});
