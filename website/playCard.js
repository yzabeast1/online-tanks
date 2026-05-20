var cardSelected;
function playCard() {
    var card = document.getElementById('modalImage').src
    card = card.split("/")
    card = card[card.length - 1]
    card = card.split(".")
    card = card[0]
    cardSelected = card
    document.getElementById('playCardBtn').onclick = sendPlayedCardToServer
    
    let deckCard = deck.find(item => item.id === card);
    let actions = [];
    if (deckCard) {
        let isShooting = typeof deckCard.card_type === 'object' && deckCard.card_type !== null && deckCard.card_type.hasOwnProperty('Shooting');
        if (isShooting || deckCard.id === 'airstrike' || deckCard.id === 'steal') {
            actions.push('target');
        }
    }
    triggerActions(actions);
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

    const myPlayer = gameData.players.find(p => p.name === username);
    if (myPlayer) {
        const queuedCards = gameData.active_cards.active_calculated_shootings.filter(s => s.owner === myPlayer.name);
        queuedCards.forEach(s => {
            const option = document.createElement('option');
            option.value = s.card_played.id;
            option.text = `${s.card_played.name} with countdown ${s.turns_remaining}`;
            dropdown.appendChild(option);
        });
    }

    popupContent.appendChild(dropdown);
    popupContent.appendChild(document.createElement('br'))
}
function createPlayerSelectPopup() {
    const popupContent = document.getElementById('popupContent');

    const label = document.createElement('label');
    label.innerText = 'Select a player: ';
    popupContent.appendChild(label);

    const dropdown = document.createElement('select');
    dropdown.id = 'playerDropdown';
    dropdown.innerHTML = '';  // Clear any existing options

    gameData.players.forEach(player => {
        if (player.name !== username) {  // Exclude current player's name
            const option = document.createElement('option');
            option.value = player.name;
            option.text = player.name;
            dropdown.appendChild(option);
        }
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
        myPlayer.hand.forEach((card, index) => {
            if (card && card.id != cardSelected) {
                const option = document.createElement('option');
                option.value = index;
                option.text = card.name;
                dropdown.appendChild(option);
            }
        });
    }

    popupContent.appendChild(dropdown);
    popupContent.appendChild(document.createElement('br'))
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
    const queuedCard = document.getElementById('queuedCardDropdown')?.value
    console.log('Target:', selectedPlayer || 'No player selected');
    console.log('Discard options:', discardOptions || 'No discard');
    var cardid = findCardInPlayerHand(cardSelected, username);
    var headers = {
        joincode: joincode,
        username: username,
        cardid: cardid
    }
    if (selectedPlayer) headers['target'] = selectedPlayer
    if (discardOptions[0]) headers['discardcard'] = discardOptions[0]
    if (discardOptions[1]) headers['discardcardtwo'] = discardOptions[1]
    if (queuedCard) headers['queuedcard'] = queuedCard
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
