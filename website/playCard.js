var cardSelected;
var radarTouchDraggingRow = null;
function isShootingCard(card) {
    return card && typeof card.card_type === 'object' && card.card_type !== null && card.card_type.hasOwnProperty('Shooting');
}

function shootingType(card) {
    return isShootingCard(card) ? card.card_type.Shooting : null;
}

function cardNeedsTarget(card) {
    if (!card) return false;
    const type = shootingType(card);
    const isCalculatedShooting = type === 'Calculated';
    const canHealOthers = gameData.game_settings?.play_heal_on_others;
    return ((type && !['landmine', 'distractor-missile'].includes(card.id)) && !isCalculatedShooting)
        || card.id === 'airstrike'
        || card.id === 'steal'
        || card.id === 'health-hazard'
        || (canHealOthers && isHealingCard(card));
}

function firingFilterBlocksTarget(card, playerName) {
    const type = shootingType(card);
    if (!type) return false;

    return gameData.active_cards.active_firing_filters.some(filter =>
        filter.owner === playerName && filter.filter_type === type
    );
}

function isHealingCard(card) {
    return card && ['repair', 'repair-kit', 'new-model'].includes(card.id);
}

function validTargetsForCard(card) {
    if (isHealingCard(card)) {
        const maxHealth = gameData.game_settings?.max_health ?? 10;
        const canRevive = gameData.game_settings?.revive_others_with_heal;
        return gameData.players.filter(player =>
            (player.health > 0 || canRevive)
            && player.health < maxHealth
        );
    }

    // Health Hazard may be played against yourself — include all alive players
    if (card && card.id === 'health-hazard') {
        return gameData.players.filter(player => player.health > 0);
    }

    return gameData.players.filter(player =>
        player.health > 0
        && player.name !== username
        && !firingFilterBlocksTarget(card, player.name)
    );
}

function hasValidTargetsForCard(card) {
    return validTargetsForCard(card).length > 0;
}

function canPlayCardNow(card) {
    if (!card || !renderedData.players) return false;

    const isShooting = isShootingCard(card);
    const isEvent = card.card_type === 'Event';
    const isDistractorMissile = card.id === 'distractor-missile';
    const isColdWar = card.id === 'cold-war';
    const distractorMissilePlayed = renderedData.turn_state.distractor_missile_played || 0;
    const maxShooting = distractorMissilePlayed > 0
        ? renderedData.turn_state.more_ammo_played
        : 1 + renderedData.turn_state.more_ammo_played;
    const shootingAllowed = renderedData.active_cards.no_shooting_played_by === -1 || renderedData.players[renderedData.active_cards.no_shooting_played_by].name === username;
    const isMyTurn = renderedData.players[renderedData.current_turn_player]?.name === username;
    const hasQueuedDistractorTarget = validQueuedCardsForDistractorMissile().length > 0;

    if (!isMyTurn) return false;
    if (cardNeedsTarget(card) && !hasValidTargetsForCard(card)) return false;
    if (card.id === 'recycle' && !renderedData.players.some(player => player.health > 0 && player.name !== username && player.hand_count > 0)) return false;
    if (isDistractorMissile && (renderedData.turn_state.shooting_locked || !shootingAllowed || !hasQueuedDistractorTarget || renderedData.turn_state.shooting_card_played > renderedData.turn_state.more_ammo_played)) return false;
    if (isColdWar && renderedData.active_cards.active_calculated_shootings.length === 0) return false;
    if (isShooting && !isDistractorMissile && (renderedData.turn_state.shooting_locked || !shootingAllowed || renderedData.turn_state.shooting_card_played >= maxShooting)) return false;
    if (card.id === 'more-ammo' && renderedData.turn_state.shooting_locked) return false;
    if (isEvent && renderedData.turn_state.event_card_played) return false;
    if (['nuke', 'lottery'].includes(card.id) && renderedData.turn_state.total_cards_played > 0) return false;

    return true;
}

