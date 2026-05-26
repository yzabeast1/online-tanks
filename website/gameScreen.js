document.getElementById('start-game').addEventListener('click', startGame);
document.getElementById('end-turn').addEventListener('click', endTurn)
document.getElementById('spectate-when-dead').addEventListener('click', spectateGame)
let deck = [];  // Deck will be fetched from the server
let gameData = {};  // Game data will be fetched from the server
var renderedData = {}
var gameStateInterval = 0;
var gameStateCooldown=1000;
var spectating = false;

function cardImageFolder(cardType) {
    if (typeof cardType === 'string') {
        return cardType.toLowerCase();
    }

    if (cardType && typeof cardType === 'object' && cardType.Shooting) {
        return 'shooting';
    }

    return 'support';
}

function spectateGame() {
    document.getElementById('spectate-when-dead').style.display = 'none'
    spectating = true
}
// Fetch the deck from the server
async function fetchDeck() {
    try {
        const response = await fetch(`https://${serverip}/getDeck`);
        deck = await response.json();
    } catch (error) {
        console.error('Failed to fetch deck:', error);
    }
}

// Fetch the game state from the server every second
async function fetchGameState() {
    try {
        const response = await fetch(`https://${serverip}/gameState`, { headers: { joincode: joincode, username: username } });
        gameData = await response.json();
        renderGame();  // Render the game whenever the game state is updated
    } catch (error) {
        console.error('Failed to fetch game state:', error);
    }
}

