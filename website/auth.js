var clerkAccount = null;

function clerkUser() {
    if (!window.Clerk || !window.Clerk.user || !window.Clerk.user.id) {
        return null;
    }

    return window.Clerk.user;
}

function clerkDefaultUsername(account) {
    return account && (account.username || account.name || account.email || account.userId) || '';
}

async function waitForClerk() {
    if (window.clerkLoadPromise) {
        await window.clerkLoadPromise;
    } else if (window.Clerk && window.Clerk.load && !window.Clerk.loaded) {
        await window.Clerk.load();
    }
}

async function hydrateClerkAccount() {
    await waitForClerk();

    const user = clerkUser();
    if (!user) {
        clerkAccount = null;
        return null;
    }

    let token = null;
    if (window.Clerk.session && window.Clerk.session.getToken) {
        try {
            token = await window.Clerk.session.getToken();
        } catch (error) {
            console.warn('Could not fetch Clerk session token:', error);
        }
    }

    clerkAccount = {
        userId: user.id,
        token,
        username: user.username || null,
        name: user.fullName || null,
        email: user.primaryEmailAddress ? user.primaryEmailAddress.emailAddress : null,
        picture: user.imageUrl || null,
        claims: {
            username: user.username || null,
            fullName: user.fullName || null,
            imageUrl: user.imageUrl || null,
        },
    };

    return clerkAccount;
}

function clerkAccountDetails() {
    return clerkAccount;
}

async function requireClerkAccount() {
    const account = clerkAccountDetails() || await hydrateClerkAccount();
    if (account && account.token) {
        return account;
    }

    if (window.Clerk && window.Clerk.redirectToSignIn) {
        await window.Clerk.redirectToSignIn({ strategy: 'oauth_google', signInForceRedirectUrl: window.location.pathname });
    }
    return null;
}

function applyClerkDefaults(account) {
    const usernameInput = document.getElementById('username-input');

    const defaultUsername = clerkEffectiveUsername(account);
    const clerkUsername = clerkDefaultUsername(account);
    if (usernameInput && defaultUsername && (!usernameInput.value.trim() || usernameInput.value.trim() === clerkUsername)) {
        usernameInput.value = defaultUsername;
    }
}

function clerkEffectiveUsername(account) {
    return clerkDefaultUsername(account);
}

function clerkHeaders(account) {
    if (!account || !account.token || !account.userId) {
        return {};
    }

    const headers = {
        clerk_token: account.token,
        clerk_user_id: account.userId,
    };

    if (account.picture) {
        headers.clerk_picture = account.picture;
    }

    return headers;
}

async function loadMyClerkGames() {
    const account = clerkAccountDetails() || await hydrateClerkAccount();
    const gamesEl = document.getElementById('my-games');
    if (!account || !account.token || !gamesEl) {
        if (gamesEl) {
            renderMyClerkGames([]);
        }
        return;
    }

    try {
        const response = await fetch(`https://${serverip}/myGames`, {
            headers: clerkHeaders(account),
        });
        if (!response.ok) {
            throw new Error('Could not load Clerk games');
        }

        const games = await response.json();
        renderMyClerkGames(games);
    } catch (error) {
        console.error('Error loading Clerk games:', error);
    }
}

function renderMyClerkGames(games) {
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

    const account = clerkAccountDetails();
    const defaultUsername = account ? clerkEffectiveUsername(account) : '';

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

        gameButton.addEventListener('click', () => rejoinClerkGame(game));
        gameButton.addEventListener('keydown', event => {
            if (event.key === 'Enter' || event.key === ' ') {
                event.preventDefault();
                rejoinClerkGame(game);
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
            removeClerkGame(game);
        });

        gameButton.appendChild(gameSummary);
        gameButton.appendChild(removeButton);
        gamesEl.appendChild(gameButton);
    });
}

async function removeClerkGame(game) {
    try {
        const account = clerkAccountDetails() || await hydrateClerkAccount();
        if (!account || !account.token) {
            throw new Error('Sign in is required to remove saved Clerk games');
        }

        const response = await fetch(`https://${serverip}/quitGame`, {
            method: 'POST',
            headers: {
                'joincode': game.joincode,
                'username': game.username,
                ...clerkHeaders(account),
            },
        });

        if (!response.ok) {
            throw new Error('Could not remove Clerk game');
        }

        loadMyClerkGames();
    } catch (error) {
        console.error('Error removing Clerk game:', error);
        alert('Could not remove game');
    }
}

function rejoinClerkGame(game) {
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

async function initializeClerkAccount() {
    await hydrateClerkAccount();

    if (window.Clerk && window.Clerk.addListener) {
        window.Clerk.addListener(async () => {
            const account = await hydrateClerkAccount();
            if (account) {
                applyClerkDefaults(account);
                await loadMyClerkGames();
            } else {
                renderMyClerkGames([]);
            }
        }, { skipInitialEmit: true });
    }

    const account = clerkAccountDetails();
    if (account) {
        applyClerkDefaults(account);
    }
    loadMyClerkGames();
}

window.addEventListener('load', () => {
    initializeClerkAccount();
});
