use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::{ApiResponse, CreateScheduleRequest, PopupDisplaySchedule, UpdateScheduleRequest};

#[derive(Debug, Deserialize)]
pub struct ListScheduleQuery {
    pub live_room_id: Option<String>,
    pub popup_id: Option<String>,
}

pub async fn create_schedule(
    State(pool): State<SqlitePool>,
    Json(req): Json<CreateScheduleRequest>,
) -> impl IntoResponse {
    let popup_exists: i32 = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM live_room_popup WHERE id = ?)")
        .bind(&req.popup_id)
        .fetch_one(&pool)
        .await
        .unwrap_or(0);

    if popup_exists == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(400, "Popup not found")),
        )
            .into_response();
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let result = sqlx::query_as::<_, PopupDisplaySchedule>(
        r#"
        INSERT INTO popup_display_schedule
            (id, popup_id, live_room_id, start_time, end_time,
             repeat_mode, repeat_interval_secs, display_duration_secs,
             enabled, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(&req.popup_id)
    .bind(&req.live_room_id)
    .bind(&req.start_time)
    .bind(&req.end_time)
    .bind(&req.repeat_mode)
    .bind(req.repeat_interval_secs)
    .bind(req.display_duration_secs)
    .bind(req.enabled)
    .bind(&now)
    .bind(&now)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(schedule) => (StatusCode::CREATED, Json(ApiResponse::success(schedule))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}

pub async fn list_schedules(
    State(pool): State<SqlitePool>,
    Query(query): Query<ListScheduleQuery>,
) -> impl IntoResponse {
    let result = if let Some(popup_id) = query.popup_id {
        sqlx::query_as::<_, PopupDisplaySchedule>(
            "SELECT * FROM popup_display_schedule WHERE popup_id = ? ORDER BY start_time ASC",
        )
        .bind(&popup_id)
        .fetch_all(&pool)
        .await
    } else if let Some(live_room_id) = query.live_room_id {
        sqlx::query_as::<_, PopupDisplaySchedule>(
            "SELECT * FROM popup_display_schedule WHERE live_room_id = ? ORDER BY start_time ASC",
        )
        .bind(&live_room_id)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as::<_, PopupDisplaySchedule>(
            "SELECT * FROM popup_display_schedule ORDER BY start_time ASC",
        )
        .fetch_all(&pool)
        .await
    };

    match result {
        Ok(schedules) => Json(ApiResponse::success(schedules)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}

pub async fn get_schedule(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, PopupDisplaySchedule>(
        "SELECT * FROM popup_display_schedule WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(schedule)) => Json(ApiResponse::success(schedule)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(404, "Schedule not found")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}

pub async fn update_schedule(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    Json(req): Json<UpdateScheduleRequest>,
) -> impl IntoResponse {
    let existing = sqlx::query_as::<_, PopupDisplaySchedule>(
        "SELECT * FROM popup_display_schedule WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await;

    let existing = match existing {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(404, "Schedule not found")),
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
    let start_time = req.start_time.unwrap_or(existing.start_time);
    let end_time = req.end_time.unwrap_or(existing.end_time);
    let repeat_mode = req.repeat_mode.unwrap_or(existing.repeat_mode);
    let repeat_interval_secs = req.repeat_interval_secs.or(existing.repeat_interval_secs);
    let display_duration_secs = req.display_duration_secs.unwrap_or(existing.display_duration_secs);
    let enabled = req.enabled.unwrap_or(existing.enabled);

    let result = sqlx::query_as::<_, PopupDisplaySchedule>(
        r#"
        UPDATE popup_display_schedule SET
            start_time = ?, end_time = ?, repeat_mode = ?,
            repeat_interval_secs = ?, display_duration_secs = ?,
            enabled = ?, updated_at = ?
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(&start_time)
    .bind(&end_time)
    .bind(&repeat_mode)
    .bind(repeat_interval_secs)
    .bind(display_duration_secs)
    .bind(enabled)
    .bind(&now)
    .bind(&id)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(schedule) => Json(ApiResponse::success(schedule)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}

pub async fn delete_schedule(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM popup_display_schedule WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            Json(ApiResponse::success(serde_json::json!({"deleted": true}))).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(404, "Schedule not found")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}
