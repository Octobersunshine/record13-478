use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LiveRoomPopup {
    pub id: String,
    pub live_room_id: String,
    pub product_id: String,
    pub product_name: String,
    pub product_image: Option<String>,
    pub product_price: f64,
    pub original_price: Option<f64>,
    pub popup_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub action_url: Option<String>,
    pub sort_order: i32,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePopupRequest {
    pub live_room_id: String,
    pub product_id: String,
    pub product_name: String,
    pub product_image: Option<String>,
    pub product_price: f64,
    pub original_price: Option<f64>,
    #[serde(default = "default_popup_type")]
    pub popup_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub action_url: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePopupRequest {
    pub product_name: Option<String>,
    pub product_image: Option<String>,
    pub product_price: Option<f64>,
    pub original_price: Option<f64>,
    pub popup_type: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub action_url: Option<String>,
    pub sort_order: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PopupDisplaySchedule {
    pub id: String,
    pub popup_id: String,
    pub live_room_id: String,
    pub start_time: String,
    pub end_time: String,
    pub repeat_mode: String,
    pub repeat_interval_secs: Option<i32>,
    pub display_duration_secs: i32,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub popup_id: String,
    pub live_room_id: String,
    pub start_time: String,
    pub end_time: String,
    #[serde(default = "default_repeat_mode")]
    pub repeat_mode: String,
    pub repeat_interval_secs: Option<i32>,
    #[serde(default = "default_display_duration")]
    pub display_duration_secs: i32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduleRequest {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub repeat_mode: Option<String>,
    pub repeat_interval_secs: Option<i32>,
    pub display_duration_secs: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ScheduleConflictInfo {
    pub conflicting_schedule_id: String,
    pub popup_id: String,
    pub popup_name: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Serialize)]
pub struct ScheduleConflictError {
    pub message: String,
    pub conflicts: Vec<ScheduleConflictInfo>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "success".to_string(),
            data: Some(data),
        }
    }

    pub fn error(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }
}

fn default_popup_type() -> String {
    "product_card".to_string()
}

fn default_repeat_mode() -> String {
    "once".to_string()
}

fn default_display_duration() -> i32 {
    10
}

fn default_enabled() -> bool {
    true
}
