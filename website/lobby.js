document.getElementById('new-game').addEventListener('click', newGame);
document.getElementById('join-lobby').addEventListener('click', joinLobby);
document.getElementById('show-deck-checkbox').addEventListener('click', toggleDeck)
document.getElementById('leave-lobby').addEventListener('click', leaveLobby);
document.getElementById('spectate').addEventListener('click', startSpectating)
var playersInLobbyCooldown = 1000;
var lobbyPlayersInterval = 0
var lobbyStartedCheckCoooldown = 1000
var lobbyStartedCheckInterval = 0;
var serverip = 'tanks.hosted.internal:8443'
function startSpectating() {
    spectating = true
    username = document.getElementById('username-input').value;
    joincode = document.getElementById('joincode-input').value;
    if (joincode.trim() === '' || username.trim() === '') {
        alert('Please enter both a join code and username');
        return;
    }
    if (username == 'server') {
        alert('Invalid username')
        return
    }
    document.querySelector('.menu-screen').style.display = 'none'
    document.querySelector('.lobby-screen').style.display = 'block'
    document.querySelector('.container').style.display = 'flex'
    document.querySelector('.start-game').style.display = 'none'
    document.getElementById('lobby-show-code').innerHTML = "JoinCode: " + joincode
    lobbyPlayersInterval = setInterval(lobbyPlayers, playersInLobbyCooldown);
    lobbyStartedCheckInterval = setInterval(lobbyStartedCheck, lobbyStartedCheckCoooldown)
    lobbyPlayers();
    startChat();
}
function joinLobby() {
    username = document.getElementById('username-input').value;
    joincode = document.getElementById('joincode-input').value;
    if (joincode.trim() === '' || username.trim() === '') {
        alert('Please enter both a join code and username');
        return;
    }
    if (username == 'server') {
        alert('Invalid username')
        return
    }
    document.querySelector('.menu-screen').style.display = 'none'
    document.querySelector('.lobby-screen').style.display = 'block'
    document.querySelector('.container').style.display = 'flex'
    document.querySelector('.start-game').style.display = 'none'
    const headers = {
        'Content-Type': 'application/json',
        'username': username,
        'joincode': joincode
    };
    fetch(`https://${serverip}/joinGame`, {
        method: 'POST',
        headers: headers
    })
        .then(response => {
            if (!response.ok) throw new Error('HTTPS failed'); // Handle HTTP errors
            document.getElementById('lobby-show-code').innerHTML = "JoinCode: " + joincode
        })
        .catch(error => {
            console.error('HTTPS error:', error);
        });
    lobbyPlayersInterval = setInterval(lobbyPlayers, playersInLobbyCooldown);
    lobbyStartedCheckInterval = setInterval(lobbyStartedCheck, lobbyStartedCheckCoooldown)
    lobbyPlayers();
    startChat();
}
function newGame() {
    username = document.getElementById('username-input').value;
    if (username.trim() === '') {
        alert('Please enter a username');
        return;
    }
    document.querySelector('.menu-screen').style.display = 'none'
    document.querySelector('.lobby-screen').style.display = 'block'
    document.querySelector('.container').style.display = 'flex'
    const headers = {
        'Content-Type': 'application/json',
        'username': username
    };

    // Try sending the message using HTTPS
    fetch(`https://${serverip}/createGame`, {
        method: 'POST',
        headers: headers
    })
        .then(response => {
            if (!response.ok) throw new Error('HTTPS failed'); // Handle HTTP errors
            joincode = response.headers.get('joincode')
            document.getElementById('joincode-input').value = joincode
            document.getElementById('lobby-show-code').innerHTML = "JoinCode: " + document.getElementById('joincode-input').value
            lobbyPlayersInterval = setInterval(lobbyPlayers, playersInLobbyCooldown);
            addLobbyPlayer(username, 'server-messsage')
            lobbyPlayers();
            startChat();
        })
        .catch(error => {
            console.error('HTTPS error:', error);
        });
}
function lobbyPlayers() {
    const headers = { 'joincode': joincode }; // Add joincode header
    fetchWithFallback(`https://${serverip}/lobbyState`, headers)
        .then(data => {
            if (data && data.length > 0) {
                clearLobbyPlayers();
                data.forEach(player => {
                    addLobbyPlayer(player, 'server-message');
                });
            } else {
                console.warn('No messages found for this joincode.');
            }
        })
        .catch(error => {
            console.error('Error fetching lobby players:', error);
        });
}
function clearLobbyPlayers() {
    const players = document.getElementById('lobby-players');
    players.innerHTML = ''; // Clear all messages
}
function addLobbyPlayer(player, className) {
    const messageElement = document.createElement('div');
    messageElement.className = className;
    messageElement.innerText = player;

    const playerList = document.getElementById('lobby-players');
    playerList.appendChild(messageElement);
}
function leaveLobby() {
    username = document.getElementById('username-input').value;
    joincode = document.getElementById('joincode-input').value;
    const headers = {
        'Content-Type': 'application/json',
        'username': username,
        'joincode': joincode
    };
    fetch(`https://${serverip}/leaveLobby`, {
        method: 'POST',
        headers: headers
    })
        .then(response => {
            if (!response.ok) throw new Error('HTTPS failed'); // Handle HTTP errors
            else {
                document.querySelector('.menu-screen').style.display = 'block'
                document.querySelector('.lobby-screen').style.display = 'none'
                document.getElementById('joincode-input').value = ''
                clearInterval(lobbyPlayersInterval)
            }
        })
        .catch(error => {
            console.error('HTTPS error:', error);
        });
}
function toggleDeck() {
    var deck = document.getElementById('show-deck')
    if (deck.style.display == 'block') deck.style.display = 'none'
    else deck.style.display = 'block'
}
window.onload = createDeck
function createDeck() {
    const deckDiv = document.getElementById('show-deck');
    fetchWithFallback("https://" + serverip + "/getDeck").then(deck => {
        deck.forEach(card => {
            // Create an image element
            const img = document.createElement('img');
            img.src = `https://raw.githubusercontent.com/yzabeast1/online-tanks/refs/heads/master/images/${card['card_type'].toLowerCase()}/${card['id']}.png`; // Set the image source to the card's image location
            img.alt = card.name; // Set the alt text to the card's name
            img.style.width = '150px'; // Optional: set the image size
            img.style.margin = '10px'; // Optional: add some margin between images

            // Append the image to the deckDiv
            deckDiv.appendChild(img);
        });
    })
}
function addCardToDeck() {
    const image = document.createElement('image')
    image.className = "deck-image"
    const deck = document.getElementById('show-deck')
    deck.appendChild(image)
}
function lobbyStartedCheck() {
    fetch(`https://${serverip}/checkGameStarted`, {
        method: 'GET',
        headers: {
            'joincode': joincode
        }
    })
        .then(response => response.text())  // Expect a simple "yes" or "no"
        .then(data => {
            if (data === "yes") {
                joinStartedGame();
            } else {
                console.log("Game has not started, waiting...");
            }
        })
        .catch(error => {
            console.error("Error checking game status:", error);
        });
}