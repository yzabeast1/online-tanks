var shooAccount = null;
var shooSettings = null;

function shooIdentity() {
    if (!window.Shoo || !window.Shoo.getIdentity) {
        return null;
    }

    const identity = window.Shoo.getIdentity();
    if (!identity || !identity.userId) {
        return null;
    }

    return identity;
}

function decodeJwtPayload(token) {
    if (!token || token.split('.').length < 2) {
        return {};
    }

    try {
        let payload = token.split('.')[1].replace(/-/g, '+').replace(/_/g, '/');
        payload = payload.padEnd(payload.length + (4 - payload.length % 4) % 4, '=');
        return JSON.parse(atob(payload));
    } catch (error) {
        console.warn('Could not decode Shoo token payload:', error);
        return {};
    }
}

function shooAccountDetails() {
    const identity = shooIdentity();
    if (!identity) {
        return null;
    }

    const claims = decodeJwtPayload(identity.token);
    return {
        userId: identity.userId,
        token: identity.token,
        email: claims.email || null,
        name: claims.name || null,
        picture: claims.picture || null,
        claims,
    };
}

async function requireShooAccount() {
    const account = shooAccountDetails();
    if (account) {
        shooAccount = account;
        updateShooUi();
        return account;
    }

    window.Shoo.startSignIn({ requestPii: true, returnTo: window.location.pathname });
    return null;
}

function applyShooDefaults(account) {
    const usernameInput = document.getElementById('username-input');

    const defaultUsername = shooEffectiveUsername(account);
    const shooUsername = shooDefaultUsername(account);
    if (usernameInput && defaultUsername && (!usernameInput.value.trim() || usernameInput.value.trim() === shooUsername)) {
        usernameInput.value = defaultUsername;
    }
}

function shooDefaultUsername(account) {
    return account && (account.name || account.email || account.userId) || '';
}

function shooDefaultPicture(account) {
    return account && account.picture || '';
}

function shooEffectiveUsername(account) {
    return shooSettings && shooSettings.username
        ? shooSettings.username
        : shooDefaultUsername(account);
}

function shooEffectivePicture(account) {
    return shooSettings && shooSettings.picture
        ? shooSettings.picture
        : shooDefaultPicture(account);
}

function shooHeaders(account) {
    if (!account) {
        return {};
    }

    return {
        shoo_token: account.token,
    };
}

function shooSettingsHeaders(account) {
    return {
        ...shooHeaders(account),
    };
}

function updateShooUi() {
    const accountEl = document.getElementById('shoo-account');
    if (!accountEl) {
        return;
    }

    const account = shooAccount || shooAccountDetails();
    if (!account) {
        accountEl.innerHTML = '<button id="shoo-sign-in" type="button">Sign in with Shoo</button>';
        renderSettingsMenu(null);
        document.getElementById('shoo-sign-in').addEventListener('click', () => {
            window.Shoo.startSignIn({ requestPii: true, returnTo: window.location.pathname });
        });
        return;
    }

    shooAccount = account;
    accountEl.innerHTML = '';

    const summary = document.createElement('div');
    summary.className = 'shoo-account-summary';

    const effectivePicture = shooEffectivePicture(account);
    if (effectivePicture) {
        const image = document.createElement('img');
        image.className = 'shoo-account-picture';
        image.src = effectivePicture;
        image.alt = account.name || account.email || 'Shoo profile picture';
        summary.appendChild(image);
    }

    const details = document.createElement('div');
    details.className = 'shoo-account-details';

    const label = document.createElement('div');
    label.className = 'shoo-account-label';
    label.innerText = 'Signed in as';
    details.appendChild(label);

    const name = document.createElement('div');
    name.className = 'shoo-account-name';
    name.innerText = shooEffectiveUsername(account);
    details.appendChild(name);

    summary.appendChild(details);
    accountEl.appendChild(summary);

    const actions = document.createElement('div');
    actions.className = 'shoo-account-actions';

    const switchButton = document.createElement('button');
    switchButton.type = 'button';
    switchButton.innerText = 'Switch Account';
    switchButton.addEventListener('click', switchShooAccount);
    actions.appendChild(switchButton);

    const signOutButton = document.createElement('button');
    signOutButton.type = 'button';
    signOutButton.innerText = 'Sign Out';
    signOutButton.addEventListener('click', signOutShoo);
    actions.appendChild(signOutButton);

    accountEl.appendChild(actions);
    renderSettingsMenu(account);
}

