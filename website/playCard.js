var cardSelected;
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
    return ((type && !['landmine', 'distractor-missile'].includes(card.id)) && !isCalculatedShooting)
        || card.id === 'airstrike'
        || card.id === 'steal'
        || card.id === 'health-hazard';
}

function firingFilterBlocksTarget(card, playerName) {
    const type = shootingType(card);
    if (!type) return false;

    return gameData.active_cards.active_firing_filters.some(filter =>
        filter.owner === playerName && filter.filter_type === type
    );
}

function validTargetsForCard(card) {
    return gameData.players.filter(player =>
        player.name !== username && !firingFilterBlocksTarget(card, player.name)
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

function handleHandCardClick(card, cardElement) {
    if (!canPlayCardNow(card)) return;

    cardSelected = card.id;
    clearInlineCardMenus();

    const actions = actionsForCard(card);
    if (actions.length === 0) {
        sendPlayedCardHeaders({});
        return;
    }

    if (actions.length === 1 && actions[0] === 'target') {
        const menu = document.createElement('div');
        menu.classList.add('inline-target-menu');

        validTargetsForCard(card).forEach(player => {
            const button = document.createElement('button');
            button.type = 'button';
            button.innerText = player.name;
            button.addEventListener('click', (event) => {
                event.stopPropagation();
                sendPlayedCardHeaders({ target: player.name });
                clearInlineCardMenus();
            });
            menu.appendChild(button);
        });

        cardElement.appendChild(menu);
        return;
    }

    playCard();
}
function triggerActions(actions) {
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
        }
    });

    showPopup();  // Show the popup after generating content
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

// Close popup
function closePopup() {
    const modal = document.getElementById('popupModal');
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
    console.log('Target:', selectedPlayer || 'No player selected');
    console.log('Discard options:', discardOptions || 'No discard');
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
    sendPlayedCardHeaders(headers);
}

function sendPlayedCardHeaders(extraHeaders) {
    var headers = {
        joincode: joincode,
        username: username,
        cardid: cardSelected,
        ...extraHeaders
    }
    console.log(headers)
    postWithFallback(`https://${serverip}/playcard`, headers)
    // You can now process the selected player and discard options here
    closePopup();  // Close the popup after playing the card
    closeModal();
}
function findCardInPlayerHand(cardId, username) {
    // Get the player's hand
    const player = gameData.players.find(p => p.name === username);
    if (!player) {
        console.error(`Player with username '${username}' not found.`);
        return null;
    }

    const playerHand = player.hand;
    console.log(playerHand)
    // Loop through the player's hand and check if any card matches the given cardId
    for (let i = 0; i < playerHand.length; i++) {
        const card = playerHand[i];
        console.log(card)
        if (card && card.id === cardId) {
            console.log(`Card with ID ${cardId} found in ${username}'s hand at hand index ${i}.`);
            return i;
        }
    }

    console.log(`Card with ID ${cardId} not found in ${username}'s hand.`);
    return null;
}
