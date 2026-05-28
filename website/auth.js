var shooAccount = null;

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

    if (usernameInput && !usernameInput.value.trim() && account.name) {
        usernameInput.value = account.name;
    }
}

function shooHeaders(account) {
    return {
        shoo_user_id: account.userId,
        shoo_token: account.token || '',
    };
}

function logShooAccountDetails(action, account) {
    console.log(`Shoo account details for ${action}:`, {
        userId: account.userId,
        email: account.email,
        name: account.name,
        picture: account.picture,
        claims: account.claims,
    });
}

function updateShooUi() {
    const accountEl = document.getElementById('shoo-account');
    if (!accountEl) {
        return;
    }

    const account = shooAccount || shooAccountDetails();
    if (!account) {
        accountEl.innerHTML = '<button id="shoo-sign-in" type="button">Sign in with Shoo</button>';
        document.getElementById('shoo-sign-in').addEventListener('click', () => {
            window.Shoo.startSignIn({ requestPii: true, returnTo: window.location.pathname });
        });
        return;
    }

    shooAccount = account;
    accountEl.innerHTML = '';

    const summary = document.createElement('div');
    summary.className = 'shoo-account-summary';

    if (account.picture) {
        const image = document.createElement('img');
        image.className = 'shoo-account-picture';
        image.src = account.picture;
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
    name.innerText = account.name || account.email || account.userId;
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
}

function clearShooSession() {
    if (window.Shoo && window.Shoo.clearIdentity) {
        window.Shoo.clearIdentity();
    }
    shooAccount = null;
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
    const defaultUsername = account && account.name ? account.name : '';

    games.forEach(game => {
        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'my-game-button';
        const usernameNote = game.username && game.username !== defaultUsername
            ? ` as ${game.username}`
            : '';
        button.innerText = `${game.joincode} - ${game.status}${usernameNote}`;
        button.addEventListener('click', () => rejoinShooGame(game));
        gamesEl.appendChild(button);
    });
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
    loadMyShooGames();
});
