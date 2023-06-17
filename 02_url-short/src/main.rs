use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    extract::Path,
    Json, Router, Extension};
use serde::{Deserialize, Serialize};
use nanoid::nanoid;

use axum_sqlite::Database;

#[derive(Deserialize)]
struct ShortenUrlPayload {
    long_url: String,
}

#[derive(Serialize)]
struct ShortenUrlResponse {
    short_id: String,
    long_url: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let extension = Database::new(":memory:").unwrap();
    extension.connection().unwrap().execute("CREATE TABLE IF NOT EXISTS shorts (short_id TEXT NOT NULL, long_url TEXT NOT NULL)", []).unwrap();
    let app = Router::new()
        .route("/:name", get(redirect))
        .route("/shorten", post(shorten_url))
        .layer(extension);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn redirect(Extension(database): Extension<Database>, Path(short_id): Path<String>) -> Redirect {
    let connection = database.connection().unwrap();
    tracing::info!("Looking up target for {} ", short_id);
    let mut statement = connection.prepare("SELECT long_url FROM shorts WHERE short_id = ?1").unwrap();
    let rows = statement.query_map([&short_id], |row| {
        let result: String = row.get("long_url").unwrap();
        Ok(result)
    }).unwrap();
    let target_url = rows.into_iter().next().unwrap().unwrap();
    tracing::info!("Redirecting from {} to {}", short_id, target_url);
    Redirect::temporary(target_url.as_ref())
}

async fn shorten_url(Extension(database): Extension<Database>, Json(payload): Json<ShortenUrlPayload>) -> impl IntoResponse {
    let id = nanoid!(10, &nanoid::alphabet::SAFE);
    tracing::info!("Create short url with id {} leading to {}", id, payload.long_url);
    let connection = database.connection().unwrap();
    connection.execute("INSERT INTO shorts (short_id, long_url) values (?1, ?2)", [&id, &payload.long_url]).unwrap();
    (StatusCode::OK, Json(ShortenUrlResponse {short_id: id, long_url: payload.long_url}))
}
