use axum::{routing::{get, post}, Json, Router};
use std::sync::{Arc, Mutex};
use engine::{GameState, GameStep};
use axum::extract::Extension;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use axum::http::StatusCode;
use axum::extract::Path;
use axum::response::IntoResponse;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::signal;
use std::path::PathBuf;
use socket2::{Socket, Domain, Type};
use serde::{Deserialize, Serialize};

/// Find the web directory relative to the project root
fn find_web_dir() -> PathBuf {
    let mut current = std::env::current_dir().expect("Failed to get current directory");
    
    // If we're in the target directory or deeper, go up to find project root
    loop {
        let web_path = current.join("web");
        if web_path.exists() && web_path.is_dir() {
            return current;
        }
        
        if !current.pop() {
            break;
        }
    }
    
    // Fallback to current directory
    std::env::current_dir().expect("Failed to get current directory")
}

fn web_path(file: &str) -> String {
    let web_dir = find_web_dir();
    web_dir.join(file).to_string_lossy().to_string()
}

fn create_listener_with_reuse(addr: &std::net::SocketAddr) -> std::io::Result<TcpListener> {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;
    socket.set_reuse_address(true)?;
    socket.bind(&(*addr).into())?;
    socket.listen(128)?;
    
    let std_listener = std::net::TcpListener::from(socket);
    std_listener.set_nonblocking(true)?;
    Ok(TcpListener::from_std(std_listener)?)
}

#[cfg(unix)]
fn kill_process_on_port(port: u16) {
    // Use lsof to find the process using the port and kill it
    let output = std::process::Command::new("lsof")
        .args(&["-ti", &format!(":{}", port)])
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(pid_str) = String::from_utf8(output.stdout) {
                let pid = pid_str.trim().parse::<u32>();
                if let Ok(pid) = pid {
                    let _ = std::process::Command::new("kill")
                        .arg("-9")
                        .arg(pid.to_string())
                        .output();
                    eprintln!("Killed existing process (PID: {}) on port {}", pid, port);
                }
            }
        }
    }
}

#[cfg(not(unix))]
fn kill_process_on_port(_port: u16) {
    // Windows would need a different approach (netstat + taskkill)
    // For now, just inform the user
    eprintln!("Port is already in use. Please close the existing process manually.");
}