function actionsForCard(card) {
    let actions = [];

    if (cardNeedsTarget(card)) {
        actions.push('target');
    }
    if (card.id === 'new-model') {
        actions.push('discardcard');
        actions.push('discardcardtwo');
    }
    if (card.id === 'firing-filter') {
        actions.push('firing_filter_type');
    }
    if (card.id === 'distractor-missile' || card.id === 'cold-war') {
        actions.push('queuedcard');
    }
    if (card.id === 'helpful-hand') {
        actions.push('discardcard');
    }
    if (card.id === 'radar') {
        actions.push('radar_order');
    }

    return actions;
}

function validQueuedCardsForDistractorMissile() {
    return gameData.active_cards.active_calculated_shootings
        .filter(s => s.owner !== username && !firingFilterBlocksTarget({ card_type: { Shooting: 'Quick' } }, s.owner));
}

function validQueuedCardsForColdWar() {
    return gameData.active_cards.active_calculated_shootings
        .filter(s => s.owner !== username || s.turns_remaining > 0);
}

function playCard() {
    var card = cardSelected
    if (!card) {
        card = document.getElementById('modalImage').src
        card = card.split("/")
        card = card[card.length - 1]
        card = card.split(".")
        card = card[0]
    }
    cardSelected = card
    document.getElementById('playCardBtn').onclick = sendPlayedCardToServer
    document.getElementById('playCardBtn').style.display = 'block'
    
    let deckCard = deck.find(item => item.id === card);
    let actions = [];
    if (deckCard) {
        if (!canPlayCardNow(deckCard)) {
            document.getElementById('playCardBtn').style.display = 'none'
        }
        if (deckCard.id === 'distractor-missile' && validQueuedCardsForDistractorMissile().length === 0) {
            document.getElementById('playCardBtn').style.display = 'none'
        }
        if (deckCard.id === 'cold-war' && validQueuedCardsForColdWar().length === 0) {
            document.getElementById('playCardBtn').style.display = 'none'
        }
        actions = actionsForCard(deckCard);
    }
    triggerActions(actions);
}

function clearInlineCardMenus() {
    document.querySelectorAll('.inline-target-menu').forEach(menu => menu.remove());
}

function singleSelectionOptions(action, card) {
    if (action === 'target') {
        return validTargetsForCard(card).map(player => ({
            value: player.name,
            text: player.name,
            headers: { target: player.name },
        }));
    }

    if (action === 'queuedcard') {
        const queuedCards = card.id === 'cold-war'
            ? validQueuedCardsForColdWar()
            : validQueuedCardsForDistractorMissile();

        return queuedCards.map(s => {
            const index = gameData.active_cards.active_calculated_shootings.indexOf(s);
            const value = `${index}:${s.card_played.id}`;
            return {
                value,
                text: `${s.owner}'s ${s.card_played.name} with countdown ${s.turns_remaining}`,
                headers: { queuedcard: value },
            };
        });
    }

    if (action === 'discardcard') {
        const myPlayer = gameData.players.find(p => p.name === username);
        if (!myPlayer) return [];

        return myPlayer.hand
            .map((handCard, index) => ({ handCard, index }))
            .filter(({ handCard }) => handCard && handCard.id !== card.id)
            .map(({ handCard, index }) => {
                const value = `${index}:${handCard.id}`;
                return {
                    value,
                    text: handCard.name,
                    headers: { discardcard: value },
                };
            });
    }

    return null;
}

function showInlineSingleSelection(card, cardElement, action) {
    const options = singleSelectionOptions(action, card);
    if (!options || options.length === 0) return false;

    const menu = document.createElement('div');
    menu.classList.add('inline-target-menu');
    menu.addEventListener('click', event => event.stopPropagation());

    const dropdown = document.createElement('select');
    options.forEach(selection => {
        const option = document.createElement('option');
        option.value = selection.value;
        option.text = selection.text;
        dropdown.appendChild(option);
    });
    menu.appendChild(dropdown);

    const button = document.createElement('button');
    button.type = 'button';
    button.innerText = 'Play Card';
    button.addEventListener('click', (event) => {
        event.stopPropagation();
        const selected = options.find(selection => selection.value === dropdown.value);
        if (!selected) return;

        sendPlayedCardHeaders(selected.headers);
        clearInlineCardMenus();
    });
    menu.appendChild(button);

    cardElement.appendChild(menu);
    return true;
}

