document.addEventListener('DOMContentLoaded', () => {
    let token = null;
    let currentGameId = null;
    let ws = null;

    document.getElementById('loginBtn').addEventListener('click', login);
    document.getElementById('createMatchBtn').addEventListener('click', createMatch);
    document.getElementById('joinMatchmakingBtn').addEventListener('click', joinMatchmaking);
    document.getElementById('leaderboardBtn').addEventListener('click', showLeaderboard);
    document.getElementById('backToMenuBtn').addEventListener('click', showMenu);

    async function login() {
        const playerName = document.getElementById('playerName').value;
        if (!playerName) return;

        const response = await fetch('/auth/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ player: playerName })
        });

        if (response.ok) {
            const data = await response.json();
            token = data.token;
            showMenu();
        } else {
            alert('Login failed');
        }
    }

    function showMenu() {
        document.getElementById('login').classList.add('hidden');
        document.getElementById('menu').classList.remove('hidden');
        document.getElementById('game').classList.add('hidden');
        document.getElementById('leaderboard').classList.add('hidden');
    }

    async function createMatch() {
        const response = await fetch('/match/new', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({ player2: 'AI' })
        });

        if (response.ok) {
            const data = await response.json();
            currentGameId = data.id;
            startGame();
        } else {
            alert('Failed to create match');
        }
    }

    async function joinMatchmaking() {
        const response = await fetch('/match/join', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({})
        });

        if (response.ok) {
            const data = await response.json();
            if (data.matched) {
                currentGameId = data.id;
                startGame();
            } else {
                alert('Waiting for opponent...');
                // In a real app, poll or use WS for matchmaking status
            }
        } else {
            alert('Failed to join matchmaking');
        }
    }

    function startGame() {
        document.getElementById('menu').classList.add('hidden');
        document.getElementById('game').classList.remove('hidden');
        connectWebSocket();
        updateGameState();
    }

    function connectWebSocket() {
        ws = new WebSocket(`ws://localhost:8080/match/${currentGameId}/ws`);
        ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            renderBoard(data.board);
            updateStatus(data);
        };
    }

    async function updateGameState() {
        const response = await fetch(`/match/${currentGameId}/state`);
        if (response.ok) {
            const data = await response.json();
            renderBoard(data.board);
            updateStatus(data);
        }
    }

    function renderBoard(board) {
        const boardDiv = document.getElementById('board');
        boardDiv.innerHTML = '';
        board.forEach((row, i) => {
            row.forEach((cell, j) => {
                const cellDiv = document.createElement('div');
                cellDiv.className = 'cell';
                if (cell === 'B') cellDiv.classList.add('black');
                if (cell === 'W') cellDiv.classList.add('white');
                cellDiv.addEventListener('click', () => makeMove(i, j));
                boardDiv.appendChild(cellDiv);
            });
        });
    }

    function updateStatus(data) {
        const statusDiv = document.getElementById('status');
        statusDiv.textContent = `Current player: ${data.current_player}, Game over: ${data.game_over}`;
        if (data.winner) {
            statusDiv.textContent += `, Winner: ${data.winner}`;
        }
    }

    async function makeMove(row, col) {
        const coord = String.fromCharCode(97 + col) + (8 - row);
        const response = await fetch(`/match/${currentGameId}/move`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({ coord })
        });

        if (!response.ok) {
            alert('Invalid move');
        }
    }

    async function showLeaderboard() {
        const response = await fetch('/leaderboard');
        if (response.ok) {
            const data = await response.json();
            const content = document.getElementById('leaderboardContent');
            content.innerHTML = data.map(p => `<p>${p.name}: ${p.elo} (${p.wins}-${p.losses})</p>`).join('');
            document.getElementById('menu').classList.add('hidden');
            document.getElementById('leaderboard').classList.remove('hidden');
        }
    }
});