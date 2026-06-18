use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::{ApiResponse, CreatePopupRequest, LiveRoomPopup, UpdatePopupRequest};

#[derive(Debug, Deserialize)]
pub struct ListPopupQuery {
    pub live_room_id: String,
}

pub async fn create_popup(
    State(pool): State<SqlitePool>,
    Json(req): Json<CreatePopupRequest>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let result = sqlx::query_as::<_, LiveRoomPopup>(
        r#"
        INSERT INTO live_room_popup
            (id, live_room_id, product_id, product_name, product_image,
             product_price, original_price, popup_type, title, description,
             action_url, sort_order, enabled, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(&req.live_room_id)
    .bind(&req.product_id)
    .bind(&req.product_name)
    .bind(&req.product_image)
    .bind(req.product_price)
    .bind(req.original_price)
    .bind(&req.popup_type)
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.action_url)
    .bind(req.sort_order)
    .bind(req.enabled)
    .bind(&now)
    .bind(&now)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(popup) => (StatusCode::CREATED, Json(ApiResponse::success(popup))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}

pub async fn list_popups(
    State(pool): State<SqlitePool>,
    Query(query): Query<ListPopupQuery>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, LiveRoomPopup>(
        "SELECT * FROM live_room_popup WHERE live_room_id = ? ORDER BY sort_order ASC, created_at DESC",
    )
    .bind(&query.live_room_id)
    .fetch_all(&pool)
    .await;

    match result {
        Ok(popups) => Json(ApiResponse::success(popups)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}

pub async fn get_popup(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, LiveRoomPopup>(
        "SELECT * FROM live_room_popup WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(popup)) => Json(ApiResponse::success(popup)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(404, "Popup not found")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}

pub async fn update_popup(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePopupRequest>,
) -> impl IntoResponse {
    let existing = sqlx::query_as::<_, LiveRoomPopup>(
        "SELECT * FROM live_room_popup WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await;

    let existing = match existing {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(404, "Popup not found")),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(500, e.to_string())),
            )
                .into_response()
        }
    };

    let now = chrono::Utc::now().to_rfc3339();
    let product_name = req.product_name.unwrap_or(existing.product_name);
    let product_image = req.product_image.or(existing.product_image);
    let product_price = req.product_price.unwrap_or(existing.product_price);
    let original_price = req.original_price.or(existing.original_price);
    let popup_type = req.popup_type.unwrap_or(existing.popup_type);
    let title = req.title.or(existing.title);
    let description = req.description.or(existing.description);
    let action_url = req.action_url.or(existing.action_url);
    let sort_order = req.sort_order.unwrap_or(existing.sort_order);
    let enabled = req.enabled.unwrap_or(existing.enabled);

    let result = sqlx::query_as::<_, LiveRoomPopup>(
        r#"
        UPDATE live_room_popup SET
            product_name = ?, product_image = ?, product_price = ?,
            original_price = ?, popup_type = ?, title = ?,
            description = ?, action_url = ?, sort_order = ?,
            enabled = ?, updated_at = ?
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(&product_name)
    .bind(&product_image)
    .bind(product_price)
    .bind(original_price)
    .bind(&popup_type)
    .bind(&title)
    .bind(&description)
    .bind(&action_url)
    .bind(sort_order)
    .bind(enabled)
    .bind(&now)
    .bind(&id)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(popup) => Json(ApiResponse::success(popup)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}

pub async fn delete_popup(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM live_room_popup WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(ApiResponse::success(serde_json::json!({"deleted": true}))).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(404, "Popup not found")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}
