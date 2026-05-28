document.getElementById('new-game').addEventListener('click', newGame);
document.getElementById('join-lobby').addEventListener('click', joinLobby);
document.getElementById('show-deck-checkbox').addEventListener('click', toggleDeck)
document.getElementById('leave-lobby').addEventListener('click', leaveLobby);
document.getElementById('spectate').addEventListener('click', startSpectating)
var playersInLobbyCooldown = 1000;
var lobbyPlayersInterval = 0
var lobbyStartedCheckCoooldown = 1000
var lobbyStartedCheckInterval = 0;
var serverip = 'yzabeast1.run.place:444'
function setStartGameVisible(isVisible) {
    document.querySelector('.start-game').style.display = isVisible ? 'inline-block' : 'none';
}
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
    setStartGameVisible(false)
    document.getElementById('lobby-show-code').innerHTML = "JoinCode: " + joincode
    lobbyPlayersInterval = setInterval(lobbyPlayers, playersInLobbyCooldown);
    lobbyStartedCheckInterval = setInterval(lobbyStartedCheck, lobbyStartedCheckCoooldown)
    lobbyPlayers();
    startChat();
}
async function joinLobby() {
    spectating = false
    const account = shooAccountDetails();
    if (account) {
        shooAccount = account;
        await ensureShooSettingsLoaded(account);
        applyShooDefaults(account);
        updateShooUi();
    }
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
    setStartGameVisible(false)
    const headers = {
        'Content-Type': 'application/json',
        'username': username,
        'joincode': joincode,
        ...shooHeaders(account)
    };
    fetch(`https://${serverip}/joinGame`, {
        method: 'POST',
        headers: headers
    })
        .then(response => {
            if (!response.ok) throw new Error('HTTPS failed'); // Handle HTTP errors
            if (account) logShooAccountDetails('joined game', account);
            document.getElementById('lobby-show-code').innerHTML = "JoinCode: " + joincode
            loadMyShooGames();
        })
        .catch(error => {
            console.error('HTTPS error:', error);
        });
    lobbyPlayersInterval = setInterval(lobbyPlayers, playersInLobbyCooldown);
    lobbyStartedCheckInterval = setInterval(lobbyStartedCheck, lobbyStartedCheckCoooldown)
    lobbyPlayers();
    startChat();
}
async function newGame() {
    spectating = false
    const account = shooAccountDetails();
    if (account) {
        shooAccount = account;
        await ensureShooSettingsLoaded(account);
        applyShooDefaults(account);
        updateShooUi();
    }
    username = document.getElementById('username-input').value;
    if (username.trim() === '') {
        alert('Please enter a username');
        return;
    }
    document.querySelector('.menu-screen').style.display = 'none'
    document.querySelector('.lobby-screen').style.display = 'block'
    document.querySelector('.container').style.display = 'flex'
    setStartGameVisible(true)
    const headers = {
        'Content-Type': 'application/json',
        'username': username,
        ...shooHeaders(account)
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
            if (account) logShooAccountDetails('created game', account);
            loadMyShooGames();
            lobbyPlayers();
            startChat();
        })
        .catch(error => {
            console.error('HTTPS error:', error);
        });
}
function lobbyPlayers() {
    const account = shooAccountDetails();
    const headers = { 'joincode': joincode }; // Add joincode header
    fetch(`https://${serverip}/lobbyState`, { headers })
        .then(response => {
            if (!response.ok) throw new Error('HTTPS failed');
            const hostUsername = response.headers.get('host-username');
            return response.json().then(data => ({ data, hostUsername }));
        })
        .then(({ data, hostUsername }) => {
            if (data && data.length > 0) {
                clearLobbyPlayers();
                data.forEach(player => {
                    addLobbyPlayer(player, 'server-message');
                });
                const fallbackHost = typeof data[0] === 'string' ? data[0] : data[0].name;
                const isHostUsername = (hostUsername || fallbackHost) === username;
                setStartGameVisible(isHostUsername && data.length >= 2 && !spectating);
            } else {
                console.warn('No messages found for this joincode.');
                setStartGameVisible(false);
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
function lobbyPlayerInitials(playerName) {
    return (playerName || '?')
        .trim()
        .split(/\s+/)
        .slice(0, 2)
        .map(part => part[0])
        .join('')
        .toUpperCase() || '?';
}
function addLobbyPlayer(player, className) {
    const messageElement = document.createElement('div');
    messageElement.className = `${className} lobby-player`;
    const playerName = typeof player === 'string' ? player : player.name;
    const playerPicture = typeof player === 'string' ? '' : player.picture;

    if (playerPicture) {
        const image = document.createElement('img');
        image.className = 'player-picture';
        image.src = playerPicture;
        image.alt = `${playerName}'s profile picture`;
        messageElement.appendChild(image);
    } else {
        const defaultPicture = document.createElement('div');
        defaultPicture.className = 'player-picture default-player-picture';
        defaultPicture.innerText = lobbyPlayerInitials(playerName);
        defaultPicture.setAttribute('aria-label', `${playerName}'s default profile picture`);
        messageElement.appendChild(defaultPicture);
    }

    const name = document.createElement('span');
    name.innerText = playerName;
    messageElement.appendChild(name);

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
                document.querySelector('.menu-screen').style.display = 'flex'
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
function cardImageFolder(cardType) {
    if (typeof cardType === 'string') {
        return cardType.toLowerCase();
    }

    if (cardType && typeof cardType === 'object' && cardType.Shooting) {
        return 'shooting';
    }

    return 'support';
}

function createDeck() {
    const deckDiv = document.getElementById('show-deck');
    fetchWithFallback("https://" + serverip + "/getDeck").then(deck => {
        deck.forEach(card => {
            // Create an image element
            const img = document.createElement('img');
            img.src = `https://${serverip}/images/${cardImageFolder(card['card_type'])}/${card['id']}.png`; // Set the image source to the card's image location
            img.alt = card.name; // Set the alt text to the card's name
            img.style.width = '150px'; // Optional: set the image size
            img.style.margin = '10px'; // Optional: add some margin between images
            img.addEventListener('click', () => openDeckCardModal(img.src));

            // Append the image to the deckDiv
            deckDiv.appendChild(img);
        });
    })
}
function openDeckCardModal(imageSrc) {
    const modal = document.getElementById('modal');
    const modalImage = document.getElementById('modalImage');
    const playCardButton = document.getElementById('play-card');

    document.body.appendChild(modal);
    modalImage.src = imageSrc;
    playCardButton.style.display = 'none';
    modal.style.display = 'flex';
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
