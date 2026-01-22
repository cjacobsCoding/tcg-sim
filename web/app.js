const API_PREFIX = '/api';
let simulationResults = null;

// Combat selection state
let selectedAttackers = new Set();
let selectedBlockers = new Map(); // blocker index -> attacker index
let currentGameState = null;

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
        "Untap": "Untap",
        "Upkeep": "Upkeep",
        "Draw": "Draw",
        "Main": "Main",
        "DeclareAttackers": "Declare Attackers",
        "DeclareBlockers": "Declare Blockers",
        "AssignDamage": "Assign Damage",
        "EndTurn": "End Turn",
        "GameOver": "Game Over"
    };
    return phaseNames[phase] || phase;
}

async function declareAttackers() {
    try {
        const payload = {
            attacking_indices: Array.from(selectedAttackers)
        };
        const response = await fetch(`${API_PREFIX}/declare-attackers`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(payload)
        });
        const newState = await response.json();
        selectedAttackers.clear();
        updateDisplay(newState);
    } catch (e) {
        console.error("Error declaring attackers:", e);
    }
}

async function declareBlockers() {
    try {
        const blocking_map = {};
        for (const [blocker, attacker] of selectedBlockers.entries()) {
            blocking_map[blocker] = attacker;
        }
        const payload = { blocking_map };
        const response = await fetch(`${API_PREFIX}/declare-blockers`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(payload)
        });
        const newState = await response.json();
        selectedBlockers.clear();
        updateDisplay(newState);
    } catch (e) {
        console.error("Error declaring blockers:", e);
    }
}

function toggleAttackerSelection(creatureIndex) {
    if (selectedAttackers.has(creatureIndex)) {
        selectedAttackers.delete(creatureIndex);
    } else {
        selectedAttackers.add(creatureIndex);
    }
}

function toggleBlockerSelection(blockerIndex, attackerIndex) {
    if (selectedBlockers.has(blockerIndex)) {
        selectedBlockers.delete(blockerIndex);
    } else {
        selectedBlockers.set(blockerIndex, attackerIndex);
    }
}

