// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use async_std::sync::{Mutex, RwLock};
use chrono::prelude::*;
use std::{
    collections::BTreeSet,
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::async_runtime;

pub mod settings;
pub mod timecard;
pub mod wayland;

struct AppState {
    // data_dir: PathBuf,
    // app_dir: PathBuf,
    logs_dir: PathBuf,
    // config_file: PathBuf,

    event_log: RwLock<timecard::EventLog>,
    settings: Mutex<settings::Settings>,
    app_handle: RwLock<Option<tauri::AppHandle>>,
}

#[tauri::command]
async fn clock_in(
    clock: timecard::ClockType,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<timecard::EventLog, String> {
    let mut event_log = state.event_log.write().await;

    event_log.add_event(timecard::Event::clock_in(clock));
    event_log.save().await.map_err(|err| err.to_string())?;

    Ok(event_log.clone())
}

#[tauri::command]
async fn clock_out(
    clock: timecard::ClockType,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<timecard::EventLog, String> {
    let mut event_log = state.event_log.write().await;

    event_log.add_event(timecard::Event::clock_out(clock));
    event_log.save().await.map_err(|err| err.to_string())?;

    Ok(event_log.clone())
}

#[tauri::command]
async fn set_tasks(
    tasks: BTreeSet<timecard::TaskID>,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<timecard::EventLog, String> {
    let mut event_log = state.event_log.write().await;

    event_log.add_event(timecard::Event::tasks(tasks));
    event_log.save().await.map_err(|err| err.to_string())?;

    Ok(event_log.clone())
}

#[tauri::command]
async fn get_state(state: tauri::State<'_, Arc<AppState>>) -> Result<timecard::EventLog, ()> {
    let event_log = state.event_log.read().await;
    Ok(event_log.clone())
}

async fn background_loop(app_state: Arc<AppState>) {
    let loop_time = std::time::Duration::from_secs(60);

    loop {
        let start = std::time::Instant::now();

        update_settings(&app_state)
            .await
            .unwrap_or_else(|e| println!("error updating settings: {}", e));
        update_event_log(&app_state)
            .await
            .unwrap_or_else(|e| println!("error updating event log: {}", e));

        async_std::task::sleep(loop_time - start.elapsed()).await;
    }
}

async fn update_settings(app_state: &AppState) -> Result<(), Box<dyn Error>> {
    let mut settings = app_state.settings.lock().await;

    let current_date = Local::now().date_naive();

    if settings.current_date != current_date {
        let mut event_log = app_state.event_log.write().await;

        let new_event_log = timecard::EventLog::new(
            log_file_for_date(&app_state.logs_dir, current_date).into(),
            event_log.get_state(),
        );
        new_event_log.save().await?;
        *event_log = new_event_log;

        settings.current_date = current_date;
        settings.save().await?;
    }

    Ok(())
}

async fn update_event_log(app_state: &AppState) -> Result<(), Box<dyn Error>> {
    let mut event_log = app_state.event_log.write().await;

    event_log.refresh_active().await
}

fn main() {
    let data_dir = dirs::data_dir().expect("could not find user data directory");
    let app_dir = data_dir.join("work-warden");
    let logs_dir = app_dir.join("logs");
    let config_file = app_dir.join("config.json");

    std::fs::create_dir_all(&app_dir).expect("could not create app directory");
    std::fs::create_dir_all(&logs_dir).expect("could not create timecard logs directory");

    let current_date = Local::now().date_naive();
    let log_file = log_file_for_date(&logs_dir, current_date);

    let mut event_log = if log_file.exists() {
        async_runtime::block_on(timecard::EventLog::load(log_file.into()))
            .expect("couldn't load initial time card")
    } else {
        timecard::EventLog::new(log_file.into(), timecard::State::default())
    };

    async_runtime::block_on(event_log.force_active()).expect("error activating event log");

    let settings = async_runtime::block_on(settings::Settings::load_or_new(config_file.clone().into()))
        .expect("error loading/initializing settings");

    let app_state = Arc::new(AppState {
        // data_dir,
        // app_dir,
        logs_dir,
        // config_file,

        event_log: RwLock::new(event_log),
        settings: Mutex::new(settings),
        app_handle: RwLock::new(None),
    });

    let app = tauri::Builder::default()
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            clock_in, clock_out, set_tasks, get_state
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    async_runtime::block_on(async {
        let mut app_handle = app_state.app_handle.write().await;
        *app_handle = Some(app.handle());
    });

    let app_state_background = app_state.clone();
    async_runtime::spawn(background_loop(app_state_background));

    wayland::listen_idle(move |idle| {
        async_runtime::block_on(async {
            let mut event_log = app_state.event_log.write().await;

            if idle {
                event_log.add_event(timecard::Event::idle());
                println!("idle");
            } else {
                event_log.add_event(timecard::Event::active());
                println!("active");
            }
        });
    });

    app.run(|_, _| {});
}

fn log_file_for_date(logs_dir: &Path, date: NaiveDate) -> PathBuf {
    logs_dir.join(format!(
        "{}-{}-{}.log.json",
        date.year(),
        date.month(),
        date.day()
    ))
}
