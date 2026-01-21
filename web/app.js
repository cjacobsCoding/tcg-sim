const API_PREFIX = '/api';
let simulationResults = null;

async function fetchState()
{
    try 
    {
        const res = await fetch(`${API_PREFIX}/state`);
        return res.json();
    } 
    catch (e) 
    {
        console.error("Failed to fetch state:", e);
        return null;
    }
}

async function doStep(command)
{
    try 
    {
        let endpoint = '';
        switch(command) {
            case 'step':
                endpoint = '/step';
                break;
            case 'turn':
                endpoint = '/turn';
                break;
            case 'game':
                endpoint = '/game';
                break;
            case 'deck':
                endpoint = '/deck';
                simulationResults = await fetch(`${API_PREFIX}${endpoint}`, { method: "POST" }).then(r => r.json());
                // After deck simulation, restart the game
                await fetch(`${API_PREFIX}/restart`, { method: "POST" });
                updateDisplay(simulationResults.state);
                updateDeckInfo();
                return;
            case 'all':
                endpoint = '/all';
                simulationResults = await fetch(`${API_PREFIX}${endpoint}`, { method: "POST" }).then(r => r.json());
                // After all simulation, restart the game
                await fetch(`${API_PREFIX}/restart`, { method: "POST" });
                updateDisplay(simulationResults.state);
                updateDeckInfo();
                return;
            default:
                return;
        }
        
        const response = await fetch(`${API_PREFIX}${endpoint}`, { method: "POST" });
        const newState = await response.json();
        updateDisplay(newState);
    } 
    catch (e) 
    {
        console.error("Error during command:", e);
    }
}

async function restart()
{
    try 
    {
        const response = await fetch(`${API_PREFIX}/restart`, { method: "POST" });
        const newState = await response.json();
        updateDisplay(newState);
        simulationResults = null;
        updateDeckInfo();
    } 
    catch (e) 
    {
        console.error("Error restarting:", e);
    }
}

async function step()
{
    try 
    {
        const response = await fetch(`${API_PREFIX}/step`, { method: "POST" });
        const newState = await response.json();
        
        // Immediately update the display with the new state
        updateDisplay(newState);
    } 
    catch (e) 
    {
        console.error("Error during step:", e);
    }
}

async function stopServer()
{
    try 
    {
        await fetch(`${API_PREFIX}/shutdown`, { method: "POST" });
        // Give the server a moment to respond before closing
        setTimeout(() => 
        {
            alert("Server is shutting down. You can close this tab.");
            window.close();
        }, 100);
    } 
    catch (e) 
    {
        console.error("Error shutting down server:", e);
        alert("Error shutting down server");
    }
}

function formatPhase(phase) {
    // Convert GameStep enum to readable text
    const phaseNames = 
    {
        "StartTurn": "Start Turn",
        "Upkeep": "Upkeep",
        "Draw": "Draw",
        "Main": "Main",
        "Combat": "Combat",
        "EndTurn": "End Turn",
        "GameOver": "Game Over"
    };
    return phaseNames[phase] || phase;
}

function updateDeckInfo()
{
    const deckComp = document.getElementById("deck-composition");
    const results = document.getElementById("results");
    
    if (simulationResults) 
    {
        deckComp.textContent = `Deck: 29 Forests, 31 Grizzly Bears`;
        results.textContent = `Results: Avg ${simulationResults.avg_turns.toFixed(2)} turns over ${simulationResults.total_games} games`;
    } 
    else 
    {
        deckComp.textContent = `Deck: 29 Forests, 31 Grizzly Bears`;
        results.textContent = `Results: No simulation data yet`;
    }
}

// Helper function to check if a card is tapped
function cardIsTapped(card) 
{
    if (!card || !card.fragments) 
    {
        return false;
    }

    const f = card.fragments.Tappable;
    if (!f) 
    {
        return false;
    }

    if (typeof f.tapped === 'boolean') return f.tapped;
    if (f.Tappable && typeof f.Tappable.tapped === 'boolean') return f.Tappable.tapped;
    return false;
}

