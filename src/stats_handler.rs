use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::{FromRow, SqlitePool};

use crate::models::{
    ApiResponse, DailyStatsQuery, LiveRoomPopupStatsRow, PopupStatsDaily, PopupStatsSummary,
    TrackEventRequest, TrackEventResult,
};

fn today_date() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

fn compute_ctr(impressions: i64, clicks: i64) -> Option<f64> {
    if impressions > 0 {
        Some((clicks as f64 / impressions as f64) * 100.0)
    } else {
        None
    }
}

async fn upsert_summary_impression(
    pool: &SqlitePool,
    popup_id: &str,
    live_room_id: &str,
    now: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO popup_stats_summary
            (popup_id, live_room_id, impression_count, click_count, last_impression_at, created_at, updated_at)
        VALUES (?, ?, 1, 0, ?, ?, ?)
        ON CONFLICT(popup_id) DO UPDATE SET
            impression_count = impression_count + 1,
            last_impression_at = excluded.last_impression_at,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(popup_id)
    .bind(live_room_id)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    let count: (i64,) = sqlx::query_as(
        "SELECT impression_count FROM popup_stats_summary WHERE popup_id = ?",
    )
    .bind(popup_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

async fn upsert_summary_click(
    pool: &SqlitePool,
    popup_id: &str,
    live_room_id: &str,
    now: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO popup_stats_summary
            (popup_id, live_room_id, impression_count, click_count, last_click_at, created_at, updated_at)
        VALUES (?, ?, 0, 1, ?, ?, ?)
        ON CONFLICT(popup_id) DO UPDATE SET
            click_count = click_count + 1,
            last_click_at = excluded.last_click_at,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(popup_id)
    .bind(live_room_id)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    let count: (i64,) = sqlx::query_as(
        "SELECT click_count FROM popup_stats_summary WHERE popup_id = ?",
    )
    .bind(popup_id)
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

async fn upsert_daily_impression(
    pool: &SqlitePool,
    popup_id: &str,
    live_room_id: &str,
    stat_date: &str,
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO popup_stats_daily
            (popup_id, live_room_id, stat_date, impression_count, click_count, created_at, updated_at)
        VALUES (?, ?, ?, 1, 0, ?, ?)
        ON CONFLICT(popup_id, stat_date) DO UPDATE SET
            impression_count = impression_count + 1,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(popup_id)
    .bind(live_room_id)
    .bind(stat_date)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

async fn upsert_daily_click(
    pool: &SqlitePool,
    popup_id: &str,
    live_room_id: &str,
    stat_date: &str,
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO popup_stats_daily
            (popup_id, live_room_id, stat_date, impression_count, click_count, created_at, updated_at)
        VALUES (?, ?, ?, 0, 1, ?, ?)
        ON CONFLICT(popup_id, stat_date) DO UPDATE SET
            click_count = click_count + 1,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(popup_id)
    .bind(live_room_id)
    .bind(stat_date)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn track_impression(
    State(pool): State<SqlitePool>,
    Json(req): Json<TrackEventRequest>,
) -> impl IntoResponse {
    let popup_exists: i32 =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM live_room_popup WHERE id = ?)")
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

    let now = chrono::Utc::now().to_rfc3339();
    let stat_date = today_date();

    let result = async {
        let total =
            upsert_summary_impression(&pool, &req.popup_id, &req.live_room_id, &now).await?;
        upsert_daily_impression(&pool, &req.popup_id, &req.live_room_id, &stat_date, &now)
            .await?;
        Ok::<i64, sqlx::Error>(total)
    }
    .await;

    match result {
        Ok(total_count) => (
            StatusCode::OK,
            Json(ApiResponse::success(TrackEventResult {
                popup_id: req.popup_id,
                event_type: "impression".to_string(),
                total_count,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}

pub async fn track_click(
    State(pool): State<SqlitePool>,
    Json(req): Json<TrackEventRequest>,
) -> impl IntoResponse {
    let popup_exists: i32 =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM live_room_popup WHERE id = ?)")
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

    let now = chrono::Utc::now().to_rfc3339();
    let stat_date = today_date();

    let result = async {
        let total = upsert_summary_click(&pool, &req.popup_id, &req.live_room_id, &now).await?;
        upsert_daily_click(&pool, &req.popup_id, &req.live_room_id, &stat_date, &now).await?;
        Ok::<i64, sqlx::Error>(total)
    }
    .await;

    match result {
        Ok(total_count) => (
            StatusCode::OK,
            Json(ApiResponse::success(TrackEventResult {
                popup_id: req.popup_id,
                event_type: "click".to_string(),
                total_count,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}

#[derive(Debug, FromRow)]
struct SummaryRow {
    pub popup_id: String,
    pub live_room_id: String,
    pub impression_count: i64,
    pub click_count: i64,
    pub last_impression_at: Option<String>,
    pub last_click_at: Option<String>,
    pub updated_at: String,
}

fn summary_from_row(row: SummaryRow) -> PopupStatsSummary {
    PopupStatsSummary {
        popup_id: row.popup_id.clone(),
        live_room_id: row.live_room_id,
        impression_count: row.impression_count,
        click_count: row.click_count,
        ctr: compute_ctr(row.impression_count, row.click_count),
        last_impression_at: row.last_impression_at,
        last_click_at: row.last_click_at,
        updated_at: row.updated_at,
    }
}

pub async fn get_popup_stats(
    State(pool): State<SqlitePool>,
    Path(popup_id): Path<String>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, SummaryRow>(
        "SELECT popup_id, live_room_id, impression_count, click_count,
                last_impression_at, last_click_at, updated_at
         FROM popup_stats_summary WHERE popup_id = ?",
    )
    .bind(&popup_id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let popup_exists: i32 =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM live_room_popup WHERE id = ?)")
                    .bind(&popup_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap_or(0);
            if popup_exists == 0 {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<()>::error(404, "Popup not found")),
                )
                    .into_response();
            }
            Json(ApiResponse::success(summary_from_row(row))).into_response()
        }
        Ok(None) => {
            let popup_exists: i32 =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM live_room_popup WHERE id = ?)")
                    .bind(&popup_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap_or(0);
            if popup_exists == 0 {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<()>::error(404, "Popup not found")),
                )
                    .into_response();
            }
            let lr: (String,) =
                match sqlx::query_as("SELECT live_room_id FROM live_room_popup WHERE id = ?")
                    .bind(&popup_id)
                    .fetch_one(&pool)
                    .await
                {
                    Ok(x) => x,
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::<()>::error(500, e.to_string())),
                        )
                            .into_response()
                    }
                };
            let zero = PopupStatsSummary {
                popup_id: popup_id.clone(),
                live_room_id: lr.0,
                impression_count: 0,
                click_count: 0,
                ctr: None,
                last_impression_at: None,
                last_click_at: None,
                updated_at: chrono::Utc::now().to_rfc3339(),
            };
            Json(ApiResponse::success(zero)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}

pub async fn list_live_room_stats(
    State(pool): State<SqlitePool>,
    Path(live_room_id): Path<String>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, LiveRoomPopupStatsRow>(
        r#"
        SELECT
            p.id, p.live_room_id, p.product_id, p.product_name,
            p.product_image, p.product_price, p.original_price,
            p.popup_type, p.title, p.description, p.action_url,
            p.sort_order, p.enabled,
            COALESCE(s.impression_count, 0) AS impression_count,
            COALESCE(s.click_count, 0) AS click_count,
            CASE
                WHEN COALESCE(s.impression_count, 0) > 0
                THEN (COALESCE(s.click_count, 0) * 100.0 / s.impression_count)
                ELSE NULL
            END AS ctr
        FROM live_room_popup p
        LEFT JOIN popup_stats_summary s ON s.popup_id = p.id
        WHERE p.live_room_id = ?
        ORDER BY p.sort_order ASC, p.created_at DESC
        "#,
    )
    .bind(&live_room_id)
    .fetch_all(&pool)
    .await;

    match result {
        Ok(rows) => Json(ApiResponse::success(rows)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}

#[derive(Debug, FromRow)]
struct DailyRow {
    pub popup_id: String,
    pub live_room_id: String,
    pub stat_date: String,
    pub impression_count: i64,
    pub click_count: i64,
    pub updated_at: String,
}

pub async fn list_daily_stats(
    State(pool): State<SqlitePool>,
    Query(query): Query<DailyStatsQuery>,
) -> impl IntoResponse {
    if query.popup_id.is_none() && query.live_room_id.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                400,
                "Either popup_id or live_room_id is required",
            )),
        )
            .into_response();
    }

    let mut sql = String::from(
        "SELECT popup_id, live_room_id, stat_date, impression_count, click_count, updated_at \
         FROM popup_stats_daily WHERE 1=1",
    );
    let mut binds: Vec<&str> = Vec::new();

    if let Some(pid) = query.popup_id.as_deref() {
        sql.push_str(" AND popup_id = ?");
        binds.push(pid);
    }
    if let Some(lr) = query.live_room_id.as_deref() {
        sql.push_str(" AND live_room_id = ?");
        binds.push(lr);
    }
    if let Some(sd) = query.start_date.as_deref() {
        sql.push_str(" AND stat_date >= ?");
        binds.push(sd);
    }
    if let Some(ed) = query.end_date.as_deref() {
        sql.push_str(" AND stat_date <= ?");
        binds.push(ed);
    }
    sql.push_str(" ORDER BY stat_date ASC");

    let mut q = sqlx::query_as::<_, DailyRow>(&sql);
    for b in &binds {
        q = q.bind(*b);
    }

    let result = q.fetch_all(&pool).await;

    match result {
        Ok(rows) => {
            let out: Vec<PopupStatsDaily> = rows
                .into_iter()
                .map(|r| PopupStatsDaily {
                    popup_id: r.popup_id,
                    live_room_id: r.live_room_id,
                    stat_date: r.stat_date,
                    impression_count: r.impression_count,
                    click_count: r.click_count,
                    ctr: compute_ctr(r.impression_count, r.click_count),
                    updated_at: r.updated_at,
                })
                .collect();
            Json(ApiResponse::success(out)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(500, e.to_string())),
        )
            .into_response(),
    }
}
