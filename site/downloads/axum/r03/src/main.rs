use axum::{extract::{FromRef, State}, routing::get, Router};
use std::sync::Arc;

struct Settings {
    greeting: String,
}

#[derive(Clone)]
struct AppState {
    settings: Arc<Settings>,
}

impl FromRef<AppState> for Arc<Settings> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.settings)
    }
}

async fn greeting(State(settings): State<Arc<Settings>>) -> String {
    settings.greeting.clone()
}

fn routes() -> Router<AppState> {
    Router::new().route("/", get(greeting))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let state = AppState {
        settings: Arc::new(Settings { greeting: "Hello from shared state".to_owned() }),
    };
    let app: Router = routes().with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