// Render a single player's zones
function renderPlayerZones(player, playerIndex, isCurrentPlayer)
{
    const container = document.createElement("div");
    container.className = `player-section ${isCurrentPlayer ? 'current-player' : ''}`;
    
    const title = document.createElement("h2");
    title.textContent = `Player ${playerIndex}${isCurrentPlayer ? ' (Current)' : ''} - ${player.life} HP`;
    container.className += isCurrentPlayer ? ' player-section-current' : ' player-section-other';
    container.appendChild(title);

    // Battlefield
    const battlefieldDiv = document.createElement("div");
    battlefieldDiv.className = "player-battlefield";
    
    const battlefieldTitle = document.createElement("h3");
    battlefieldTitle.textContent = "Battlefield";
    battlefieldDiv.appendChild(battlefieldTitle);
    
    const bfCards = player.zones.Battlefield || [];
    const grizzlies = bfCards.filter(c => c.name === "Grizzly Bears");
    const forests = bfCards.filter(c => c.name === "Forest");

    const grizzliesContainer = document.createElement("div");
    grizzliesContainer.className = "grizzlies-section";
    grizzlies.forEach(card => 
    {
        const img = document.createElement("img");
        img.src = `/cards/${encodeURIComponent(card.name)}.jpg`;
        img.className = "card";
        img.alt = card.name;
        
        if (cardIsTapped(card)) 
        {
            img.style.transform = `rotate(90deg)`;
        }

        grizzliesContainer.appendChild(img);
    });
    battlefieldDiv.appendChild(grizzliesContainer);

    const forestsContainer = document.createElement("div");
    forestsContainer.className = "forests-section";
    forests.forEach((card, i) => 
    {
        const img = document.createElement("img");
        img.src = `/cards/${encodeURIComponent(card.name)}.jpg`;
        img.className = "card";
        img.alt = card.name;

        const CARD_W = 90;
        const containerW = 800; // Estimate for forests section
        const overlap = CARD_W * 0.18;
        const totalWidth = (forests.length - 1) * overlap + CARD_W;
        const startX = (containerW - totalWidth) / 2;
        const left = startX + i * overlap;
        img.style.left = `${left}px`;
        img.style.top = `10px`;
        img.style.zIndex = `${i}`;

        if (cardIsTapped(card)) 
        {
            img.style.transform = `rotate(90deg)`;
        }

        forestsContainer.appendChild(img);
    });
    battlefieldDiv.appendChild(forestsContainer);
    
    container.appendChild(battlefieldDiv);

    // Hand & Library
    const handLibraryDiv = document.createElement("div");
    handLibraryDiv.className = "player-hand-library";

    const handLibTitle = document.createElement("h3");
    handLibTitle.textContent = "Hand & Library";
    handLibraryDiv.appendChild(handLibTitle);

    const handLibContent = document.createElement("div");
    handLibContent.className = "hand-library-content";

    // Library
    const libraryDiv = document.createElement("div");
    libraryDiv.className = "library-section";
    const libraryCards = player.zones.Library || [];
    
    const LIB_CARD_W = 90;
    const LIB_CARD_H = 120;
    const overlapW = LIB_CARD_W * 0.003;
    const overlapH = LIB_CARD_H * 0.003;

    libraryCards.forEach((card, i) => 
    {
        const img = document.createElement("img");
        img.src = `/cards/back.jpg`;
        img.className = "card back";
        img.alt = "card back";
        img.style.width = `${LIB_CARD_W}px`;
        img.style.height = `${LIB_CARD_H}px`;
        img.style.position = "absolute";
        
        const left = i * overlapW;
        const top = i * -overlapH;
        img.style.top = `${top}px`;
        img.style.left = `${left}px`;
        img.style.zIndex = `${i}`;
        
        libraryDiv.appendChild(img);
    });
    
    const libLabel = document.createElement("p");
    libLabel.className = "zone-label";
    libLabel.textContent = `Library (${libraryCards.length})`;
    libraryDiv.appendChild(libLabel);
    handLibContent.appendChild(libraryDiv);

    // Hand
    const handDiv = document.createElement("div");
    handDiv.className = "hand-section";
    const handCards = player.zones.Hand || [];

    const CARD_W = 90;
    const CARD_H = 120;
    const MAX_PER_CARD_ANGLE = 18;
    const MAX_SPAN = 90;
    const n = handCards.length;
    const span = n > 1 ? Math.min(MAX_SPAN, (n - 1) * MAX_PER_CARD_ANGLE) : 0;
    const step = n > 1 ? span / (n - 1) : 0;
    const centerIndex = (n - 1) / 2;

    const containerW = 600;
    const baseShift = CARD_W * 0.45;
    const MIN_OVERLAP = 0.10;
    const maxShiftOverlap = CARD_W * (1 - MIN_OVERLAP);
    const maxShiftContainer = n > 1 ? Math.max((containerW * 0.9 - CARD_W) / (n - 1), 10) : baseShift;
    const shift = Math.min(baseShift, maxShiftOverlap, maxShiftContainer);

    handCards.forEach((card, i) => 
    {
        const img = document.createElement("img");
        img.src = `/cards/${encodeURIComponent(card.name)}.jpg`;
        img.className = "card";
        img.alt = card.name;
        img.style.width = `${CARD_W}px`;
        img.style.height = `${CARD_H}px`;

        const angle = n > 1 ? -span / 2 + i * step : 0;
        const x = (i - centerIndex) * shift;
        const y = span > 0 ? (Math.abs(angle) / (span / 2 || 1)) * 40 : 0;
        const finalAngle = angle + (cardIsTapped(card) ? 90 : 0);

        img.style.transform = `translate(-50%, -50%) translateX(${x}px) rotate(${finalAngle}deg) translateY(${y}px)`;
        img.dataset.baseTransform = img.style.transform;

        img.addEventListener("mouseenter", () => {
            img.style.zIndex = "9999";
            img.style.transform = `${img.dataset.baseTransform} scale(1.35) translateY(-30px)`;
        });
        img.addEventListener("mouseleave", () => {
            img.style.zIndex = `${i * 10}`;
            img.style.transform = img.dataset.baseTransform;
        });
        img.style.zIndex = `${i * 10}`;

        handDiv.appendChild(img);
    });
    
    const handLabel = document.createElement("p");
    handLabel.className = "zone-label";
    handLabel.textContent = `Hand (${handCards.length})`;
    handDiv.appendChild(handLabel);
    handLibContent.appendChild(handDiv);

    handLibraryDiv.appendChild(handLibContent);
    container.appendChild(handLibraryDiv);

    return container;
}

