use std::collections::HashMap;
use std::net::SocketAddr;

use axum::{
    routing::{get},
    extract::Path,
    response::{IntoResponse, Html},
    Router};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use minijinja::render;

const STORY_ARC_TEMPLATE: &str = r#"
<!doctype html>

<html lang="en">
<head>
  <meta charset="utf-8">
  <title>A Basic HTML5 Template</title>
</head>

<body>
    <h1>{{ arc.title }}</h1>

    {% for paragraph in arc.story %}
    <p>{{ paragraph }}</p>
    {% endfor %}

    <ul>
        {% for option in arc.options %}
        <li><a href="/{{ option.arc }}">{{ option.text }}</a></li>
        {% endfor %}
    <ul>
</body>
</html>
"#;

#[derive(Serialize, Deserialize)]
struct StoryOption {
    text: String,
    arc: String,
}

#[derive(Serialize, Deserialize)]
struct StoryArc {
    title: String,
    story: Vec<String>,
    options: Vec<StoryOption>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = Router::new()
        .route("/", get(lookup_index))
        .route("/:name", get(lookup_chapter));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn lookup_index() -> impl IntoResponse {
    lookup_or_404(String::from("intro")).await
}

async fn lookup_chapter(Path(arc_id): Path<String>) -> impl IntoResponse {
    lookup_or_404(arc_id).await
}

async fn lookup_or_404(arc_id: String) -> (StatusCode, Html<String>) {
    match lookup(arc_id).await {
        Some(html) => (StatusCode::OK, html),
        None => (StatusCode::NOT_FOUND, Html(String::from(""))),
    }
}

async fn lookup(arc_id: String) -> Option<Html<String>> {
    let contents = tokio::fs::read("gopher.json").await.unwrap();
    let mapping: HashMap<String, StoryArc> = serde_json::from_slice(&contents).unwrap();
    mapping.get(&arc_id).map(|arc| render!(STORY_ARC_TEMPLATE, arc => arc )).map(Html)
}