// Render the game
function renderGame() {
    if (JSON.stringify(renderedData) != JSON.stringify(gameData)) {
        const myPlayerIndex = gameData.players.findIndex(p => p.name === username);
        const isDead = myPlayerIndex === -1;

        if (isDead && !spectating) {
            document.getElementById('dead').style.display = 'block'
            document.getElementById('spectate-when-dead').style.display = 'block'
            document.getElementById('turn').style.display = 'none'
            document.getElementById('no-shooting').style.display = 'none'
            document.getElementById('landmine').style.display = 'none'
            const gameDiv = document.getElementById('game');
            gameDiv.innerHTML = '';  // Clear the game div
        } else {
            if (gameData.players.length == 1) {
                document.getElementById('win').style.display = "block"
                document.getElementById('win').innerHTML = gameData.players[0].name + " Wins"
                document.getElementById('turn').style.display = 'none'
                document.getElementById('end-turn').style.display = 'none'
                document.getElementById('no-shooting').style.display = 'none'
                document.getElementById('landmine').style.display = 'none'
                const gameDiv = document.getElementById('game');
                gameDiv.innerHTML = '';  // Clear the game div
            }
            else {
                document.getElementById('turn').innerHTML = gameData.players[gameData.current_turn_player].name + "'s Turn"
                if (gameData.players[gameData.current_turn_player].name == username) {
                    document.getElementById('end-turn').style.display = 'block'
                    document.getElementById('play-card').style.display = 'block'
                }
                else {
                    document.getElementById('end-turn').style.display = 'none'
                    document.getElementById('play-card').style.display = 'none'
                }
                if (gameData.active_cards.no_shooting_played_by !== -1) {
                    document.getElementById('no-shooting').style.display = 'block'
                    document.getElementById('no-shooting').innerHTML = "No shooting was played by " + gameData.players[gameData.active_cards.no_shooting_played_by].name
                }
                else document.getElementById('no-shooting').style.display = 'none'
                if (gameData.active_cards.landmine_played_by !== -1) {
                    document.getElementById('landmine').style.display = 'block'
                }
                else document.getElementById('landmine').style.display = 'none'
                const gameDiv = document.getElementById('game');
                gameDiv.innerHTML = '';  // Clear the game div

                // Move the current player's data to the top of the player list for rendering
                var orderedPlayers = [];
                if (!spectating && !isDead) {
                    orderedPlayers.push(gameData.players[myPlayerIndex]);
                    orderedPlayers.push(...gameData.players.filter((p, i) => i !== myPlayerIndex));
                } else {
                    orderedPlayers = gameData.players;
                }

                const opponentGrid = document.createElement('div');
                opponentGrid.classList.add('opponent-grid');

                // Render players' hands
                for (let j = 0; j < orderedPlayers.length; j++) {
                    const playerData = orderedPlayers[j];
                    const isMyPlayer = playerData.name === username;
                    const playerDiv = document.createElement('div');
                    playerDiv.classList.add('player');
                    if (!isMyPlayer) {
                        playerDiv.classList.add('opponent-player');
                    }

                    const healthText = document.createElement('p');
                    healthText.innerText = `${playerData.name}'s Health: ${playerData.health}`;
                    playerDiv.appendChild(healthText);

                    let playerQueuedCards = gameData.active_cards.active_calculated_shootings.filter(s => s.owner === playerData.name);
                    if (playerQueuedCards.length > 0) {
                        const queuedCards = document.createElement('div')
                        queuedCards.classList.add('queuedCards')
                        for (var i = 0; i < playerQueuedCards.length; i++) {
                            const queuedCardData = playerQueuedCards[i];
                            var cardDisplay = document.createElement('p')
                            cardDisplay.innerText = `${queuedCardData.card_played.name} has a countdown of ${queuedCardData.turns_remaining}`
                            queuedCards.appendChild(cardDisplay)
                            if (queuedCardData.turns_remaining == 0 && playerData.name == username && gameData.players[gameData.current_turn_player].name == username) {
                                var activateButton = document.createElement('button')
                                activateButton.classList.add('activateButton')
                                activateButton.innerHTML = 'Activate'
                                activateButton.id = queuedCardData.card_played.id
                                activateButton.addEventListener('click', (event) => { activateCalculatedShootingPopup(event.target.id) })
                                queuedCards.appendChild(activateButton)
                            }
                        }
                        playerDiv.appendChild(queuedCards)
                    }

                    const handDiv = document.createElement('div');
                    handDiv.classList.add('hand');

                    if (!isMyPlayer) {
                        handDiv.classList.add('opponent-hand');
                        const cardDiv = document.createElement('div');
                        cardDiv.classList.add('card', 'opponent-card-count');

                        const img = document.createElement('img');
                        img.src = `https://${serverip}/images/back.png`;
                        img.alt = 'Card Back';

                        const count = document.createElement('span');
                        const handCount = playerData.hand_count || 0;
                        count.classList.add('card-count-badge');
                        count.innerText = handCount.toString();
                        count.setAttribute('aria-label', `${handCount} cards`);

                        cardDiv.appendChild(img);
                        cardDiv.appendChild(count);
                        handDiv.appendChild(cardDiv);
                    } else {
                        playerData.hand.forEach((card) => {
                            const cardDiv = document.createElement('div');
                            cardDiv.classList.add('card');

                            const img = document.createElement('img');
                            img.src = `https://${serverip}/images/${cardImageFolder(card.card_type)}/${card.id}.png`;  // Use card ID to get the front image
                            img.alt = card.name;
                            cardDiv.classList.add('my-hand');

                            cardDiv.appendChild(img);

                            // Add click event to zoom in on card
                            cardDiv.addEventListener('click', () => openModal(img.src, card));

                            handDiv.appendChild(cardDiv);
                        });
                    }

                    playerDiv.appendChild(handDiv);
                    if (isMyPlayer) {
                        gameDiv.appendChild(playerDiv);
                    } else {
                        opponentGrid.appendChild(playerDiv);
                    }
                }
                if (opponentGrid.children.length > 0) {
                    gameDiv.appendChild(opponentGrid);
                }
                renderedData = gameData
            }
        }
    }
}