function updateDisplay(state)
{
    if (!state) 
    {
        return;
    }

    // Update phase, current player, and turns
    const phaseElement = document.getElementById("phase");
    const currentPlayerElement = document.getElementById("currentPlayer");
    const playersHealthElement = document.getElementById("playersHealth");
    const turnsElement = document.getElementById("turns");
    
    phaseElement.textContent = formatPhase(state.step);
    currentPlayerElement.textContent = state.current_player_index;
    turnsElement.textContent = state.turns;
    
    // Display all players' health
    if (state.players && state.players.length > 0) {
        const healthText = state.players.map((p, i) => 
            `Player ${i}: ${p.life} HP ${i === state.current_player_index ? '(current)' : ''}`
        ).join(' | ');
        playersHealthElement.textContent = healthText;
    }

    // Render all players' zones
    const playersContainer = document.getElementById("players-container");
    playersContainer.innerHTML = "";
    
    if (state.players && state.players.length > 0) {
        state.players.forEach((player, index) => {
            const isCurrentPlayer = index === state.current_player_index;
            const playerDiv = renderPlayerZones(player, index, isCurrentPlayer);
            playersContainer.appendChild(playerDiv);
        });
    }

    // If the game reached GameOver, auto-restart to next game after short delay
    if (state.step === "GameOver") 
    {
        setTimeout(() => 
        {
            // Restart engine to next game
            restart();
        }, 3000);
    }
}

async function render()
{
    const state = await fetchState();
    updateDisplay(state);
}

