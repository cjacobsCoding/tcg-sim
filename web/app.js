const API_PREFIX = '/api';
let simulationResults = null;

async function fetchState()
{
    try {
        const res = await fetch(`${API_PREFIX}/state`);
        return res.json();
    } catch (e) {
        console.error("Failed to fetch state:", e);
        return null;
    }
}

async function doStep(command)
{
    try {
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
    } catch (e) {
        console.error("Error during command:", e);
    }
}

async function restart()
{
    try {
        const response = await fetch(`${API_PREFIX}/restart`, { method: "POST" });
        const newState = await response.json();
        updateDisplay(newState);
        simulationResults = null;
        updateDeckInfo();
    } catch (e) {
        console.error("Error restarting:", e);
    }
}

async function step()
{
    try {
        const response = await fetch(`${API_PREFIX}/step`, { method: "POST" });
        const newState = await response.json();
        
        // Immediately update the display with the new state
        updateDisplay(newState);
    } catch (e) {
        console.error("Error during step:", e);
    }
}

async function stopServer()
{
    try {
        await fetch(`${API_PREFIX}/shutdown`, { method: "POST" });
        // Give the server a moment to respond before closing
        setTimeout(() => {
            alert("Server is shutting down. You can close this tab.");
            window.close();
        }, 100);
    } catch (e) {
        console.error("Error shutting down server:", e);
        alert("Error shutting down server");
    }
}

