document.getElementById('start-game').addEventListener('click', startGame);
document.getElementById('end-turn').addEventListener('click', endTurn)
document.getElementById('spectate-when-dead').addEventListener('click', spectateGame)
let deck = [];  // Deck will be fetched from the server
let gameData = {};  // Game data will be fetched from the server
var renderedData = {}
var gameStateInterval = 0;
var gameStateCooldown=1000;
var spectating = false;

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
        if (!gameData['players'][username] && !spectating) {
            document.getElementById('dead').style.display = 'block'
            document.getElementById('spectate-when-dead').style.display = 'block'
            document.getElementById('turn').style.display = 'none'
            document.getElementById('no-shooting').style.display = 'none'
            document.getElementById('landmine').style.display = 'none'
            const gameDiv = document.getElementById('game');
            gameDiv.innerHTML = '';  // Clear the game div
        } else {
            if (gameData.order.length == 1) {
                document.getElementById('win').style.display = "block"
                document.getElementById('win').innerHTML = gameData['order'][0] + " Wins"
                document.getElementById('turn').style.display = 'none'
                document.getElementById('end-turn').style.display = 'none'
                document.getElementById('no-shooting').style.display = 'none'
                document.getElementById('landmine').style.display = 'none'
                const gameDiv = document.getElementById('game');
                gameDiv.innerHTML = '';  // Clear the game div
            }
            else {
                document.getElementById('turn').innerHTML = gameData['order'][gameData['turn']] + "'s Turn"
                if (gameData['order'][gameData['turn']] == username) {
                    document.getElementById('end-turn').style.display = 'block'
                    document.getElementById('play-card').style.display = 'block'
                }
                else {
                    document.getElementById('end-turn').style.display = 'none'
                    document.getElementById('play-card').style.display = 'none'
                }
                if (!gameData['shooting_allowed']) {
                    document.getElementById('no-shooting').style.display = 'block'
                    document.getElementById('no-shooting').innerHTML = "No shooting was played by " + gameData['no_shooting_player']
                }
                else document.getElementById('no-shooting').style.display = 'none'
                if (gameData['landmine_in_play']) {
                    document.getElementById('landmine').style.display = 'block'
                }
                else document.getElementById('landmine').style.display = 'none'
                const gameDiv = document.getElementById('game');
                gameDiv.innerHTML = '';  // Clear the game div

                // Move the current player's data to the top of the player list for rendering
                var orderedPlayers
                if (!spectating) orderedPlayers = Object.assign({}, { [username]: gameData.players[username] }, gameData.players);
                else orderedPlayers = gameData.players

                // Render players' hands
                for (const player in orderedPlayers) {
                    const playerData = orderedPlayers[player];
                    const playerDiv = document.createElement('div');
                    playerDiv.classList.add('player');

                    const healthText = document.createElement('p');
                    healthText.innerText = `${player}'s Health: ${playerData.health}`;
                    playerDiv.appendChild(healthText);

                    if (playerData['queued_cards'].length > 0) {
                        const queuedCards = document.createElement('div')
                        queuedCards.classList.add('queuedCards')
                        for (var i = 0; i < playerData['queued_cards'].length; i++) {
                            var cardDisplay = document.createElement('p')
                            cardDisplay.innerText = `${playerData['queued_cards'][i]['name']} has a countdown of ${playerData['queued_cards'][i]['countdown']}`
                            queuedCards.appendChild(cardDisplay)
                            if (playerData['queued_cards'][i]['countdown'] == 0&&player==username&&gameData['order'][gameData['turn']]==username) {
                                var activateButton = document.createElement('button')
                                activateButton.classList.add('activateButton')
                                activateButton.innerHTML = 'Activate'
                                activateButton.id=playerData['queued_cards'][i]['cardid']
                                activateButton.addEventListener('click', (event) => { activateCalculatedShootingPopup(event.target.id) })
                                queuedCards.appendChild(activateButton)
                            }
                        }
                        playerDiv.appendChild(queuedCards)
                    }

                    const handDiv = document.createElement('div');
                    handDiv.classList.add('hand');

                    playerData.hand.forEach(cardIndex => {
                        const cardDiv = document.createElement('div');
                        cardDiv.classList.add('card');

                        const img = document.createElement('img');
                        if (player === username) {
                            img.src = `https://raw.githubusercontent.com/yzabeast1/tanks/refs/heads/master/server/${deck[cardIndex]['image-location']}`;  // Use card ID to get the front image
                            img.alt = deck[cardIndex].name;
                            cardDiv.classList.add('my-hand');
                        } else {
                            img.src = `https://raw.githubusercontent.com/yzabeast1/tanks/refs/heads/master/server/cards/back.png`;  // Use the back image for others
                            img.alt = 'Card Back';
                        }

                        cardDiv.appendChild(img);

                        // Add click event to zoom in on card
                        cardDiv.addEventListener('click', () => openModal(img.src, deck[cardIndex]));

                        handDiv.appendChild(cardDiv);
                    });

                    playerDiv.appendChild(handDiv);
                    gameDiv.appendChild(playerDiv);
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
    modalImage.src = imageSrc;
    modal.style.display = 'flex';
    if (card['type'] == 'shooting' && ((!renderedData['shooting_allowed'] && renderedData['no_shooting_player'] != username) || renderedData['shooting_count'] <= 0)) {
        document.getElementById('play-card').style.display = 'none'
    }
    else if (card['type'] == 'event' && renderedData['event_count'] <= 0) {
        document.getElementById('play-card').style.display = 'none'
    }
    else if (card['only_card'] && renderedData['card_played_this_turn']) {
        document.getElementById('play-card').style.display = 'none'
    }
    else document.getElementById('play-card').style.display = 'block'
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

    createPlayerSelectPopup();

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