// Music Functions
let musicEnabled = localStorage.getItem('musicEnabled') !== 'false';
let musicFiles = [];
let shuffledPlaylist = [];
let currentPlaylistIndex = 0;

// Fisher-Yates shuffle algorithm
function shuffleArray(array) 
{
    const shuffled = [...array];
    for (let i = shuffled.length - 1; i > 0; i--) 
    {
        const j = Math.floor(Math.random() * (i + 1));
        [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
    }
    return shuffled;
}

async function loadMusicFiles()
{
    try
    {
        const response = await fetch('/api/music-list');
        const data = await response.json();
        musicFiles = data.files.map(file => `/music/${file}`);
        
        if (musicFiles.length > 0)
        {
            // Create initial shuffled playlist
            shuffledPlaylist = shuffleArray(musicFiles);
            currentPlaylistIndex = 0;
            console.log("Music files loaded and shuffled:", shuffledPlaylist);
        }
        else
        {
            console.warn("No music files found on server");
            document.getElementById('nowPlayingName').textContent = 'No music files found';
        }
    }
    catch (e)
    {
        console.error("Failed to load music files:", e);
        document.getElementById('nowPlayingName').textContent = 'Error loading music';
    }
}

function updateNowPlaying()
{
    if (shuffledPlaylist.length > 0 && currentPlaylistIndex < shuffledPlaylist.length)
    {
        const currentFile = shuffledPlaylist[currentPlaylistIndex];
        // Extract the filename from the path
        const filename = currentFile.split('/').pop();
        document.getElementById('nowPlayingName').textContent = filename;
    }
}

function playNextMusic()
{
    if (musicFiles.length === 0)
    {
        // Try loading again
        loadMusicFiles().then(() =>
        {
            if (musicFiles.length > 0)
            {
                playNextMusic();
            }
        });
        return;
    }
    
    // If we've reached the end of the playlist, reshuffle
    if (currentPlaylistIndex >= shuffledPlaylist.length)
    {
        shuffledPlaylist = shuffleArray(musicFiles);
        currentPlaylistIndex = 0;
        console.log("Playlist reshuffled");
    }
    
    const audioElement = document.getElementById('backgroundMusic');
    const musicFile = shuffledPlaylist[currentPlaylistIndex];
    audioElement.src = musicFile;
    
    updateNowPlaying();
    
    if (musicEnabled)
    {
        audioElement.volume = 0.3; // 30% volume
        audioElement.play().catch(e => 
        {
            console.warn("Failed to play music:", e);
        });
    }
    
    currentPlaylistIndex++;
}

function skipMusic()
{
    const audioElement = document.getElementById('backgroundMusic');
    audioElement.pause();
    audioElement.currentTime = 0;
    
    // Play the next song after a short delay
    setTimeout(playNextMusic, 100);
}

function toggleMusic()
{
    musicEnabled = !musicEnabled;
    const button = document.getElementById('musicToggle');
    const audioElement = document.getElementById('backgroundMusic');
    
    if (musicEnabled)
    {
        button.textContent = '🔊 Music On';
        button.style.backgroundColor = 'lightgreen';
        if (audioElement.src)
        {
            audioElement.play().catch(e =>
            {
                console.warn("Failed to play music:", e);
            });
        }
    }
    else
    {
        button.textContent = '🔇 Music Off';
        button.style.backgroundColor = 'lightcoral';
        audioElement.pause();
    }
    
    localStorage.setItem('musicEnabled', musicEnabled);
}

// Initial render and setup
render();
updateDeckInfo();

// Load and play music
loadMusicFiles().then(() =>
{
    if (musicFiles.length > 0)
    {
        playNextMusic();
        
        // Setup music to play the next track when current ends
        const audioElement = document.getElementById('backgroundMusic');
        audioElement.addEventListener('ended', () =>
        {
            setTimeout(playNextMusic, 2000); // 2 second delay between songs
        });
    }
    else
    {
        console.warn("No music files found");
    }
});

// Restore music button state
if (!musicEnabled)
{
    const button = document.getElementById('musicToggle');
    button.textContent = '🔇 Music Off';
    button.style.backgroundColor = 'lightcoral';
}
