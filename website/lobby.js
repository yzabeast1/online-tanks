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

const defaultLobbySettings = {
    starting_hand_size: 4,
    starting_health: 10,
    max_health: 10,
    play_heal_on_others: false,
    revive_others_with_heal: false,
};

function getLobbySettingsValue(id, fallback) {
    const element = document.getElementById(id);
    if (!element) {
        return fallback;
    }

    if (element.type === 'checkbox') {
        return element.checked;
    }

    const value = Number.parseInt(element.value, 10);
    return Number.isFinite(value) ? value : fallback;
}

function readLobbySettings() {
    const playHealOnOthers = getLobbySettingsValue('lobby-play-heal-on-others', defaultLobbySettings.play_heal_on_others);
    return {
        starting_hand_size: getLobbySettingsValue('lobby-starting-hand-size', defaultLobbySettings.starting_hand_size),
        starting_health: getLobbySettingsValue('lobby-starting-health', defaultLobbySettings.starting_health),
        max_health: getLobbySettingsValue('lobby-max-health', defaultLobbySettings.max_health),
        play_heal_on_others: playHealOnOthers,
        revive_others_with_heal: playHealOnOthers && getLobbySettingsValue('lobby-revive-others-with-heal', defaultLobbySettings.revive_others_with_heal),
    };
}

function syncLobbySettingsMenuFromDefaults() {
    const handSize = document.getElementById('lobby-starting-hand-size');
    const startingHealth = document.getElementById('lobby-starting-health');
    const maxHealth = document.getElementById('lobby-max-health');
    const playHealOnOthers = document.getElementById('lobby-play-heal-on-others');
    const reviveOthersWithHeal = document.getElementById('lobby-revive-others-with-heal');

    if (handSize) handSize.value = defaultLobbySettings.starting_hand_size;
    if (startingHealth) startingHealth.value = defaultLobbySettings.starting_health;
    if (maxHealth) maxHealth.value = defaultLobbySettings.max_health;
    if (playHealOnOthers) playHealOnOthers.checked = defaultLobbySettings.play_heal_on_others;
    if (reviveOthersWithHeal) reviveOthersWithHeal.checked = defaultLobbySettings.revive_others_with_heal;

    syncLobbySettingsDependency();
}

function syncLobbySettingsDependency() {
    const playHealOnOthers = document.getElementById('lobby-play-heal-on-others');
    const reviveOthersWithHeal = document.getElementById('lobby-revive-others-with-heal');

    if (!playHealOnOthers || !reviveOthersWithHeal) {
        return;
    }

    // Hide the revive option entirely unless healing others is enabled.
    const reviveContainer = reviveOthersWithHeal.parentElement;
    if (reviveContainer) {
        reviveContainer.style.display = playHealOnOthers.checked ? '' : 'none';
    }

    // Keep the checkbox cleared when not applicable.
    if (!playHealOnOthers.checked) {
        reviveOthersWithHeal.checked = false;
    }
}