function handleHandCardClick(card, cardElement) {
    if (!canPlayCardNow(card)) return;

    cardSelected = card.id;
    clearInlineCardMenus();

    const actions = actionsForCard(card);
    if (actions.length === 0) {
        sendPlayedCardHeaders({});
        return;
    }

    if (actions.length === 1 && showInlineSingleSelection(card, cardElement, actions[0])) {
        return;
    }

    playCard();
}
function triggerActions(actions) {
    setPopupRequired(false);
    document.getElementById('playCardBtn').innerText = 'Play Card';
    const popupContent = document.getElementById('popupContent');
    popupContent.innerHTML = '';  // Clear previous popup content

    actions.forEach(action => {
        if (action === 'target') {
            createPlayerSelectPopup();
        } else if (action.startsWith('discard')) {
            createDiscardDropdown(action);
        } else if (action == 'queuedcard') {
            createQueuedCardDropdown(username);
        } else if (action == 'firing_filter_type') {
            createFiringFilterTypeDropdown();
        } else if (action == 'radar_order') {
            createRadarOrderControls();
        }
    });

    showPopup();  // Show the popup after generating content
}
function createRadarOrderControls() {
    const popupContent = document.getElementById('popupContent');
    const radarCards = gameData.radar_cards || [];

    const list = document.createElement('div');
    list.id = 'radarOrderList';
    list.classList.add('radar-order-list');

    radarCards.forEach((card, index) => {
        const row = document.createElement('div');
        row.classList.add('radar-order-card');
        row.dataset.originalIndex = index.toString();
        row.draggable = true;
        row.addEventListener('dragstart', handleRadarDragStart);
        row.addEventListener('dragend', handleRadarDragEnd);
        row.addEventListener('touchstart', handleRadarTouchStart, { passive: true });
        row.addEventListener('touchmove', handleRadarTouchMove, { passive: false });
        row.addEventListener('touchend', handleRadarTouchEnd);
        row.addEventListener('touchcancel', handleRadarTouchEnd);

        const img = document.createElement('img');
        img.src = `https://${serverip}/images/${cardImageFolder(card.card_type)}/${card.id}.png`;
        img.alt = card.name;
        row.appendChild(img);

        const label = document.createElement('span');
        label.innerText = card.name;
        row.appendChild(label);

        const controls = document.createElement('div');
        controls.classList.add('radar-order-controls');

        const upButton = document.createElement('button');
        upButton.type = 'button';
        upButton.innerText = 'Up';
        upButton.addEventListener('click', () => moveRadarCard(row, -1));
        controls.appendChild(upButton);

        const downButton = document.createElement('button');
        downButton.type = 'button';
        downButton.innerText = 'Down';
        downButton.addEventListener('click', () => moveRadarCard(row, 1));
        controls.appendChild(downButton);

        row.appendChild(controls);
        list.appendChild(row);
    });

    list.addEventListener('dragover', handleRadarDragOver);
    popupContent.appendChild(list);
    updateRadarOrderButtons();

    const playButton = document.getElementById('playCardBtn');
    playButton.style.display = radarCards.length > 0 ? 'block' : 'none';
}
function handleRadarDragStart(event) {
    event.currentTarget.classList.add('dragging');
    event.dataTransfer.effectAllowed = 'move';
    event.dataTransfer.setData('text/plain', event.currentTarget.dataset.originalIndex);
}
function handleRadarDragEnd(event) {
    event.currentTarget.classList.remove('dragging');
    updateRadarOrderButtons();
}
function handleRadarDragOver(event) {
    event.preventDefault();
    const draggingRow = document.querySelector('.radar-order-card.dragging');
    if (!draggingRow) return;

    moveDraggedRadarRow(event.currentTarget, draggingRow, event.clientY);
}
function handleRadarTouchStart(event) {
    if (event.target.closest('button')) return;
    radarTouchDraggingRow = event.currentTarget;
    radarTouchDraggingRow.classList.add('dragging');
}
function handleRadarTouchMove(event) {
    if (!radarTouchDraggingRow) return;

    event.preventDefault();
    const touch = event.touches[0];
    moveDraggedRadarRow(radarTouchDraggingRow.parentElement, radarTouchDraggingRow, touch.clientY);
}
function handleRadarTouchEnd() {
    if (!radarTouchDraggingRow) return;

    radarTouchDraggingRow.classList.remove('dragging');
    radarTouchDraggingRow = null;
    updateRadarOrderButtons();
}
function moveDraggedRadarRow(list, row, y) {
    const afterElement = radarDragAfterElement(list, y);
    if (afterElement) {
        list.insertBefore(row, afterElement);
    } else {
        list.appendChild(row);
    }
}
function radarDragAfterElement(list, y) {
    const rows = Array.from(list.querySelectorAll('.radar-order-card:not(.dragging)'));

    return rows.reduce((closest, row) => {
        const box = row.getBoundingClientRect();
        const offset = y - box.top - box.height / 2;
        if (offset < 0 && offset > closest.offset) {
            return { offset, element: row };
        }
        return closest;
    }, { offset: Number.NEGATIVE_INFINITY, element: null }).element;
}
function moveRadarCard(row, direction) {
    const sibling = direction < 0 ? row.previousElementSibling : row.nextElementSibling;
    if (!sibling) return;

    if (direction < 0) {
        row.parentElement.insertBefore(row, sibling);
    } else {
        row.parentElement.insertBefore(sibling, row);
    }
    updateRadarOrderButtons();
}
function updateRadarOrderButtons() {
    const rows = Array.from(document.querySelectorAll('.radar-order-card'));
    rows.forEach((row, index) => {
        const buttons = row.querySelectorAll('button');
        if (buttons[0]) buttons[0].disabled = index === 0;
        if (buttons[1]) buttons[1].disabled = index === rows.length - 1;
    });
}
function radarOrder() {
    return Array.from(document.querySelectorAll('.radar-order-card'))
        .map(row => Number.parseInt(row.dataset.originalIndex, 10));
}
function createQueuedCardDropdown(username) {
    const popupContent = document.getElementById('popupContent');
    const label = document.createElement('label');
    label.innerText = `Select a queued card: `;
    popupContent.appendChild(label);

    const dropdown = document.createElement('select');
    dropdown.id = `queuedCardDropdown`;
    dropdown.innerHTML = '';  // Clear any existing options

    const selectedCard = deck.find(item => item.id === cardSelected);
    const queuedCards = selectedCard?.id === 'cold-war'
        ? validQueuedCardsForColdWar()
        : validQueuedCardsForDistractorMissile();

    queuedCards
        .forEach(s => {
            const option = document.createElement('option');
            const index = gameData.active_cards.active_calculated_shootings.indexOf(s);
            option.value = `${index}:${s.card_played.id}`;
            option.text = `${s.owner}'s ${s.card_played.name} with countdown ${s.turns_remaining}`;
            dropdown.appendChild(option);
        });

    popupContent.appendChild(dropdown);
    popupContent.appendChild(document.createElement('br'))
}
function createFiringFilterTypeDropdown() {
    const popupContent = document.getElementById('popupContent');
    const label = document.createElement('label');
    label.innerText = 'Select a shooting type: ';
    popupContent.appendChild(label);

    const dropdown = document.createElement('select');
    dropdown.id = 'firingFilterTypeDropdown';
    dropdown.innerHTML = '';

    ['Quick', 'Calculated', 'Boom'].forEach(type => {
        const option = document.createElement('option');
        option.value = type;
        option.text = type;
        dropdown.appendChild(option);
    });

    popupContent.appendChild(dropdown);
    popupContent.appendChild(document.createElement('br'))
}
function createPlayerSelectPopup(card = deck.find(item => item.id === cardSelected)) {
    const popupContent = document.getElementById('popupContent');

    const label = document.createElement('label');
    label.innerText = 'Select a player: ';
    popupContent.appendChild(label);

    const dropdown = document.createElement('select');
    dropdown.id = 'playerDropdown';
    dropdown.innerHTML = '';  // Clear any existing options

    validTargetsForCard(card).forEach(player => {
        const option = document.createElement('option');
        option.value = player.name;
        option.text = player.name;
        dropdown.appendChild(option);
    });

    popupContent.appendChild(dropdown);
    popupContent.appendChild(document.createElement('br'))
}
function createDiscardDropdown(discardAction) {
    const popupContent = document.getElementById('popupContent');
    const label = document.createElement('label');
    label.innerText = `Select a card to discard: `;
    popupContent.appendChild(label);

    const dropdown = document.createElement('select');
    dropdown.id = `discardDropdown_${discardAction}`;
    dropdown.innerHTML = '';  // Clear any existing options

    const myPlayer = gameData.players.find(p => p.name === username);
    if (myPlayer) {
        const playedCard = deck.find(item => item.id === cardSelected);
        myPlayer.hand.forEach((card, index) => {
            const canDiscardForPlayedCard = playedCard?.id !== 'new-model' || isShootingCard(card);
            if (card && card.id != cardSelected && canDiscardForPlayedCard) {
                const option = document.createElement('option');
                option.value = `${index}:${card.id}`;
                option.text = card.name;
                dropdown.appendChild(option);
            }
        });
    }

    popupContent.appendChild(dropdown);
    popupContent.appendChild(document.createElement('br'))
    syncDiscardDropdowns();
}