#[tokio::main]
async fn main()
{
    let game = Arc::new(Mutex::new(GameState::new_default()));
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    // API routes
    let api = Router::new()
        .route("/state", get(get_state))
        .route("/step", post(post_step))
        .route("/turn", post(post_turn))
        .route("/game", post(post_game))
        .route("/deck", post(post_deck))
        .route("/all", post(post_all))
        .route("/restart", post(post_restart))
        .route("/declare-attackers", post(post_declare_attackers))
        .route("/declare-blockers", post(post_declare_blockers))
        .route("/music-list", get(get_music_list))
        .route("/shutdown", post({
            let flag = shutdown_flag.clone();
            move || {
                let flag = flag.clone();
                async move {
                    flag.store(true, Ordering::Relaxed);
                    StatusCode::OK
                }
            }
        }))
        .layer(Extension(game.clone()));

    // Static routes for the web/ directory (simple handlers)
    let app = Router::new()
        .nest("/api", api)
        .route("/", get(index))
        .route("/app.js", get(js))
        .route("/style.css", get(css))
        .route("/cards/*file", get(serve_card))
        .route("/music/*file", get(serve_music));

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    
    // Try to create socket with SO_REUSEADDR enabled to allow immediate port reuse
    // If the port is already in use, kill the existing process and retry
    let listener = {
        let mut listener = None;
        for attempt in 1..=3 {
            match create_listener_with_reuse(&addr) {
                Ok(l) => {
                    listener = Some(l);
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse && attempt < 3 => {
                    if attempt == 1 {
                        eprintln!("Port 3000 is already in use. Attempting to kill the existing process...");
                        kill_process_on_port(3000);
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    } else {
                        eprintln!("Port 3000 is still in use, retrying ({}/3)...", attempt);
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                    continue;
                }
                Err(e) => {
                    eprintln!("Failed to bind to port 3000: {}", e);
                    return;
                }
            }
        }
        listener.expect("Failed to create listener")
    };

    println!("Server running at http://{}", addr);
    println!("Press Ctrl+C to stop the server, or visit http://{}:3000 and click 'Stop Server'", addr.ip());

    // Spawn a background task to check for shutdown flag
    let shutdown_flag_clone = shutdown_flag.clone();
    tokio::spawn(async move {
        loop {
            if shutdown_flag_clone.load(Ordering::Relaxed) {
                println!("\nShutdown signal received, exiting gracefully...");
                std::process::exit(0);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    // Build the server and run it
    let server = axum::serve(listener, app);

    // Handle Ctrl+C (SIGINT) for graceful shutdown
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                eprintln!("Server error: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            println!("\nReceived Ctrl+C, shutting down gracefully...");
        }
    }
}
#[derive(Deserialize, Serialize)]
pub struct DeclareAttackersRequest {
    pub attacking_indices: Vec<usize>,
}

#[derive(Deserialize, Serialize)]
pub struct DeclareBlockersRequest {
    pub blocking_map: std::collections::HashMap<usize, usize>, // blocker index -> attacker index
}

async fn post_declare_attackers(
    Extension(game): Extension<Arc<Mutex<GameState>>>,
    Json(payload): Json<DeclareAttackersRequest>,
) -> Json<GameState> {
    let mut g = game.lock().unwrap();
    g.attacking_creatures = payload.attacking_indices;
    
    // Tap all attacking creatures
    let attacking_to_tap = g.attacking_creatures.clone();
    if let Some(battlefield) = g.zones_mut().get_mut(&engine::Zone::Battlefield) {
        for idx in attacking_to_tap {
            if idx < battlefield.len() {
                engine::tappable::set_tapped(&mut battlefield[idx], true);
            }
        }
    }
    
    g.step = GameStep::DeclareBlockers;
    Json(g.clone())
}

async fn post_declare_blockers(
    Extension(game): Extension<Arc<Mutex<GameState>>>,
    Json(payload): Json<DeclareBlockersRequest>,
) -> Json<GameState> {
    let mut g = game.lock().unwrap();
    g.blocking_map = payload.blocking_map;
    g.step = GameStep::AssignDamage;
    Json(g.clone())
}
async fn get_state(Extension(game): Extension<Arc<Mutex<GameState>>>) -> Json<GameState> {
    Json(game.lock().unwrap().clone())
}

async fn post_step(Extension(game): Extension<Arc<Mutex<GameState>>>) -> Json<GameState> {
    let mut g = game.lock().unwrap();
    g.step();
    Json(g.clone())
}

async fn post_turn(Extension(game): Extension<Arc<Mutex<GameState>>>) -> Json<GameState> {
    let mut g = game.lock().unwrap();
    let start_turn = g.turns;
    while g.turns == start_turn && g.step != GameStep::GameOver {
        g.step();
    }
    Json(g.clone())
}

async fn post_game(Extension(game): Extension<Arc<Mutex<GameState>>>) -> Json<GameState> {
    let mut g = game.lock().unwrap();
    while g.step != GameStep::GameOver {
        g.step();
    }
    Json(g.clone())
}

async fn post_deck(Extension(game): Extension<Arc<Mutex<GameState>>>) -> Json<serde_json::Value> {
    // Run 10,000 games and track average turns
    let mut total_turns = 0;
    for _ in 0..10000 {
        let mut g = GameState::new_default();
        while g.step != GameStep::GameOver {
            g.step();
        }
        total_turns += g.turns as u64;
    }
    let avg_turns = total_turns as f64 / 10000.0;
    
    let mut g = game.lock().unwrap();
    *g = GameState::new_default();
    
    serde_json::json!({
        "avg_turns": avg_turns,
        "total_games": 10000,
        "state": g.clone()
    })
    .into()
}

async fn post_all(Extension(game): Extension<Arc<Mutex<GameState>>>) -> Json<serde_json::Value> {
    // For now, same as deck - could be extended to run multiple deck configs
    let mut total_turns = 0;
    for _ in 0..10000 {
        let mut g = GameState::new_default();
        while g.step != GameStep::GameOver {
            g.step();
        }
        total_turns += g.turns as u64;
    }
    let avg_turns = total_turns as f64 / 10000.0;
    
    let mut g = game.lock().unwrap();
    *g = GameState::new_default();
    
    serde_json::json!({
        "avg_turns": avg_turns,
        "total_games": 10000,
        "state": g.clone()
    })
    .into()
}

async fn post_restart(Extension(game): Extension<Arc<Mutex<GameState>>>) -> Json<GameState> {
    let mut g = game.lock().unwrap();
    *g = GameState::new_default();
    Json(g.clone())
}

async fn get_music_list() -> Json<serde_json::Value> {
    let mut music_files = Vec::new();
    let music_dir = format!("{}/web/music", find_web_dir().to_string_lossy());
    
    // List all files in the music directory
    if let Ok(entries) = std::fs::read_dir(&music_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        // Only include audio files
                        if file_name.ends_with(".mp3") || file_name.ends_with(".wav") || 
                           file_name.ends_with(".ogg") || file_name.ends_with(".flac") ||
                           file_name.ends_with(".m4a") || file_name.ends_with(".aac") {
                            music_files.push(file_name.to_string());
                        }
                    }
                }
            }
        }
    }
    
    Json(serde_json::json!({
        "files": music_files
    }))
}

async fn index() -> impl IntoResponse {
    match tokio::fs::read_to_string(web_path("web/index.html")).await {
        Ok(s) => ([("content-type", "text/html; charset=utf-8")], s).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

async fn js() -> impl IntoResponse {
    match tokio::fs::read_to_string(web_path("web/app.js")).await {
        Ok(s) => ([("content-type", "application/javascript")], s).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

async fn css() -> impl IntoResponse {
    match tokio::fs::read_to_string(web_path("web/style.css")).await {
        Ok(s) => ([("content-type", "text/css")], s).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

async fn serve_card(Path(file): Path<String>) -> impl IntoResponse 
{
    if file.contains("..") 
    {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }

    match tokio::fs::read(web_path(&format!("web/cards/{}", file))).await {
        Ok(bytes) => ([("content-type", "application/octet-stream")], bytes).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

async fn serve_music(Path(file): Path<String>) -> impl IntoResponse 
{
    if file.contains("..") 
    {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }

    let content_type = if file.ends_with(".mp3") {
        "audio/mpeg"
    } else if file.ends_with(".wav") {
        "audio/wav"
    } else if file.ends_with(".ogg") || file.ends_with(".oga") {
        "audio/ogg"
    } else if file.ends_with(".flac") {
        "audio/flac"
    } else if file.ends_with(".m4a") || file.ends_with(".aac") {
        "audio/aac"
    } else {
        "audio/mpeg"
    };
    
    match tokio::fs::read(web_path(&format!("web/music/{}", file))).await {
        Ok(bytes) => ([("content-type", content_type)], bytes).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}