function clearShooSession() {
    if (window.Shoo && window.Shoo.clearIdentity) {
        window.Shoo.clearIdentity();
    }
    shooAccount = null;
    shooSettings = null;
    renderSettingsMenu(null);
    renderMyShooGames([]);
}

function signOutShoo() {
    clearShooSession();
    updateShooUi();
}

function switchShooAccount() {
    clearShooSession();
    window.Shoo.startSignIn({ requestPii: true, returnTo: window.location.pathname });
}

async function loadMyShooGames() {
    const account = shooAccountDetails();
    const gamesEl = document.getElementById('my-games');
    if (!account || !gamesEl) {
        return;
    }

    try {
        const response = await fetch(`https://${serverip}/myGames`, {
            headers: shooHeaders(account),
        });
        if (!response.ok) {
            throw new Error('Could not load Shoo games');
        }

        const games = await response.json();
        renderMyShooGames(games);
    } catch (error) {
        console.error('Error loading Shoo games:', error);
    }
}

async function loadShooSettings() {
    const account = shooAccountDetails();
    if (!account) {
        shooSettings = null;
        renderSettingsMenu(null);
        return null;
    }

    try {
        const response = await fetch(`https://${serverip}/shooSettings`, {
            headers: shooSettingsHeaders(account),
        });
        if (!response.ok) {
            throw new Error('Could not load Shoo settings');
        }

        shooSettings = await response.json();
        applyShooDefaults(account);
        updateShooUi();
        return shooSettings;
    } catch (error) {
        console.error('Error loading Shoo settings:', error);
        renderSettingsMenu(account);
        return null;
    }
}

async function ensureShooSettingsLoaded(account) {
    if (!account || shooSettings) {
        return;
    }

    await loadShooSettings();
}

async function saveShooSettings(event) {
    if (event) {
        event.preventDefault();
    }

    const account = shooAccountDetails();
    if (!account) {
        return;
    }

    const usernameInput = document.getElementById('settings-username-input');
    const pictureInput = document.getElementById('settings-picture-input');
    const status = document.getElementById('settings-status');

    try {
        const response = await fetch(`https://${serverip}/shooSettings`, {
            method: 'POST',
            headers: {
                ...shooSettingsHeaders(account),
                settings_username: usernameInput ? usernameInput.value.trim() : '',
                settings_picture: pictureInput ? pictureInput.value.trim() : '',
            },
        });
        if (!response.ok) {
            throw new Error('Could not save Shoo settings');
        }

        shooSettings = await response.json();
        const usernameInputMain = document.getElementById('username-input');
        if (usernameInputMain) {
            usernameInputMain.value = shooEffectiveUsername(account);
        }
        updateShooUi();
        const details = document.querySelector('#settings-menu details');
        const updatedStatus = document.getElementById('settings-status');
        if (details) {
            details.open = true;
        }
        if (updatedStatus) {
            updatedStatus.innerText = 'Saved';
        }
    } catch (error) {
        console.error('Error saving Shoo settings:', error);
        if (status) {
            status.innerText = 'Could not save';
        }
    }
}

function resetShooSettings() {
    const account = shooAccountDetails();
    if (!account) {
        return;
    }

    const usernameInput = document.getElementById('settings-username-input');
    const pictureInput = document.getElementById('settings-picture-input');
    if (usernameInput) {
        usernameInput.value = shooDefaultUsername(account);
    }
    if (pictureInput) {
        pictureInput.value = shooDefaultPicture(account);
    }

    saveShooSettings();
}

function renderSettingsMenu(account) {
    const settingsEl = document.getElementById('settings-menu');
    if (!settingsEl) {
        return;
    }

    if (!account) {
        settingsEl.style.display = 'none';
        settingsEl.innerHTML = '';
        return;
    }

    settingsEl.style.display = 'block';
    settingsEl.innerHTML = `
        <details>
            <summary>Settings</summary>
            <form id="settings-form" class="settings-form">
                <label for="settings-username-input">Default username</label>
                <input type="text" id="settings-username-input" value="">
                <label for="settings-picture-input">Profile picture URL</label>
                <input type="text" id="settings-picture-input" value="">
                <div class="settings-actions">
                    <button type="submit">Save</button>
                    <button type="button" id="settings-reset">Reset Settings</button>
                </div>
                <div id="settings-status" class="settings-status"></div>
            </form>
        </details>
    `;

    document.getElementById('settings-username-input').value = shooEffectiveUsername(account);
    document.getElementById('settings-picture-input').value = shooEffectivePicture(account);
    document.getElementById('settings-form').addEventListener('submit', saveShooSettings);
    document.getElementById('settings-reset').addEventListener('click', resetShooSettings);
}