function syncDiscardDropdowns() {
    const discardDropdowns = Array.from(document.querySelectorAll('[id^=discardDropdown_]'));
    if (discardDropdowns.length < 2) return;

    discardDropdowns.forEach(dropdown => {
        dropdown.onchange = syncDiscardDropdowns;
    });

    discardDropdowns.forEach(dropdown => {
        const selectedInOtherDropdowns = discardDropdowns
            .filter(otherDropdown => otherDropdown !== dropdown)
            .map(otherDropdown => otherDropdown.value);

        Array.from(dropdown.options).forEach(option => {
            option.disabled = selectedInOtherDropdowns.includes(option.value);
        });

        if (dropdown.selectedOptions[0]?.disabled) {
            const replacement = Array.from(dropdown.options).find(option => !option.disabled);
            if (replacement) {
                dropdown.value = replacement.value;
            }
        }
    });
}
function showPopup() {
    const modal = document.getElementById('popupModal');
    modal.style.display = 'flex';
}

function setPopupRequired(required) {
    const modal = document.getElementById('popupModal');
    const closeButton = modal.querySelector('.close-btn');
    modal.dataset.required = required ? 'true' : 'false';
    if (closeButton) closeButton.style.display = required ? 'none' : 'block';
}

// Close popup
function closePopup() {
    const modal = document.getElementById('popupModal');
    if (modal.dataset.required === 'true') return;
    modal.style.display = 'none';
}