function drawBlockingArrows() {
    // Clear existing arrows
    const existingSvg = document.getElementById('blocking-arrows-svg');
    if (existingSvg) {
        existingSvg.remove();
    }
    
    if (selectedBlockers.size === 0 || !currentGameState) {
        return;
    }
    
    // Create SVG overlay for arrows
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.id = 'blocking-arrows-svg';
    svg.style.position = 'absolute';
    svg.style.top = '0';
    svg.style.left = '0';
    svg.style.width = '100%';
    svg.style.height = '100%';
    svg.style.pointerEvents = 'none';
    svg.style.zIndex = '5000';
    
    const defs = document.createElementNS('http://www.w3.org/2000/svg', 'defs');
    const marker = document.createElementNS('http://www.w3.org/2000/svg', 'marker');
    marker.setAttribute('id', 'arrow');
    marker.setAttribute('markerWidth', '10');
    marker.setAttribute('markerHeight', '10');
    marker.setAttribute('refX', '9');
    marker.setAttribute('refY', '3');
    marker.setAttribute('orient', 'auto');
    marker.setAttribute('markerUnits', 'strokeWidth');
    const polygon = document.createElementNS('http://www.w3.org/2000/svg', 'polygon');
    polygon.setAttribute('points', '0 0, 10 3, 0 6');
    polygon.setAttribute('fill', '#FF9500');
    marker.appendChild(polygon);
    defs.appendChild(marker);
    svg.appendChild(defs);
    
    // Draw lines for each blocking relationship
    for (const [blockerIdx, attackerIdx] of selectedBlockers.entries()) {
        const blockerCard = document.querySelector(`[data-creature-index="${blockerIdx}"].blocker-card`);
        const attackerCard = document.querySelector(`[data-creature-index="${attackerIdx}"].attacker-card`);
        
        if (blockerCard && attackerCard) {
            const blockerRect = blockerCard.getBoundingClientRect();
            const attackerRect = attackerCard.getBoundingClientRect();
            
            const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
            line.setAttribute('x1', blockerRect.left + blockerRect.width / 2);
            line.setAttribute('y1', blockerRect.top + blockerRect.height / 2);
            line.setAttribute('x2', attackerRect.left + attackerRect.width / 2);
            line.setAttribute('y2', attackerRect.top + attackerRect.height / 2);
            line.setAttribute('stroke', '#FF9500');
            line.setAttribute('stroke-width', '2');
            line.setAttribute('marker-end', 'url(#arrow)');
            svg.appendChild(line);
        }
    }
    
    document.body.appendChild(svg);
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
    container.className = `player-section ${isCurrentPlayer ? 'player-section-current' : 'player-section-other'}`;
    
    const title = document.createElement("h2");
    title.textContent = `Player ${playerIndex}${isCurrentPlayer ? ' (Current)' : ''} - ${player.life} Life`;
    if (!isCurrentPlayer) {
        title.style.transform = 'rotate(180deg)';
        title.style.transformOrigin = 'center';
    }
    container.appendChild(title);

    // Battlefield
    const battlefieldDiv = document.createElement("div");
    battlefieldDiv.className = "player-battlefield";
    
    const battlefieldTitle = document.createElement("h3");
    battlefieldTitle.textContent = "Battlefield";
    // Keep battlefield title upright for inactive player
    if (!isCurrentPlayer) battlefieldTitle.style.transform = 'rotate(180deg)';
    battlefieldDiv.appendChild(battlefieldTitle);
    
    const bfCards = player.zones.Battlefield || [];
    const grizzlies = bfCards.map((c, idx) => ({ card: c, index: idx })).filter(c => c.card.name === "Grizzly Bears");
    const forests = bfCards.filter(c => c.name === "Forest");

    const grizzliesContainer = document.createElement("div");
    grizzliesContainer.className = "grizzlies-section";
    grizzlies.forEach((g, gi) => 
    {
        const card = g.card;
        const creatureIndex = g.index;
        const img = document.createElement("img");
        img.src = `/cards/${encodeURIComponent(card.name)}.jpg`;
        img.className = "card";
        img.alt = card.name;
        img.dataset.creatureIndex = creatureIndex;
        img.dataset.playerIndex = playerIndex;
        
        if (cardIsTapped(card)) 
        {
            img.style.transform = `translateX(300%) translateY(70%) rotate(90deg) translateX(-10px)` + (isCurrentPlayer ? '' : ' rotate(180deg)');
        } else if (!isCurrentPlayer) {
            // ensure grizzly is upright for inactive player
            img.style.transform = `translateX(300%) translateY(70%) rotate(180deg)`;
        } else {
            // active untapped grizzly: set inline transform explicitly to avoid CSS matrix issues
            img.style.transform = `translateX(300%) translateY(70%)`;
        }

        img.style.transformOrigin = 'center';
        // save base transform and add hover handlers to preserve position when scaling
        img.dataset.baseTransform = img.style.transform;
        img.addEventListener('mouseenter', () => {
            img.style.zIndex = '9999';
            img.style.transform = `${img.dataset.baseTransform} scale(1.15)`;
        });
        img.addEventListener('mouseleave', () => {
            img.style.zIndex = `${gi}`;
            img.style.transform = img.dataset.baseTransform;
        });

        // Add click handling for attack declaration and blocking
        if (isCurrentPlayer && currentGameState && currentGameState.step === "DeclareAttackers") {
            img.classList.add('attacker-card');
            img.style.cursor = 'pointer';
            img.style.border = selectedAttackers.has(creatureIndex) ? '3px solid red' : '3px solid transparent';
            img.addEventListener('click', () => {
                toggleAttackerSelection(creatureIndex);
                img.style.border = selectedAttackers.has(creatureIndex) ? '3px solid red' : '3px solid transparent';
                drawBlockingArrows();
            });
        } else if (!isCurrentPlayer && currentGameState && currentGameState.step === "DeclareBlockers") {
            img.classList.add('blocker-card');
            img.style.cursor = 'pointer';
            img.style.border = selectedBlockers.has(creatureIndex) ? '3px solid orange' : '3px solid transparent';
            img.addEventListener('click', () => {
                // For now, auto-assign to first attacker if available
                if (currentGameState && currentGameState.attacking_creatures && currentGameState.attacking_creatures.length > 0) {
                    const targetAttacker = currentGameState.attacking_creatures[0];
                    toggleBlockerSelection(creatureIndex, targetAttacker);
                    img.style.border = selectedBlockers.has(creatureIndex) ? '3px solid orange' : '3px solid transparent';
                    drawBlockingArrows();
                }
            });
        } else if (currentGameState && currentGameState.step === "DeclareAttackers") {
            img.classList.add('attacker-card');
        } else if (!isCurrentPlayer) {
            img.classList.add('blocker-card');
        }

        grizzliesContainer.appendChild(img);
    });
    battlefieldDiv.appendChild(grizzliesContainer);

    const forestsContainer = document.createElement("div");
    forestsContainer.className = "forests-section";

    // Group untapped forests with horizontal overlap and stack tapped forests to save room
    const untappedForests = forests.filter(f => !cardIsTapped(f));
    const tappedForests = forests.filter(f => cardIsTapped(f));

    {
    const CARD_W = 70;
    const containerW = 450;
    const overlap = CARD_W * 0.35;

    // Untapped group (left side)
    if (untappedForests.length > 0) {
        const totalWidth = (untappedForests.length - 1) * overlap + CARD_W;
        const startX = (containerW - totalWidth) / 2;
        untappedForests.forEach((card, i) => {
            const img = document.createElement("img");
            img.src = `/cards/${encodeURIComponent(card.name)}.jpg`;
            img.className = "card";
            img.alt = card.name;

            const left = startX + i * overlap;
            img.style.left = `${left}px`;
            img.style.top = `10px`;
            img.style.zIndex = `${i}`;

            // Keep untapped orientation upright; if inactive player, counter-rotate children
            img.style.transform = (isCurrentPlayer ? `translate(${left}px, ${10}px)` : `translate(${left}px, ${10}px) rotate(180deg)`);
            img.style.transformOrigin = 'center';

            // hover handlers
            img.dataset.baseTransform = img.style.transform;
            img.addEventListener('mouseenter', () => {
                img.style.zIndex = '9999';
                img.style.transform = `${img.dataset.baseTransform} scale(1.15)`;
            });
            img.addEventListener('mouseleave', () => {
                img.style.zIndex = `${i}`;
                img.style.transform = img.dataset.baseTransform;
            });

            forestsContainer.appendChild(img);
        });
    }

    // Tapped stack (right side of forest group) - stacked overlap to save horizontal space
    if (tappedForests.length > 0) {
        const stackXBase = untappedForests.length > 0 ? ( (containerW + ((untappedForests.length - 1) * overlap + CARD_W)) / 2 + 8 ) : (containerW / 2 - CARD_W / 2);
        tappedForests.forEach((card, i) => {
            const img = document.createElement("img");
            img.src = `/cards/${encodeURIComponent(card.name)}.jpg`;
            img.className = "card";
            img.alt = card.name;

            // small horizontal offset between stacked tapped cards
            const left = stackXBase + i * 6;
            const top = 10 + i * 6; // slight vertical offset to show stack
            img.style.left = `${left}px`;
            img.style.top = `${top}px`;
            img.style.zIndex = `${i + 100}`;

            // tapped: rotate 90deg for tapped orientation; if inactive player also add 180deg
            img.style.transform = (isCurrentPlayer ? `translate(${left}px, ${top}px) rotate(90deg)` : `translate(${left}px, ${top}px) rotate(90deg) rotate(180deg)`);
            img.style.transformOrigin = 'center';

            // base transform & hover handlers for tapped stack
            img.dataset.baseTransform = img.style.transform;
            img.addEventListener('mouseenter', () => {
                img.style.zIndex = '9999';
                img.style.transform = `${img.dataset.baseTransform} scale(1.15)`;
            });
            img.addEventListener('mouseleave', () => {
                img.style.zIndex = `${i + 100}`;
                img.style.transform = img.dataset.baseTransform;
            });

            forestsContainer.appendChild(img);
        });
    }
    battlefieldDiv.appendChild(forestsContainer);
    }
    
    container.appendChild(battlefieldDiv);

    // Hand & Library
    const handLibraryDiv = document.createElement("div");
    handLibraryDiv.className = "player-hand-library";

    const handLibContent = document.createElement("div");
    handLibContent.className = "hand-library-content";

    // Library
    const libraryDiv = document.createElement("div");
    libraryDiv.className = "library-section";
    const libraryCards = player.zones.Library || [];
    
    const LIB_CARD_W = 60;
    const LIB_CARD_H = 80;
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
        
        img.style.transformOrigin = 'center';
        img.dataset.baseTransform = img.style.transform || '';
        img.addEventListener('mouseenter', () => {
            img.style.zIndex = '9999';
            img.style.transform = `${img.dataset.baseTransform} scale(1.15)`.trim();
        });
        img.addEventListener('mouseleave', () => {
            img.style.zIndex = `${i}`;
            img.style.transform = img.dataset.baseTransform;
        });
        
        libraryDiv.appendChild(img);
    });
    
    const libLabel = document.createElement("p");
    libLabel.className = "zone-label";
    libLabel.textContent = `Library (${libraryCards.length})`;
    if (!isCurrentPlayer) libLabel.style.transform = 'rotate(180deg)';
    libraryDiv.appendChild(libLabel);
    handLibContent.appendChild(libraryDiv);

    // Graveyard
    const graveyardDiv = document.createElement("div");
    graveyardDiv.className = "graveyard-section";
    const graveyardCards = player.zones.Graveyard || [];
    
    const GY_CARD_W = 60;
    const GY_CARD_H = 80;
    const gyOverlapW = GY_CARD_W * 0.05;
    const gyOverlapH = GY_CARD_H * 0.05;

    graveyardCards.forEach((card, i) => 
    {
        const img = document.createElement("img");
        img.src = `/cards/${encodeURIComponent(card.name)}.jpg`;
        img.className = "card";
        img.alt = card.name;
        img.style.width = `${GY_CARD_W}px`;
        img.style.height = `${GY_CARD_H}px`;
        img.style.position = "absolute";
        
        const left = i * gyOverlapW;
        const top = i * gyOverlapH;
        img.style.top = `${top}px`;
        img.style.left = `${left}px`;
        img.style.zIndex = `${i}`;
        
        img.style.transformOrigin = 'center';
        img.dataset.baseTransform = '';
        img.addEventListener('mouseenter', () => {
            img.style.zIndex = '9999';
            img.style.transform = `scale(1.15)`;
        });
        img.addEventListener('mouseleave', () => {
            img.style.zIndex = `${i}`;
            img.style.transform = '';
        });
        
        graveyardDiv.appendChild(img);
    });
    
    const gyLabel = document.createElement("p");
    gyLabel.className = "zone-label";
    gyLabel.textContent = `Graveyard (${graveyardCards.length})`;
    if (!isCurrentPlayer) gyLabel.style.transform = 'rotate(180deg)';
    graveyardDiv.appendChild(gyLabel);
    handLibContent.appendChild(graveyardDiv);

    // Hand
    const handDiv = document.createElement("div");
    handDiv.className = "hand-section";
    const handCards = player.zones.Hand || [];

    const CARD_W = 60;
    const CARD_H = 80;
    const MAX_PER_CARD_ANGLE = 12;
    const MAX_SPAN = 60;
    const n = handCards.length;
    const span = n > 1 ? Math.min(MAX_SPAN, (n - 1) * MAX_PER_CARD_ANGLE) : 0;
    const step = n > 1 ? span / (n - 1) : 0;
    const centerIndex = (n - 1) / 2;

    const containerW = 400;
    const baseShift = CARD_W * 0.35;
    const MIN_OVERLAP = 0.10;
    const maxShiftOverlap = CARD_W * (1 - MIN_OVERLAP);
    const maxShiftContainer = n > 1 ? Math.max((containerW * 0.8 - CARD_W) / (n - 1), 8) : baseShift;
    const shift = Math.min(baseShift, maxShiftOverlap, maxShiftContainer);

    handCards.forEach((card, i) => 
    {
        const img = document.createElement("img");
            // Use back of card for inactive player's hand
            img.src = isCurrentPlayer ? `/cards/${encodeURIComponent(card.name)}.jpg` : `/cards/back.jpg`;
        img.className = "card";
        img.alt = card.name;
        img.style.width = `${CARD_W}px`;
        img.style.height = `${CARD_H}px`;

        const angle = n > 1 ? -span / 2 + i * step : 0;
        const x = (i - centerIndex) * shift;
        const y = span > 0 ? (Math.abs(angle) / (span / 2 || 1)) * 30 : 0;
            // If this is the inactive player their zone is rotated 180deg overall,
            // so add an extra 180deg to card rotation to keep the card visually upright.
            const tappedAngle = cardIsTapped(card) ? 90 : 0;
            const finalAngle = angle + tappedAngle + (isCurrentPlayer ? 0 : 180);

        img.style.transform = `translate(-50%, -50%) translateX(${x}px) rotate(${finalAngle}deg) translateY(${y}px)`;
        img.style.transformOrigin = 'center';
        img.dataset.baseTransform = img.style.transform;

        img.addEventListener("mouseenter", () => {
            img.style.zIndex = "9999";
            img.style.transform = `${img.dataset.baseTransform} scale(1.2)`;
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
    if (!isCurrentPlayer) handLabel.style.transform = 'rotate(180deg)';
    handDiv.appendChild(handLabel);
    handLibContent.appendChild(handDiv);

    handLibraryDiv.appendChild(handLibContent);
    container.appendChild(handLibraryDiv);

    // Add action buttons for current player during combat phases
    if (isCurrentPlayer && currentGameState) {
        const actionDiv = document.createElement("div");
        actionDiv.className = "action-buttons";
        actionDiv.style.marginTop = "10px";
        actionDiv.style.textAlign = "center";
        
        if (currentGameState.step === "DeclareAttackers") {
            const confirmBtn = document.createElement("button");
            confirmBtn.textContent = `Declare Attackers (${selectedAttackers.size} selected)`;
            confirmBtn.style.padding = "10px 20px";
            confirmBtn.style.backgroundColor = "#4CAF50";
            confirmBtn.style.color = "white";
            confirmBtn.style.border = "none";
            confirmBtn.style.borderRadius = "4px";
            confirmBtn.style.cursor = "pointer";
            confirmBtn.style.fontSize = "14px";
            confirmBtn.addEventListener('click', declareAttackers);
            actionDiv.appendChild(confirmBtn);
        } else if (currentGameState.step === "DeclareBlockers") {
            const confirmBtn = document.createElement("button");
            confirmBtn.textContent = `Declare Blockers (${selectedBlockers.size} assigned)`;
            confirmBtn.style.padding = "10px 20px";
            confirmBtn.style.backgroundColor = "#2196F3";
            confirmBtn.style.color = "white";
            confirmBtn.style.border = "none";
            confirmBtn.style.borderRadius = "4px";
            confirmBtn.style.cursor = "pointer";
            confirmBtn.style.fontSize = "14px";
            confirmBtn.addEventListener('click', declareBlockers);
            actionDiv.appendChild(confirmBtn);
        }
        
        if (actionDiv.children.length > 0) {
            container.appendChild(actionDiv);
        }
    }

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
            `Player ${i}: ${p.life} Life ${i === state.current_player_index ? '(current)' : ''}`
        ).join(' | ');
        playersHealthElement.textContent = healthText;
    }

    // Render all players' zones in circular wedge arrangement
    const playersContainer = document.getElementById("players-container");
    
    // Rotate the entire container based on current player
    // For 2 players: container at 0° when Player 0 active, 180° when Player 1 active
    const playerCount = state.players ? state.players.length : 1;
    const rotationPerPlayer = 360 / playerCount;
    const containerRotation = state.current_player_index * rotationPerPlayer;
    
    // Apply smooth rotation transition to the container
    playersContainer.style.transition = "transform 0.6s ease-in-out";
    playersContainer.style.transform = `rotate(${containerRotation}deg)`;
    
    playersContainer.innerHTML = "";
    
    if (state.players && state.players.length > 0) {
        state.players.forEach((player, index) => {
            const isCurrentPlayer = index === state.current_player_index;
            
            // Create wrapper for positioning in the wedge
            const wrapper = document.createElement("div");
            wrapper.className = "player-wrapper";
            wrapper.setAttribute("data-player-index", index);
            
            // Create the player section
            const playerDiv = renderPlayerZones(player, index, isCurrentPlayer);
            
            // Counter-rotate player content to maintain correct orientation
            // Active player should always be right-side up (0°)
            // Inactive player zones should be rotationally symmetric (180°)
            // but cards and text inside should be kept right-side up by adding
            // a 180° rotation on the children where needed.
            let playerContentTransform;
            if (isCurrentPlayer) {
                // Active: keep right-side up by counter-rotating the container rotation
                playerContentTransform = `rotate(${-containerRotation}deg)`;
            } else {
                // Inactive: rotate zones 180° for symmetry
                playerContentTransform = `rotate(${180 - containerRotation}deg)`;
            }
            
            playerDiv.style.transform = playerContentTransform;
            playerDiv.style.transition = "transform 0.6s ease-in-out";
            
            wrapper.appendChild(playerDiv);
            playersContainer.appendChild(wrapper);
        });
    }

    // Store current game state for UI functions
    currentGameState = state;

    // If the game reached GameOver, auto-restart to next game after short delay
    if (state.step === "GameOver") 
    {
        setTimeout(() => 
        {
            // Restart engine to next game
            restart();
        }, 3000);
    }
    
    // Redraw blocking arrows
    setTimeout(drawBlockingArrows, 100);
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
