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

    // Render hand
    const hand = document.getElementById("hand");
    hand.innerHTML = "";
    const handCards = state.zones.Hand || [];
    handCards.forEach(card => {
        const img = document.createElement("img");
        img.src = `/cards/${encodeURIComponent(card.name)}.jpg`;
        img.className = "card";
        img.alt = card.name;
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
        grizzliesContainer.appendChild(img);
    });

    // Render stacked Forests
    forests.forEach(card => {
        const img = document.createElement("img");
        img.src = `/cards/${encodeURIComponent(card.name)}.jpg`;
        img.className = "card";
        img.alt = card.name;
        forestsContainer.appendChild(img);
    });
}

async function render()
{
    const state = await fetchState();
    updateDisplay(state);
}

// Initial render and setup
render();
updateDeckInfo();