// Close the modal with ESC key
document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') {
        closePopup();
    }
});
function sendPlayedCardToServer() {
    const selectedPlayer = document.getElementById('playerDropdown')?.value;
    const discardOptions = Array.from(document.querySelectorAll('[id^=discardDropdown_]'))
        .map(dropdown => dropdown.value);
    if (new Set(discardOptions).size !== discardOptions.length) {
        return;
    }
    const queuedCard = document.getElementById('queuedCardDropdown')?.value
    const firingFilterType = document.getElementById('firingFilterTypeDropdown')?.value
    const radarOrderValues = radarOrder();
    var headers = {
        joincode: joincode,
        username: username,
        cardid: cardSelected
    }
    if (selectedPlayer) headers['target'] = selectedPlayer
    if (discardOptions[0]) headers['discardcard'] = discardOptions[0]
    if (discardOptions[1]) headers['discardcardtwo'] = discardOptions[1]
    if (queuedCard) headers['queuedcard'] = queuedCard
    if (firingFilterType) headers['firing_filter_type'] = firingFilterType
    if (radarOrderValues.length > 0) headers['radar_order'] = JSON.stringify(radarOrderValues)
    sendPlayedCardHeaders(headers);
}

function sendPlayedCardHeaders(extraHeaders) {
    setPopupRequired(false);
    var headers = {
        joincode: joincode,
        username: username,
        cardid: cardSelected,
        ...extraHeaders,
        ...clerkHeaders(clerkAccountDetails()),
    }
    postWithFallback(`https://${serverip}/playcard`, headers)
    // You can now process the selected player and discard options here
    closePopup();  // Close the popup after playing the card
    closeModal();
}