// Open modal to zoom in on card
function openModal(imageSrc, card) {
    const modal = document.getElementById('modal');
    const modalImage = document.getElementById('modalImage');
    cardSelected = card.id;
    modalImage.src = imageSrc;
    modal.style.display = 'flex';

    let isShooting = typeof card.card_type === 'object' && card.card_type !== null && card.card_type.hasOwnProperty('Shooting');
    let isEvent = card.card_type === 'Event';
    
    let distractorMissilePlayed = renderedData.turn_state.distractor_missile_played || 0;
    let maxShooting = distractorMissilePlayed > 0
        ? renderedData.turn_state.more_ammo_played
        : 1 + renderedData.turn_state.more_ammo_played;
    let isDistractorMissile = card.id === 'distractor-missile';
    let shootingAllowed = renderedData.active_cards.no_shooting_played_by === -1 || renderedData.players[renderedData.active_cards.no_shooting_played_by].name === username;
    let isMyTurn = renderedData.players[renderedData.current_turn_player]?.name === username;
    let hasQueuedDistractorTarget = renderedData.active_cards.active_calculated_shootings.some(s => s.owner !== username);

    if (!isMyTurn) {
        document.getElementById('play-card').style.display = 'none';
    }
    else if (isShooting && cardNeedsTarget(card) && !hasValidTargetsForCard(card)) {
        document.getElementById('play-card').style.display = 'none';
    }
    else if (isDistractorMissile && (!shootingAllowed || !hasQueuedDistractorTarget || renderedData.turn_state.shooting_card_played > renderedData.turn_state.more_ammo_played)) {
        document.getElementById('play-card').style.display = 'none';
    }
    else if (isShooting && !isDistractorMissile && (!shootingAllowed || renderedData.turn_state.shooting_card_played >= maxShooting)) {
        document.getElementById('play-card').style.display = 'none';
    }
    else if (isEvent && renderedData.turn_state.event_card_played) {
        document.getElementById('play-card').style.display = 'none';
    }
    else if (['nuke', 'lottery'].includes(card.id) && renderedData.turn_state.total_cards_played > 0) {
        document.getElementById('play-card').style.display = 'none';
    }
    else {
        document.getElementById('play-card').style.display = 'block';
    }
}

// Close modal
function closeModal() {
    const modal = document.getElementById('modal');
    modal.style.display = 'none';
}

// Close modal on pressing 'Esc' key
document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') {
        closeModal();
    }
});
function startGame() {
    postWithFallbackNoJSON(`https://${serverip}/startGame`, { joincode: joincode })
    joinStartedGame();
}
function joinStartedGame() {
    document.querySelector('.lobby-screen').style.display = 'none'
    document.querySelector('.game-screen').style.display = 'block'
    fetchDeck();
    clearIntervals()
    chatInterval=setInterval(function() {fetchChatMessages(joincode)}, chatCooldown);
    gameStateInterval = setInterval(fetchGameState, gameStateCooldown);
}
function endTurn() {
    postWithFallbackNoJSON(`https://${serverip}/endTurn`, { joincode: joincode, username: username })
}

const dragBar = document.getElementById('dragBar');
const leftPane = document.getElementById('leftPane');
const rightPane = document.getElementById('rightPane');

// Variable to track whether the user is dragging the bar
let isDragging = false;

// Mouse down event on the drag bar
dragBar.addEventListener('mousedown', (e) => {
    isDragging = true;
    document.body.style.cursor = 'col-resize';  // Change cursor while dragging
    document.body.style.userSelect = 'none';   // Disable text selection while dragging
});

// Mouse move event on the document (tracking drag)
document.addEventListener('mousemove', (e) => {
    if (!isDragging) return;

    // Calculate new width for left pane based on mouse position
    const newLeftPaneWidth = e.clientX;

    // Update the width of the left pane and right pane
    leftPane.style.width = `${newLeftPaneWidth}px`;
    rightPane.style.width = `calc(100% - ${newLeftPaneWidth + dragBar.offsetWidth}px)`;
});

// Mouse up event to stop dragging
document.addEventListener('mouseup', () => {
    isDragging = false;
    document.body.style.cursor = 'default';  // Reset cursor
    document.body.style.userSelect = '';     // Re-enable text selection after dragging
});

function clearIntervals() {
    // Get a reference to the last interval + 1
    const interval_id = window.setInterval(function () { }, Number.MAX_SAFE_INTEGER);

    // Clear any timeout/interval up to that id
    for (let i = 1; i < interval_id; i++) {
        window.clearInterval(i);
    }
}
function activateCalculatedShootingPopup(cardid) {
    cardSelected = cardid
    document.getElementById('playCardBtn').onclick = activateCalculatedShooting
    const popupContent = document.getElementById('popupContent');
    popupContent.innerHTML = '';  // Clear previous popup content

    const card = deck.find(item => item.id === cardid);
    createPlayerSelectPopup(card);
    document.getElementById('playCardBtn').style.display = hasValidTargetsForCard(card) ? 'block' : 'none';

    showPopup();  // Show the popup after generating content

}
function activateCalculatedShooting() {
    const selectedPlayer = document.getElementById('playerDropdown')?.value;
    var cardid = cardSelected
    var headers = {
        joincode: joincode,
        username: username,
        cardid: cardid,
        target: selectedPlayer
    }
    postWithFallback(`https://${serverip}/activateCalculatedShooting`, headers)
    closePopup();  // Close the popup after playing the card
}