function formatPhase(phase) {
    // Convert GameStep enum to readable text
    const phaseNames = {
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
    
    if (simulationResults) {
        deckComp.textContent = `Deck: 29 Forests, 31 Grizzly Bears`;
        results.textContent = `Results: Avg ${simulationResults.avg_turns.toFixed(2)} turns over ${simulationResults.total_games} games`;
    } else {
        deckComp.textContent = `Deck: 29 Forests, 31 Grizzly Bears`;
        results.textContent = `Results: No simulation data yet`;
    }
}

function updateDisplay(state)
{
    if (!state) return;

    // Update phase, life, and turns
    const phaseElement = document.getElementById("phase");
    const lifeElement = document.getElementById("life");
    const turnsElement = document.getElementById("turns");
    
    phaseElement.textContent = formatPhase(state.step);
    lifeElement.textContent = state.life;
    turnsElement.textContent = state.turns;

    // Render hand with dynamic fan layout (no hidden cards)
    const hand = document.getElementById("hand");
    hand.innerHTML = "";
    const handCards = state.zones.Hand || [];
    const CARD_W = 90;
    const CARD_H = 120;
    const MAX_PER_CARD_ANGLE = 18; // degrees per card before capping
    const MAX_SPAN = 90; // maximum total fan span in degrees
    const n = handCards.length;
    const span = n > 1 ? Math.min(MAX_SPAN, (n - 1) * MAX_PER_CARD_ANGLE) : 0;
    const step = n > 1 ? span / (n - 1) : 0;
    const centerIndex = (n - 1) / 2;

    // compute shift per card but cap so cards never spread beyond container
    const containerW = hand.clientWidth || window.innerWidth;
    const baseShift = CARD_W * 0.45; // preferred separation
    const MIN_OVERLAP = 0.10; // require at least 10% overlap
    const maxShiftOverlap = CARD_W * (1 - MIN_OVERLAP); // max allowed shift to maintain min overlap
    const maxShiftContainer = n > 1 ? Math.max((containerW * 0.9 - CARD_W) / (n - 1), 10) : baseShift;
    const shift = Math.min(baseShift, maxShiftOverlap, maxShiftContainer);

    // debug: expose computed values to console for troubleshooting
    console.debug('hand fan', { n, containerW, baseShift, maxShiftOverlap, maxShiftContainer, shift, span, step });

    // tapped state -> rotate additional 90deg to the right
    function cardIsTapped(card) 
    {
        if (!card || !card.fragments) return false;
        const f = card.fragments.Tappable;
        if (!f) return false;
        if (typeof f.tapped === 'boolean') return f.tapped;
        if (f.Tappable && typeof f.Tappable.tapped === 'boolean') return f.Tappable.tapped;
        return false;
    }

    handCards.forEach((card, i) => {
        const img = document.createElement("img");
        img.src = `/cards/${encodeURIComponent(card.name)}.jpg`;
        img.className = "card";
        img.alt = card.name;
        img.style.width = `${CARD_W}px`;
        img.style.height = `${CARD_H}px`;

        // compute angle and position
        const angle = n > 1 ? -span / 2 + i * step : 0; // degrees
        const x = (i - centerIndex) * shift; // horizontal offset (capped)
        const maxYOffset = 40; // px
        const y = span > 0 ? (Math.abs(angle) / (span / 2 || 1)) * maxYOffset : 0;

        // tapped state -> rotate additional 90deg to the right
        const isTapped = card.fragments && card.fragments.Tappable && card.fragments.Tappable.tapped === true;
        const finalAngle = angle + (isTapped ? 90 : 0);

        // transform: center the card then offset and rotate
        img.style.transform = `translate(-50%, -40%) translateX(${x}px) rotate(${finalAngle}deg) translateY(${y}px)`;
        img.style.zIndex = `${(i * 10)}`;

        // debug: store shift/angle for inspection in devtools
        img.dataset.fan = JSON.stringify({ i, angle, x, y, shift });

        hand.appendChild(img);
    });

    // Render battlefield: separate Grizzly Bears and Forests
    const bfCards = state.zones.Battlefield || [];
    
    const grizzlies = bfCards.filter(c => c.name === "Grizzly Bears");
    const forests = bfCards.filter(c => c.name === "Forest");

    const grizzliesContainer = document.getElementById("battlefield-grizzlies");
    const forestsContainer = document.getElementById("battlefield-forests");
    
    grizzliesContainer.innerHTML = "";
    forestsContainer.innerHTML = "";

    // Render Grizzly Bears
    grizzlies.forEach(card => {
        const img = document.createElement("img");
        img.src = `/cards/${encodeURIComponent(card.name)}.jpg`;
        img.className = "card";
        img.alt = card.name;
        
        const isTapped = cardIsTapped(card);
        if (isTapped) {
            img.style.transform = `rotate(90deg)`;
        }
        else{
            img.style.transform = `rotate(0deg)`;
        }

        grizzliesContainer.appendChild(img);
    });

    // Render forests with slight horizontal offsets so all are visible (no hiding)
    forests.forEach((card, i) => {
        const img = document.createElement("img");
        img.src = `/cards/${encodeURIComponent(card.name)}.jpg`;
        img.className = "card";
        img.alt = card.name;
        img.style.width = `90px`;
        img.style.height = `120px`;
        img.style.position = 'absolute';

        const containerW = forestsContainer.clientWidth || 600;
        const CARD_W = 90;
        const overlap = CARD_W * 0.18; // how much to slide each subsequent forest
        const totalWidth = (forests.length - 1) * overlap + CARD_W;
        const startX = (containerW - totalWidth) / 2;
        const left = startX + i * overlap;
        img.style.left = `${left}px`;
        img.style.top = `10px`;
        img.style.zIndex = `${i}`;

        const isTapped = cardIsTapped(card);
        if (isTapped) {
            img.style.transform = `rotate(90deg)`;
        }
        else{
            img.style.transform = `rotate(0deg)`;
        }

        forestsContainer.appendChild(img);
    });

    // If the game reached GameOver, auto-restart to next game after short delay
    if (state.step === "GameOver") {
        setTimeout(() => {
            // Restart engine to next game
            restart();
        }, 300);
    }
}

async function render()
{
    const state = await fetchState();
    updateDisplay(state);
}

// Initial render and setup
render();
updateDeckInfo();