function renderLobbySettingsMenu(isVisible) {
    const settingsEl = document.getElementById('lobby-settings-menu');
    if (!settingsEl) {
        return;
    }

    if (!isVisible) {
        settingsEl.style.display = 'none';
        return;
    }

    settingsEl.style.display = 'block';
    if (settingsEl.innerHTML.trim() !== '') {
        return;
    }

    settingsEl.innerHTML = `
        <details>
            <summary>Game Settings</summary>
            <form id="lobby-settings-form" class="lobby-settings-form">
                <label for="lobby-starting-hand-size">Starting hand size</label>
                <input type="number" id="lobby-starting-hand-size" min="1" step="1" value="${defaultLobbySettings.starting_hand_size}">
                <label for="lobby-starting-health">Starting health</label>
                <input type="number" id="lobby-starting-health" min="1" step="1" value="${defaultLobbySettings.starting_health}">
                <label for="lobby-max-health">Max health</label>
                <input type="number" id="lobby-max-health" min="1" step="1" value="${defaultLobbySettings.max_health}">
                <label class="lobby-settings-check">
                    <input type="checkbox" id="lobby-play-heal-on-others">
                    <span>Allow heal cards to affect other players</span>
                </label>
                <label class="lobby-settings-check">
                    <input type="checkbox" id="lobby-revive-others-with-heal">
                    <span>Allow healing to revive other players</span>
                </label>
                <div id="lobby-settings-status" class="lobby-settings-status"></div>
            </form>
        </details>
    `;

    syncLobbySettingsMenuFromDefaults();

    const playHealOnOthers = document.getElementById('lobby-play-heal-on-others');
    const reviveOthersWithHeal = document.getElementById('lobby-revive-others-with-heal');
    if (playHealOnOthers) {
        playHealOnOthers.addEventListener('change', syncLobbySettingsDependency);
    }
    if (reviveOthersWithHeal) {
        reviveOthersWithHeal.addEventListener('change', syncLobbySettingsDependency);
    }
}

function updateLobbySettingsVisibility(isVisible) {
    renderLobbySettingsMenu(isVisible);
}

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
    updateLobbySettingsVisibility(false);
    document.getElementById('lobby-show-code').innerText = "JoinCode: " + joincode
    lobbyPlayersInterval = setInterval(lobbyPlayers, playersInLobbyCooldown);
    lobbyStartedCheckInterval = setInterval(lobbyStartedCheck, lobbyStartedCheckCoooldown)
    lobbyPlayers();
    startChat();
}
async function joinLobby() {
    spectating = false
    const account = clerkAccountDetails();
    if (account) {
        clerkAccount = account;
        applyClerkDefaults(account);
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
    updateLobbySettingsVisibility(false);
    const headers = {
        'Content-Type': 'application/json',
        'username': username,
        'joincode': joincode,
        ...clerkHeaders(account)
    };
    fetch(`https://${serverip}/joinGame`, {
        method: 'POST',
        headers: headers
    })
        .then(response => {
            if (!response.ok) throw new Error('HTTPS failed'); // Handle HTTP errors
            document.getElementById('lobby-show-code').innerText = "JoinCode: " + joincode
            loadMyClerkGames();
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
    const account = clerkAccountDetails();
    if (account) {
        clerkAccount = account;
        applyClerkDefaults(account);
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
    updateLobbySettingsVisibility(true);
    const headers = {
        'Content-Type': 'application/json',
        'username': username,
        ...clerkHeaders(account)
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
            document.getElementById('lobby-show-code').innerText = "JoinCode: " + document.getElementById('joincode-input').value
            lobbyPlayersInterval = setInterval(lobbyPlayers, playersInLobbyCooldown);
            addLobbyPlayer(username, 'server-messsage')
            loadMyClerkGames();
            lobbyPlayers();
            startChat();
        })
        .catch(error => {
            console.error('HTTPS error:', error);
        });
}
function lobbyPlayers() {
    const account = clerkAccountDetails();
    const headers = { 'joincode': joincode }; // Add joincode header
    fetch(`https://${serverip}/lobbyState`, { headers })
        .then(response => {
            if (!response.ok) throw new Error('HTTPS failed');
            const hostUsername = response.headers.get('host-username');
            return response.json().then(data => ({ data, hostUsername }));
        })
        .then(({ data, hostUsername }) => {
            updateLobbySettingsVisibility((hostUsername || '') === username && !spectating);
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
                updateLobbySettingsVisibility(false);
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
    const account = clerkAccountDetails();
    username = document.getElementById('username-input').value;
    joincode = document.getElementById('joincode-input').value;
    const headers = {
        'Content-Type': 'application/json',
        'username': username,
        'joincode': joincode,
        ...clerkHeaders(account)
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
                updateLobbySettingsVisibility(false);
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
