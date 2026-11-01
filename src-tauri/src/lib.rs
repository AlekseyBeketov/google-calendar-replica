//! gcal-app — Tauri 2 shell for the Google Calendar replica.
//!
//! Owns a single [`AppStore`] (SQLite) shared behind a [`std::sync::Mutex`] and
//! exposes the API surface from `replica/architecture.md` as IPC commands.
//!
//! All commands are synchronous: `rusqlite::Connection` is `Send` but not
//! `Sync`, so serializing through a std mutex is correct and avoids pulling in
//! an async runtime. IPC pool blocking is acceptable at this scale (single
//! local user, sub-millisecond queries).

use std::sync::Mutex;

use gcal_core::events::EventDetail;
use gcal_core::models::{
    Calendar, CalendarInput, Event, EventInput, EventInstance, EventPatch, Scope, Task,
    TaskInput, TaskPatch,
};
use gcal_core::store::AppStore;
use tauri::{AppHandle, Manager, State};

/// Shared application state: the crate-pure store behind a mutex.
pub struct AppState {
    pub db: Mutex<AppStore>,
}

/// Map gcal-core domain errors onto serializable command errors.
fn wrap<T>(r: gcal_core::Result<T>) -> Result<T, String> {
    r.map_err(|e| e.to_string())
}

// ---- calendars ----

#[tauri::command]
fn get_calendars(state: State<'_, AppState>) -> Result<Vec<Calendar>, String> {
    wrap(state.db.lock().unwrap().get_calendars())
}

#[tauri::command]
fn add_calendar(
    state: State<'_, AppState>,
    input: CalendarInput,
) -> Result<Calendar, String> {
    wrap(state.db.lock().unwrap().add_calendar(input))
}

#[tauri::command]
fn set_calendar_visible(
    state: State<'_, AppState>,
    id: String,
    visible: bool,
) -> Result<(), String> {
    wrap(state.db.lock().unwrap().set_calendar_visible(&id, visible))
}

// ---- events ----

#[tauri::command]
fn list_events_range(
    state: State<'_, AppState>,
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<EventInstance>, String> {
    wrap(state.db.lock().unwrap().list_events_range(start, end))
}

#[tauri::command]
fn get_event(state: State<'_, AppState>, id: String) -> Result<EventDetail, String> {
    wrap(state.db.lock().unwrap().get_event(&id))
}

#[tauri::command]
fn create_event(
    state: State<'_, AppState>,
    input: EventInput,
) -> Result<Event, String> {
    wrap(state.db.lock().unwrap().create_event(input))
}

#[tauri::command]
fn update_event(
    state: State<'_, AppState>,
    id: String,
    patch: EventPatch,
    scope: Scope,
    occurrence_date: Option<chrono::NaiveDate>,
) -> Result<Event, String> {
    wrap(state
        .db
        .lock()
        .unwrap()
        .update_event(&id, patch, scope, occurrence_date))
}

#[tauri::command]
fn delete_event(
    state: State<'_, AppState>,
    id: String,
    scope: Scope,
    occurrence_date: Option<chrono::NaiveDate>,
) -> Result<(), String> {
    wrap(state.db.lock().unwrap().delete_event(&id, scope, occurrence_date))
}

// ---- tasks ----

#[tauri::command]
fn list_tasks(state: State<'_, AppState>, only_active: bool) -> Result<Vec<Task>, String> {
    wrap(state.db.lock().unwrap().list_tasks(only_active))
}

#[tauri::command]
fn create_task(
    state: State<'_, AppState>,
    input: TaskInput,
) -> Result<Task, String> {
    wrap(state.db.lock().unwrap().create_task(input))
}

#[tauri::command]
fn update_task(
    state: State<'_, AppState>,
    id: String,
    patch: TaskPatch,
) -> Result<Task, String> {
    wrap(state.db.lock().unwrap().update_task(&id, patch))
}

#[tauri::command]
fn set_task_completed(
    state: State<'_, AppState>,
    id: String,
    completed: bool,
) -> Result<Task, String> {
    wrap(state.db.lock().unwrap().set_task_completed(&id, completed))
}

#[tauri::command]
fn delete_task(state: State<'_, AppState>, id: String) -> Result<(), String> {
    wrap(state.db.lock().unwrap().delete_task(&id))
}

/// Open (or create) the DB at `<app_data_dir>/gcal.db`, creating the dir if
/// needed. If the file already exists the store opens/migrates it in place.
fn init_db(app: &AppHandle) -> Result<AppState, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let store = AppStore::open(&dir.join("gcal.db")).map_err(|e| e.to_string())?;
    Ok(AppState { db: Mutex::new(store) })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = init_db(app.handle())?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_calendars,
            add_calendar,
            set_calendar_visible,
            list_events_range,
            get_event,
            create_event,
            update_event,
            delete_event,
            list_tasks,
            create_task,
            update_task,
            set_task_completed,
            delete_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
