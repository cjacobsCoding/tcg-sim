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
        .route("/cards/*file", get(serve_card));

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    let listener = TcpListener::bind(addr).await.unwrap();

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
async fn index() -> impl IntoResponse {
    match tokio::fs::read_to_string("web/index.html").await {
        Ok(s) => ([("content-type", "text/html; charset=utf-8")], s).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

async fn js() -> impl IntoResponse {
    match tokio::fs::read_to_string("web/app.js").await {
        Ok(s) => ([("content-type", "application/javascript")], s).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

async fn css() -> impl IntoResponse {
    match tokio::fs::read_to_string("web/style.css").await {
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

    let path = format!("web/cards/{}", file);
    match tokio::fs::read(path).await {
        Ok(bytes) => ([("content-type", "application/octet-stream")], bytes).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}