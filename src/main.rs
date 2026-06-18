mod db;
mod models;
mod popup_handler;
mod schedule_handler;
mod stats_handler;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let database_url = "sqlite:live_popup.db";
    let pool = db::init_pool(database_url)
        .await
        .expect("Failed to initialize database");

    let popup_routes = Router::new()
        .route("/", post(popup_handler::create_popup))
        .route("/", get(popup_handler::list_popups))
        .route("/{id}", get(popup_handler::get_popup))
        .route("/{id}", put(popup_handler::update_popup))
        .route("/{id}", delete(popup_handler::delete_popup));

    let schedule_routes = Router::new()
        .route("/", post(schedule_handler::create_schedule))
        .route("/", get(schedule_handler::list_schedules))
        .route("/{id}", get(schedule_handler::get_schedule))
        .route("/{id}", put(schedule_handler::update_schedule))
        .route("/{id}", delete(schedule_handler::delete_schedule));

    let stats_routes = Router::new()
        .route("/impression", post(stats_handler::track_impression))
        .route("/click", post(stats_handler::track_click))
        .route("/popup/{popup_id}", get(stats_handler::get_popup_stats))
        .route(
            "/live-room/{live_room_id}",
            get(stats_handler::list_live_room_stats),
        )
        .route("/daily", get(stats_handler::list_daily_stats));

    let app = Router::new()
        .nest("/api/popups", popup_routes)
        .nest("/api/schedules", schedule_routes)
        .nest("/api/stats", stats_routes)
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind address");

    println!("🚀 Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.expect("Server error");
}
