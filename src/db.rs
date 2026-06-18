use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

pub async fn init_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS live_room_popup (
            id TEXT PRIMARY KEY,
            live_room_id TEXT NOT NULL,
            product_id TEXT NOT NULL,
            product_name TEXT NOT NULL,
            product_image TEXT,
            product_price REAL NOT NULL,
            original_price REAL,
            popup_type TEXT NOT NULL DEFAULT 'product_card',
            title TEXT,
            description TEXT,
            action_url TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS popup_display_schedule (
            id TEXT PRIMARY KEY,
            popup_id TEXT NOT NULL,
            live_room_id TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            repeat_mode TEXT NOT NULL DEFAULT 'once',
            repeat_interval_secs INTEGER,
            display_duration_secs INTEGER NOT NULL DEFAULT 10,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (popup_id) REFERENCES live_room_popup(id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_popup_live_room ON live_room_popup(live_room_id);",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_schedule_popup ON popup_display_schedule(popup_id);",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_schedule_live_room ON popup_display_schedule(live_room_id);",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS popup_stats_summary (
            popup_id TEXT PRIMARY KEY,
            live_room_id TEXT NOT NULL,
            impression_count INTEGER NOT NULL DEFAULT 0,
            click_count INTEGER NOT NULL DEFAULT 0,
            last_impression_at TEXT,
            last_click_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS popup_stats_daily (
            popup_id TEXT NOT NULL,
            live_room_id TEXT NOT NULL,
            stat_date TEXT NOT NULL,
            impression_count INTEGER NOT NULL DEFAULT 0,
            click_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            PRIMARY KEY (popup_id, stat_date)
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_stats_summary_live_room ON popup_stats_summary(live_room_id);",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_stats_daily_live_room_date ON popup_stats_daily(live_room_id, stat_date);",
    )
    .execute(pool)
    .await?;

    Ok(())
}
