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

function playerInitials(playerName) {
    return (playerName || '?')
        .trim()
        .split(/\s+/)
        .slice(0, 2)
        .map(part => part[0])
        .join('')
        .toUpperCase() || '?';
}

function createPlayerPicture(playerData) {
    if (playerData.picture) {
        const playerPicture = document.createElement('img');
        playerPicture.classList.add('player-picture');
        playerPicture.src = playerData.picture;
        playerPicture.alt = `${playerData.name}'s profile picture`;
        return playerPicture;
    }

    const defaultPicture = document.createElement('div');
    defaultPicture.classList.add('player-picture', 'default-player-picture');
    defaultPicture.innerText = playerInitials(playerData.name);
    defaultPicture.setAttribute('aria-label', `${playerData.name}'s default profile picture`);
    return defaultPicture;
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
        const headers = {
            joincode: joincode,
            username: username,
            ...shooHeaders(shooAccountDetails()),
        };
        const response = await fetch(`https://${serverip}/gameState`, { headers });
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

                    const playerHeader = document.createElement('div');
                    playerHeader.classList.add('player-header');
                    playerHeader.appendChild(createPlayerPicture(playerData));

                    const healthText = document.createElement('p');
                    healthText.innerText = `${playerData.name}'s Health: ${playerData.health}`;
                    playerHeader.appendChild(healthText);
                    playerDiv.appendChild(playerHeader);

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

                            cardDiv.addEventListener('click', () => handleHandCardClick(card, cardDiv));

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
    document.getElementById('play-card').style.display = canPlayCardNow(card) ? 'block' : 'none';
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
    const playButton = document.getElementById('playCardBtn');
    playButton.onclick = activateCalculatedShooting
    playButton.disabled = false;
    const popupContent = document.getElementById('popupContent');
    popupContent.innerHTML = '';  // Clear previous popup content

    const card = deck.find(item => item.id === cardid);
    if (cardid === 'decision-missile') {
        createDecisionMissilePopup(card);
        playButton.style.display = 'block';
    } else if (cardid === 'multi-strike') {
        createMultiStrikePopup();
    } else {
        createPlayerSelectPopup(card);
        playButton.style.display = hasValidTargetsForCard(card) ? 'block' : 'none';
    }

    showPopup();  // Show the popup after generating content

}
function createDecisionMissilePopup(card) {
    const popupContent = document.getElementById('popupContent');

    const actionLabel = document.createElement('label');
    actionLabel.innerText = 'Choose an effect: ';
    popupContent.appendChild(actionLabel);

    const actionDropdown = document.createElement('select');
    actionDropdown.id = 'decisionMissileActionDropdown';

    const drawOption = document.createElement('option');
    drawOption.value = 'draw';
    drawOption.text = 'Draw 2 cards';
    actionDropdown.appendChild(drawOption);

    if (hasValidTargetsForCard(card)) {
        const damageOption = document.createElement('option');
        damageOption.value = 'damage';
        damageOption.text = 'Deal 2 damage';
        actionDropdown.appendChild(damageOption);
    }

    popupContent.appendChild(actionDropdown);
    popupContent.appendChild(document.createElement('br'));

    const targetWrapper = document.createElement('div');
    targetWrapper.id = 'decisionMissileTargetWrapper';
    popupContent.appendChild(targetWrapper);

    const renderTargetSelect = () => {
        targetWrapper.innerHTML = '';
        if (actionDropdown.value !== 'damage') return;

        const label = document.createElement('label');
        label.innerText = 'Select a player: ';
        targetWrapper.appendChild(label);

        const dropdown = document.createElement('select');
        dropdown.id = 'playerDropdown';
        validTargetsForCard(card).forEach(player => {
            const option = document.createElement('option');
            option.value = player.name;
            option.text = player.name;
            dropdown.appendChild(option);
        });
        targetWrapper.appendChild(dropdown);
        targetWrapper.appendChild(document.createElement('br'));
    };

    actionDropdown.addEventListener('change', renderTargetSelect);
    renderTargetSelect();
}
function createMultiStrikePopup() {
    const popupContent = document.getElementById('popupContent');

    const totalLabel = document.createElement('label');
    totalLabel.id = 'multiStrikeTotalLabel';
    popupContent.appendChild(totalLabel);

    const allocationList = document.createElement('div');
    allocationList.id = 'multiStrikeAllocationList';
    popupContent.appendChild(allocationList);

    gameData.players.forEach(player => {
        const row = document.createElement('div');
        row.classList.add('multi-strike-row');

        const label = document.createElement('label');
        label.htmlFor = `multiStrikeDamage_${player.name}`;
        label.innerText = player.name;
        row.appendChild(label);

        const input = document.createElement('input');
        input.type = 'number';
        input.id = `multiStrikeDamage_${player.name}`;
        input.dataset.target = player.name;
        input.classList.add('multi-strike-damage');
        input.min = '0';
        input.max = '7';
        input.step = '1';
        input.value = '0';
        input.addEventListener('input', updateMultiStrikeState);
        row.appendChild(input);

        allocationList.appendChild(row);
    });

    updateMultiStrikeState();
}
function multiStrikeAllocations() {
    return Array.from(document.querySelectorAll('.multi-strike-damage'))
        .map(input => ({
            target: input.dataset.target,
            damage: Number.parseInt(input.value, 10) || 0
        }))
        .filter(allocation => allocation.damage > 0);
}
function updateMultiStrikeState() {
    const allocations = multiStrikeAllocations();
    const total = allocations.reduce((sum, allocation) => sum + allocation.damage, 0);
    const targetCount = allocations.length;
    const selfAllocation = allocations.find(allocation => allocation.target === username);
    const needsSelfDamage = gameData.players.length === 2;
    const isValid = total === 7 && targetCount >= 2 && (!needsSelfDamage || (selfAllocation?.damage || 0) >= 1);

    const totalLabel = document.getElementById('multiStrikeTotalLabel');
    if (totalLabel) {
        const selfDamageText = needsSelfDamage ? ', including at least 1 to yourself' : '';
        totalLabel.innerText = `Assign 7 total damage across at least 2 targets${selfDamageText}. Current total: ${total}`;
    }

    const playButton = document.getElementById('playCardBtn');
    playButton.style.display = 'block';
    playButton.disabled = !isValid;
}
function activateCalculatedShooting() {
    const selectedPlayer = document.getElementById('playerDropdown')?.value;
    const decisionMissileAction = document.getElementById('decisionMissileActionDropdown')?.value;
    const multiStrikeDamageInputs = document.querySelectorAll('.multi-strike-damage');
    var cardid = cardSelected
    var headers = {
        joincode: joincode,
        username: username,
        cardid: cardid
    }
    if (selectedPlayer) headers.target = selectedPlayer
    if (decisionMissileAction) headers.decision_action = decisionMissileAction
    if (multiStrikeDamageInputs.length > 0) headers.multistrike_allocations = JSON.stringify(multiStrikeAllocations())
    postWithFallback(`https://${serverip}/activateCalculatedShooting`, headers)
    closePopup();  // Close the popup after playing the card
}
