document.getElementById('start-game').addEventListener('click', startGame);
document.getElementById('end-turn').addEventListener('click', endTurn)
document.getElementById('spectate-when-dead').addEventListener('click', spectateGame)
document.getElementById('rejoin-game').addEventListener('click', rejoinGame)
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

function formatShootingType(shootingType) {
    return `${shootingType || ''}`.replace(/([a-z])([A-Z])/g, '$1 $2').toLowerCase();
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
            ...clerkHeaders(clerkAccountDetails()),
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
        const myPlayer = myPlayerIndex === -1 ? null : gameData.players[myPlayerIndex];
        const isDead = !myPlayer || myPlayer.health <= 0;
        const alivePlayers = gameData.players.filter(player => player.health > 0);
        const gameEnded = alivePlayers.length == 1;

        if (gameEnded) {
            document.getElementById('dead').style.display = 'none'
            document.getElementById('spectate-when-dead').style.display = 'none'
            document.getElementById('win').style.display = "block"
            document.getElementById('win').innerText = alivePlayers[0].name + " Wins"
            document.getElementById('rejoin-game').style.display = 'inline-block'
            document.getElementById('turn').style.display = 'none'
            document.getElementById('end-turn').style.display = 'none'
            document.getElementById('play-card').style.display = 'none'
            document.getElementById('no-shooting').style.display = 'none'
            document.getElementById('landmine').style.display = 'none'
            document.getElementById('card-play-status').style.display = 'none'
            const gameDiv = document.getElementById('game');
            gameDiv.innerHTML = '';  // Clear the game div
            renderedData = gameData
        } else if (isDead && !spectating) {
            document.getElementById('dead').style.display = 'block'
            document.getElementById('spectate-when-dead').style.display = 'block'
            document.getElementById('rejoin-game').style.display = 'none'
            document.getElementById('turn').style.display = 'none'
            document.getElementById('no-shooting').style.display = 'none'
            document.getElementById('landmine').style.display = 'none'
            document.getElementById('card-play-status').style.display = 'none'
            const gameDiv = document.getElementById('game');
            gameDiv.innerHTML = '';  // Clear the game div
        } else {
            document.getElementById('dead').style.display = 'none'
            document.getElementById('spectate-when-dead').style.display = 'none'
            document.getElementById('rejoin-game').style.display = 'none'
                document.getElementById('win').style.display = "none"
                document.getElementById('turn').style.display = ''
                document.getElementById('turn').innerText = gameData.players[gameData.current_turn_player].name + "'s Turn"
                if (gameData.players[gameData.current_turn_player].name == username) {
                    document.getElementById('end-turn').style.display = 'block'
                    document.getElementById('play-card').style.display = 'block'
                }
                else {
                    document.getElementById('end-turn').style.display = 'none'
                    document.getElementById('play-card').style.display = 'none'
                }
                const isMyTurn = gameData.players[gameData.current_turn_player].name == username;
                const statusDiv = document.getElementById('card-play-status');
                if (statusDiv) {
                    if (isMyTurn) {
                        statusDiv.style.display = 'block';
                        statusDiv.innerHTML = '';

                        // 1. Shooting
                        const distractorMissilePlayed = gameData.turn_state.distractor_missile_played || 0;
                        const maxShooting = distractorMissilePlayed > 0
                            ? gameData.turn_state.more_ammo_played
                            : 1 + gameData.turn_state.more_ammo_played;
                        const shootingAllowed = gameData.active_cards.no_shooting_played_by === -1 || gameData.players[gameData.active_cards.no_shooting_played_by].name === username;
                        const shootingLocked = gameData.turn_state.shooting_locked;

                        let shootingStatusText = "";
                        let shootingClass = "yes";
                        if (shootingLocked) {
                            shootingStatusText = "Shooting: Locked (0 remaining)";
                            shootingClass = "no";
                        } else if (!shootingAllowed) {
                            shootingStatusText = "Shooting: Blocked by No Shooting (0 remaining)";
                            shootingClass = "no";
                        } else {
                            const remaining = Math.max(0, maxShooting - gameData.turn_state.shooting_card_played);
                            shootingStatusText = `Shooting: ${remaining} remaining (played ${gameData.turn_state.shooting_card_played}/${maxShooting})`;
                            shootingClass = remaining > 0 ? "yes" : "no";
                        }

                        // 2. Event
                        let eventStatusText = "";
                        let eventClass = "yes";
                        if (gameData.turn_state.event_card_played) {
                            eventStatusText = "Event: 0 remaining (played 1/1)";
                            eventClass = "no";
                        } else {
                            eventStatusText = "Event: 1 remaining (played 0/1)";
                            eventClass = "yes";
                        }

                        // 3. More Ammo
                        let ammoStatusText = "";
                        let ammoClass = "yes";
                        if (shootingLocked) {
                            ammoStatusText = "More Ammo: Locked";
                            ammoClass = "no";
                        } else if (gameData.turn_state.event_card_played && gameData.turn_state.more_ammo_played === 0) {
                            ammoStatusText = "More Ammo: Blocked (Event played)";
                            ammoClass = "no";
                        } else {
                            ammoStatusText = `More Ammo: Playable (played ${gameData.turn_state.more_ammo_played})`;
                            ammoClass = "yes";
                        }

                        // 4. Support
                        const supportStatusText = "Support: Unlimited";
                        const supportClass = "unlimited";

                        // 5. Plus
                        const plusStatusText = "Plus: Unlimited";
                        const plusClass = "unlimited";

                        const appendTag = (text, className) => {
                            const span = document.createElement('span');
                            span.className = `playable-tag ${className}`;
                            span.innerText = text;
                            statusDiv.appendChild(span);
                        };

                        appendTag(shootingStatusText, shootingClass);
                        appendTag(eventStatusText, eventClass);
                        appendTag(ammoStatusText, ammoClass);
                        appendTag(supportStatusText, supportClass);
                        appendTag(plusStatusText, plusClass);
                    } else {
                        statusDiv.style.display = 'none';
                        statusDiv.innerHTML = '';
                    }
                }
                if (gameData.active_cards.no_shooting_played_by !== -1) {
                    document.getElementById('no-shooting').style.display = 'block'
                    document.getElementById('no-shooting').innerText = "No shooting was played by " + gameData.players[gameData.active_cards.no_shooting_played_by].name
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

                    let playerFiringFilters = gameData.active_cards.active_firing_filters.filter(f => f.owner === playerData.name);
                    if (playerFiringFilters.length > 0) {
                        const queuedCards = document.createElement('div')
                        queuedCards.classList.add('queuedCards')
                        for (var i = 0; i < playerFiringFilters.length; i++) {
                            const firingFilterData = playerFiringFilters[i];
                            var cardDisplay = document.createElement('p')
                            cardDisplay.innerText = `${firingFilterData.card_played.name} is filtering ${formatShootingType(firingFilterData.filter_type)} shooting`
                            queuedCards.appendChild(cardDisplay)
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
                        const isMyTurn = gameData.players[gameData.current_turn_player]?.name === username;
                        playerData.hand.forEach((card) => {
                            const cardDiv = document.createElement('div');
                            cardDiv.classList.add('card');
                            if (card.id === 'armor' || (isMyTurn && !canPlayCardNow(card))) {
                                cardDiv.classList.add('unplayable');
                            }

                            const img = document.createElement('img');
                            img.src = `https://${serverip}/images/${cardImageFolder(card.card_type)}/${card.id}.png`;  // Use card ID to get the front image
                            img.alt = card.name;
                            cardDiv.classList.add('my-hand');

                            cardDiv.appendChild(img);

                            cardDiv.addEventListener('click', () => handleHandCardClick(card, cardDiv));

                            handDiv.appendChild(cardDiv);
                        });
                    }

                    // Armor configuration UI (only for own player with armor cards)
                    if (isMyPlayer && playerData.hand.some(c => c.id === 'armor')) {
                        playerDiv.appendChild(createArmorPanel());
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
                showRecycleChoicePopup();
                handleArmorDisabledNotification();
                renderedData = gameData
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
    const account = clerkAccountDetails();
    const headers = {
        joincode: joincode,
        username: username,
        ...clerkHeaders(account)
    };
    if (typeof readLobbySettings === 'function') {
        headers.settings = JSON.stringify(readLobbySettings());
    }

    postWithFallbackNoJSON(`https://${serverip}/startGame`, headers)
        .then(response => {
            if (response && response.ok) {
                joinStartedGame();
            }
        });
}
function joinStartedGame() {
    document.querySelector('.lobby-screen').style.display = 'none'
    document.querySelector('.game-screen').style.display = 'block'
    document.getElementById('rejoin-game').style.display = 'none'
    spectating = false
    renderedData = {}
    clearChatBox()
    fetchDeck();
    clearIntervals()
    chatInterval=setInterval(function() {fetchChatMessages(joincode)}, chatCooldown);
    gameStateInterval = setInterval(fetchGameState, gameStateCooldown);
}
function rejoinGame() {
    const account = clerkAccountDetails();
    const headers = {
        'Content-Type': 'application/json',
        joincode: joincode,
        username: username,
        ...clerkHeaders(account),
    };

    fetch(`https://${serverip}/rejoinGame`, {
        method: 'POST',
        headers: headers
    })
        .then(response => {
            if (!response.ok) throw new Error('HTTPS failed');
            joincode = response.headers.get('joincode') || joincode;
            document.getElementById('joincode-input').value = joincode;
            document.getElementById('lobby-show-code').innerText = "JoinCode: " + joincode;
            document.querySelector('.game-screen').style.display = 'none';
            document.querySelector('.lobby-screen').style.display = 'block';
            document.querySelector('.container').style.display = 'flex';
            document.getElementById('rejoin-game').style.display = 'none';
            renderedData = {};
            spectating = false;
            clearIntervals();
            lobbyPlayersInterval = setInterval(lobbyPlayers, playersInLobbyCooldown);
            lobbyStartedCheckInterval = setInterval(lobbyStartedCheck, lobbyStartedCheckCoooldown);
            lobbyPlayers();
            startChat();
            loadMyClerkGames();
        })
        .catch(error => {
            console.error('HTTPS error:', error);
        });
}
function endTurn() {
    postWithFallbackNoJSON(`https://${serverip}/endTurn`, {
        joincode: joincode,
        username: username,
        ...clerkHeaders(clerkAccountDetails()),
    })
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
        const targets = validTargetsForCard(card);
        if (targets.length === 1) {
            activateCalculatedShootingHeaders({ target: targets[0].name });
            return;
        }

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
    var headers = {}
    if (selectedPlayer) headers.target = selectedPlayer
    if (decisionMissileAction) headers.decision_action = decisionMissileAction
    if (multiStrikeDamageInputs.length > 0) headers.multistrike_allocations = JSON.stringify(multiStrikeAllocations())
    activateCalculatedShootingHeaders(headers)
}

function activateCalculatedShootingHeaders(extraHeaders) {
    var headers = {
        joincode: joincode,
        username: username,
        cardid: cardSelected,
        ...extraHeaders,
        ...clerkHeaders(clerkAccountDetails()),
    }
    postWithFallback(`https://${serverip}/activateCalculatedShooting`, headers)
    closePopup();  // Close the popup after playing the card
}

// ── Armor configuration ──────────────────────────────────────────────

var lastArmorConfigs = [];

function currentArmorConfigs() {
    if (Array.isArray(gameData.armor_configs)) return gameData.armor_configs;
    return gameData.armor_config ? [gameData.armor_config] : [];
}

function createArmorPanel() {
    const panel = document.createElement('div');
    panel.classList.add('armor-panel');

    const configs = currentArmorConfigs();
    const isEnabled = configs.length > 0;

    // Sync last config from server state so defaults survive page reloads
    if (isEnabled && (lastArmorConfigs.length === 0 || configs.length >= lastArmorConfigs.length)) {
        lastArmorConfigs = configs.map(config => ({
            enabled: config.enabled !== false,
            threshold: config.threshold,
            discard_card_id: config.discard_card_id,
        }));
    }

    const header = document.createElement('div');
    header.classList.add('armor-header');

    const label = document.createElement('span');
    label.classList.add('armor-label');
    label.innerText = isEnabled ? `\u{1F6E1}\uFE0F Armor Armed (${configs.length})` : '\u{1F6E1}\uFE0F Armor';
    header.appendChild(label);

    if (isEnabled) {
        const configureBtn = document.createElement('button');
        configureBtn.type = 'button';
        configureBtn.classList.add('armor-enable-btn');
        configureBtn.innerText = 'Configure';
        configureBtn.addEventListener('click', () => showArmorConfigPopup());
        header.appendChild(configureBtn);

        const disableBtn = document.createElement('button');
        disableBtn.type = 'button';
        disableBtn.classList.add('armor-disable-btn');
        disableBtn.innerText = 'Disable';
        disableBtn.addEventListener('click', () => sendArmorConfig(false));
        header.appendChild(disableBtn);

        panel.appendChild(header);

        const myPlayer = gameData.players.find(p => p.name === username);
        const infoList = document.createElement('div');
        infoList.classList.add('armor-info-list');
        configs.forEach((config, index) => {
            const info = document.createElement('div');
            info.classList.add('armor-info');
            const discardCardName = myPlayer
                ?.hand.find(c => c.id === config.discard_card_id)?.name || config.discard_card_id;
            const status = config.enabled === false ? 'off' : 'on';
            info.innerText = `#${index + 1} (${status}): ${config.threshold}+ damage \u2022 discard ${discardCardName}`;
            infoList.appendChild(info);
        });
        panel.appendChild(infoList);
    } else {
        const enableBtn = document.createElement('button');
        enableBtn.type = 'button';
        enableBtn.classList.add('armor-enable-btn');
        enableBtn.innerText = 'Configure';
        enableBtn.addEventListener('click', () => showArmorConfigPopup());
        header.appendChild(enableBtn);

        panel.appendChild(header);
    }

    return panel;
}

function showArmorConfigPopup() {
    const popupContent = document.getElementById('popupContent');
    popupContent.innerHTML = '';

    const myPlayer = gameData.players.find(p => p.name === username);
    if (!myPlayer) return;

    const armorCount = myPlayer.hand.filter(card => card.id === 'armor').length;
    const discardOptions = myPlayer.hand.filter(card => card && card.id !== 'armor');

    if (armorCount === 0 || discardOptions.length === 0) {
        const noCards = document.createElement('p');
        noCards.innerText = 'You need at least one non-armor card to discard.';
        noCards.style.color = 'var(--danger)';
        popupContent.appendChild(noCards);

        const playButton = document.getElementById('playCardBtn');
        playButton.style.display = 'none';
        setPopupRequired(false);
        showPopup();
        return;
    }

    for (let armorIndex = 0; armorIndex < armorCount; armorIndex++) {
        const previous = lastArmorConfigs[armorIndex] || lastArmorConfigs[0] || null;
        const defaultEnabled = previous?.enabled !== false;
        const defaultThreshold = previous ? String(previous.threshold) : '1';
        const defaultDiscardId = previous && myPlayer.hand.some(c => c.id === previous.discard_card_id)
            ? previous.discard_card_id : null;

        const row = document.createElement('div');
        row.classList.add('armor-config-row');

        const title = document.createElement('div');
        title.classList.add('armor-config-title');
        title.innerText = `Armor #${armorIndex + 1}`;
        row.appendChild(title);

        const enabledLabel = document.createElement('label');
        enabledLabel.classList.add('armor-enabled-toggle');
        const enabledInput = document.createElement('input');
        enabledInput.type = 'checkbox';
        enabledInput.classList.add('armor-enabled-checkbox');
        enabledInput.checked = defaultEnabled;
        enabledInput.addEventListener('change', syncArmorDiscardSelections);
        enabledLabel.appendChild(enabledInput);
        enabledLabel.appendChild(document.createTextNode('Enabled'));
        row.appendChild(enabledLabel);

        const threshLabel = document.createElement('label');
        threshLabel.innerText = 'Minimum damage to activate:';
        row.appendChild(threshLabel);

        const threshInput = document.createElement('input');
        threshInput.type = 'number';
        threshInput.min = '1';
        threshInput.max = '99';
        threshInput.value = defaultThreshold;
        threshInput.classList.add('armor-threshold-input');
        row.appendChild(threshInput);

        const discardLabel = document.createElement('label');
        discardLabel.innerText = 'Card to discard when armor activates:';
        row.appendChild(discardLabel);

        const dropdown = document.createElement('select');
        dropdown.classList.add('armor-discard-select');
        discardOptions.forEach((card) => {
            const option = document.createElement('option');
            option.value = card.id;
            option.text = card.name;
            if (defaultDiscardId && card.id === defaultDiscardId) {
                option.selected = true;
            }
            dropdown.appendChild(option);
        });
        dropdown.addEventListener('change', syncArmorDiscardSelections);
        row.appendChild(dropdown);
        popupContent.appendChild(row);
    }

    const playButton = document.getElementById('playCardBtn');
    playButton.innerText = 'Arm Armor';
    playButton.style.display = 'block';
    playButton.disabled = false;
    playButton.onclick = () => {
        const configs = Array.from(document.querySelectorAll('.armor-config-row')).map(row => ({
            enabled: row.querySelector('.armor-enabled-checkbox')?.checked !== false,
            threshold: Number(row.querySelector('.armor-threshold-input')?.value || '1'),
            discard_card_id: row.querySelector('select')?.value,
        })).filter(config => config.discard_card_id);
        if (configs.length === 0) return;
        lastArmorConfigs = configs;
        sendArmorConfig(true, configs);
        playButton.innerText = 'Play Card';
        setPopupRequired(false);
        closePopup();
    };

    syncArmorDiscardSelections();
    setPopupRequired(false);
    showPopup();
}

function armorSelectedDiscardCounts() {
    return Array.from(document.querySelectorAll('.armor-discard-select'))
        .reduce((counts, dropdown) => {
            const row = dropdown.closest('.armor-config-row');
            const enabled = row?.querySelector('.armor-enabled-checkbox')?.checked !== false;
            if (enabled && dropdown.value) counts[dropdown.value] = (counts[dropdown.value] || 0) + 1;
            return counts;
        }, {});
}

function syncArmorDiscardSelections() {
    const myPlayer = gameData.players.find(p => p.name === username);
    if (!myPlayer) return;

    const availableCounts = myPlayer.hand
        .filter(card => card && card.id !== 'armor')
        .reduce((counts, card) => {
            counts[card.id] = (counts[card.id] || 0) + 1;
            return counts;
        }, {});
    const selectedCounts = armorSelectedDiscardCounts();

    document.querySelectorAll('.armor-discard-select').forEach(dropdown => {
        Array.from(dropdown.options).forEach(option => {
            const selectedByOthers = (selectedCounts[option.value] || 0)
                - (dropdown.value === option.value && dropdown.closest('.armor-config-row')?.querySelector('.armor-enabled-checkbox')?.checked !== false ? 1 : 0);
            option.disabled = selectedByOthers >= (availableCounts[option.value] || 0);
        });

        if (dropdown.selectedOptions[0]?.disabled) {
            const replacement = Array.from(dropdown.options).find(option => !option.disabled);
            if (replacement) dropdown.value = replacement.value;
        }
    });

    const finalCounts = armorSelectedDiscardCounts();
    const isValid = Object.entries(finalCounts)
        .every(([cardId, count]) => count <= (availableCounts[cardId] || 0));
    const playButton = document.getElementById('playCardBtn');
    if (playButton) playButton.disabled = !isValid;
}

function sendArmorConfig(enabled, configs) {
    const headers = {
        joincode: joincode,
        username: username,
        armor_enabled: enabled ? 'true' : 'false',
        ...clerkHeaders(clerkAccountDetails()),
    };
    if (configs != null) headers.armor_configs = JSON.stringify(configs);

    postWithFallback(`https://${serverip}/configureArmor`, headers);
    renderedData = {};
}

function handleArmorDisabledNotification() {
    if (gameData.armor_disabled) {
        const notice = document.createElement('div');
        notice.classList.add('armor-toast');
        notice.innerText = 'Armor was automatically disabled (discard card no longer available)';
        document.body.appendChild(notice);
        setTimeout(() => notice.remove(), 4000);
    }
}