function renderMyShooGames(games) {
    const gamesEl = document.getElementById('my-games');
    if (!gamesEl) {
        return;
    }

    gamesEl.innerHTML = '';
    if (!games || games.length === 0) {
        return;
    }

    const title = document.createElement('div');
    title.className = 'my-games-title';
    title.innerText = 'Your games';
    gamesEl.appendChild(title);

    const account = shooAccountDetails();
    const defaultUsername = account ? shooEffectiveUsername(account) : '';

    games.forEach(game => {
        const gameButton = document.createElement('div');
        gameButton.className = 'my-game-button';
        gameButton.role = 'button';
        gameButton.tabIndex = 0;
        const usernameNote = game.username && game.username !== defaultUsername
            ? ` as ${game.username}`
            : '';
        const gameLabel = document.createElement('span');
        gameLabel.className = 'my-game-label';
        gameLabel.innerText = `${game.joincode} - ${game.status}${usernameNote}`;

        const gameDetails = document.createElement('div');
        gameDetails.className = 'my-game-details';
        if (game.status === 'started' && game.current_turn_player) {
            const turn = document.createElement('div');
            turn.className = 'my-game-turn';
            turn.innerText = `${game.current_turn_player}'s turn`;
            gameDetails.appendChild(turn);
        }
        const otherPlayers = (game.players || []).filter(player => player.name !== game.username);
        if (otherPlayers.length > 0) {
            const players = document.createElement('div');
            players.className = 'my-game-players';
            otherPlayers.forEach(player => {
                const playerButton = document.createElement('span');
                playerButton.className = 'my-game-player';
                playerButton.setAttribute('aria-label', player.name);
                if (player.name === game.current_turn_player) {
                    playerButton.classList.add('current-turn-player');
                }
                playerButton.appendChild(createPlayerPicture(player));
                const playerName = document.createElement('span');
                playerName.className = 'my-game-player-name';
                playerName.innerText = player.name;
                playerButton.appendChild(playerName);
                players.appendChild(playerButton);
            });
            gameDetails.appendChild(players);
        }

        const gameSummary = document.createElement('div');
        gameSummary.className = 'my-game-summary';
        gameSummary.appendChild(gameLabel);
        if (gameDetails.childElementCount > 0) {
            gameSummary.appendChild(gameDetails);
        }

        gameButton.addEventListener('click', () => rejoinShooGame(game));
        gameButton.addEventListener('keydown', event => {
            if (event.key === 'Enter' || event.key === ' ') {
                event.preventDefault();
                rejoinShooGame(game);
            }
        });

        const removeButton = document.createElement('button');
        removeButton.type = 'button';
        removeButton.className = 'my-game-remove';
        removeButton.innerHTML = '&#128465;';
        removeButton.setAttribute('aria-label', `Remove game ${game.joincode}`);
        removeButton.title = 'Remove game';
        removeButton.addEventListener('click', event => {
            event.stopPropagation();
            removeShooGame(game);
        });

        gameButton.appendChild(gameSummary);
        gameButton.appendChild(removeButton);
        gamesEl.appendChild(gameButton);
    });
}

async function removeShooGame(game) {
    try {
        const account = shooAccountDetails();
        const response = await fetch(`https://${serverip}/quitGame`, {
            method: 'POST',
            headers: {
                'joincode': game.joincode,
                'username': game.username,
                ...shooHeaders(account),
            },
        });

        if (!response.ok) {
            throw new Error('Could not remove Shoo game');
        }

        loadMyShooGames();
    } catch (error) {
        console.error('Error removing Shoo game:', error);
        alert('Could not remove game');
    }
}

function rejoinShooGame(game) {
    document.getElementById('username-input').value = game.username;
    document.getElementById('joincode-input').value = game.joincode;
    username = game.username;
    joincode = game.joincode;

    if (game.status === 'started') {
        document.querySelector('.menu-screen').style.display = 'none';
        document.querySelector('.lobby-screen').style.display = 'none';
        document.querySelector('.container').style.display = 'flex';
        document.querySelector('.game-screen').style.display = 'block';
        joinStartedGame();
    } else {
        joinLobby();
    }
}

window.addEventListener('load', () => {
    const account = shooAccountDetails();
    if (account) {
        shooAccount = account;
        applyShooDefaults(account);
    }
    updateShooUi();
    loadShooSettings();
    loadMyShooGames();
});