function showRecycleChoicePopup() {
    const pendingRecycle = gameData.pending_recycle;
    if (!pendingRecycle) return;

    if (pendingRecycle.awaiting_discards?.includes(username)) {
        showRecycleDiscardPopup();
        return;
    }

    if (pendingRecycle.player !== username || pendingRecycle.awaiting_discards?.length > 0) return;

    if (document.getElementById('recycleCardDropdown')) {
        setPopupRequired(true);
        showPopup();
        return;
    }

    const popupContent = document.getElementById('popupContent');
    popupContent.innerHTML = '';

    const label = document.createElement('label');
    label.innerText = 'Choose a card to recycle: ';
    popupContent.appendChild(label);

    const dropdown = document.createElement('select');
    dropdown.id = 'recycleCardDropdown';
    pendingRecycle.cards.forEach((card, index) => {
        const option = document.createElement('option');
        option.value = `${index}:${card.id}`;
        option.text = card.name;
        dropdown.appendChild(option);
    });
    popupContent.appendChild(dropdown);
    popupContent.appendChild(document.createElement('br'));

    const playButton = document.getElementById('playCardBtn');
    playButton.onclick = sendRecycleChoiceToServer;
    playButton.innerText = 'Choose Card';
    playButton.style.display = pendingRecycle.cards.length > 0 ? 'block' : 'none';
    setPopupRequired(true);
    showPopup();
}

function showRecycleDiscardPopup() {
    if (document.getElementById('recycleDiscardDropdown')) {
        setPopupRequired(true);
        showPopup();
        return;
    }

    const myPlayer = gameData.players.find(player => player.name === username);
    if (!myPlayer || myPlayer.hand.length === 0) return;

    const popupContent = document.getElementById('popupContent');
    popupContent.innerHTML = '';

    const label = document.createElement('label');
    label.innerText = 'Choose a card to discard for Recycle: ';
    popupContent.appendChild(label);

    const dropdown = document.createElement('select');
    dropdown.id = 'recycleDiscardDropdown';
    myPlayer.hand.forEach((card, index) => {
        const option = document.createElement('option');
        option.value = `${index}:${card.id}`;
        option.text = card.name;
        dropdown.appendChild(option);
    });
    popupContent.appendChild(dropdown);
    popupContent.appendChild(document.createElement('br'));

    const playButton = document.getElementById('playCardBtn');
    playButton.onclick = sendRecycleDiscardToServer;
    playButton.innerText = 'Discard Card';
    playButton.style.display = 'block';
    setPopupRequired(true);
    showPopup();
}

function sendRecycleDiscardToServer() {
    const discardCard = document.getElementById('recycleDiscardDropdown')?.value;
    if (!discardCard) return;

    postWithFallback(`https://${serverip}/chooseRecycleDiscard`, {
        joincode: joincode,
        username: username,
        discardcard: discardCard,
        ...clerkHeaders(clerkAccountDetails()),
    });
    setPopupRequired(false);
    document.getElementById('playCardBtn').innerText = 'Play Card';
    closePopup();
}

function sendRecycleChoiceToServer() {
    const recycleCard = document.getElementById('recycleCardDropdown')?.value;
    if (!recycleCard) return;

    postWithFallback(`https://${serverip}/chooseRecycleCard`, {
        joincode: joincode,
        username: username,
        recyclecard: recycleCard,
        ...clerkHeaders(clerkAccountDetails()),
    });
    setPopupRequired(false);
    document.getElementById('playCardBtn').innerText = 'Play Card';
    closePopup();
}
function findCardInPlayerHand(cardId, username) {
    // Get the player's hand
    const player = gameData.players.find(p => p.name === username);
    if (!player) {
        console.error(`Player with username '${username}' not found.`);
        return null;
    }

    const playerHand = player.hand;
    // Loop through the player's hand and check if any card matches the given cardId
    for (let i = 0; i < playerHand.length; i++) {
        const card = playerHand[i];
        if (card && card.id === cardId) {
            return i;
        }
    }

    return null;
}
