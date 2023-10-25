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
use tauri::{async_runtime, Manager};

mod notifications;
mod settings;
mod tasks;
mod timecard;
mod wayland;

struct AppState {
    // data_dir: PathBuf,
    // app_dir: PathBuf,
    logs_dir: PathBuf,
    // config_file: PathBuf,
    event_log: RwLock<timecard::EventLog>,
    settings: Mutex<settings::Settings>,
    app_handle: RwLock<Option<tauri::AppHandle>>,
    notifier: notifications::Notifier,
    task_manager: tasks::TaskManager,
}

impl AppState {
    async fn send_event_log(&self, event_log: &timecard::EventLog) {
        let app_handle = self.app_handle.read().await;

        if let Some(app_handle) = &*app_handle {
            app_handle.emit_all("timecard", event_log).unwrap();
        }
    }

    async fn send_tasks(&self, tasks: &[tasks::Task]) {
        let app_handle = self.app_handle.read().await;

        if let Some(app_handle) = &*app_handle {
            app_handle.emit_all("tasks", tasks).unwrap();
        }
    }

    async fn update_notifications(
        &self,
        event_log: &timecard::EventLog,
    ) -> Result<(), Box<dyn Error>> {
        let elapsed = event_log.elapsed();

        let (work_target, lunch_target, break_target) = {
            let settings = self.settings.lock().await;

            (
                settings.work_target,
                settings.lunch_target,
                settings.break_target,
            )
        };

        if elapsed.working && elapsed.work_time - elapsed.lunch_time > work_target {
            self.notifier
                .show_overtime(elapsed.work_time - elapsed.lunch_time - work_target)
                .await?;
        } else {
            self.notifier.clear_overtime().await;
        }

        if elapsed.on_lunch && elapsed.lunch_time > lunch_target {
            self.notifier
                .show_long_lunch(elapsed.lunch_time - lunch_target)
                .await?;
        } else {
            self.notifier.clear_long_lunch().await;
        }

        if elapsed.on_break && elapsed.break_time > break_target {
            self.notifier
                .show_long_break(elapsed.break_time - break_target)
                .await?;
        } else {
            self.notifier.clear_long_break().await;
        }

        Ok(())
    }

    async fn refresh_date(
        &self,
        send: bool,
        renew_active: bool,
        refresh_active: bool,
    ) -> Result<bool, Box<dyn Error>> {
        let mut settings = self.settings.lock().await;
        let mut event_log = self.event_log.write().await;
        let current_date = Local::now().date_naive();

        // Add idle event if needed
        let injected_idle = event_log.infer_idle();

        if settings.current_date != current_date {
            // Save changes to old event log before we create a new one
            if injected_idle {
                event_log.save().await?;
            }

            // Create new event log
            let mut new_state = event_log.get_state();
            new_state.reset_accumulations();

            let new_event_log = timecard::EventLog::new(
                log_file_for_date(&self.logs_dir, current_date).into(),
                current_date,
                new_state,
            );
            new_event_log.save().await?;
            *event_log = new_event_log;

            // Update current date in settings
            settings.current_date = current_date;
            settings.save().await?;

            // Send new event log to frontend
            if send {
                self.send_event_log(&event_log).await;
            }
        }

        // Inject new active event
        if renew_active && injected_idle {
            event_log.force_active();
        } else if refresh_active {
            event_log.refresh_active().await?;
        }

        Ok(injected_idle)
    }
}

#[tauri::command]
async fn clock_in(
    clock: timecard::ClockType,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state
        .refresh_date(false, true, false)
        .await
        .map_err(|err| err.to_string())?;

    let mut event_log = state.event_log.write().await;

    event_log.add_event(timecard::Event::clock_in(clock));
    event_log.save().await.map_err(|err| err.to_string())?;

    state.send_event_log(&event_log).await;
    state
        .update_notifications(&event_log)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
}

#[tauri::command]
async fn clock_out(
    clock: timecard::ClockType,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state
        .refresh_date(false, true, false)
        .await
        .map_err(|err| err.to_string())?;

    let mut event_log = state.event_log.write().await;

    event_log.add_event(timecard::Event::clock_out(clock));
    event_log.save().await.map_err(|err| err.to_string())?;

    state.send_event_log(&event_log).await;
    state
        .update_notifications(&event_log)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
}

#[tauri::command]
async fn set_tasks(
    tasks: BTreeSet<tasks::TaskID>,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state
        .refresh_date(false, true, false)
        .await
        .map_err(|err| err.to_string())?;

    let mut event_log = state.event_log.write().await;

    event_log.add_event(timecard::Event::tasks(tasks));
    event_log.save().await.map_err(|err| err.to_string())?;

    state.send_event_log(&event_log).await;
    state
        .update_notifications(&event_log)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_current_timecard(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<timecard::EventLog, ()> {
    let event_log = state.event_log.read().await;
    Ok(event_log.clone())
}

#[tauri::command]
async fn get_recents(state: tauri::State<'_, Arc<AppState>>) -> Result<tasks::Recents, ()> {
    Ok(state.task_manager.get_recents().await)
}

#[tauri::command]
async fn get_tasks(
    state: tauri::State<'_, Arc<AppState>>,
    ids: Vec<tasks::TaskID>,
) -> Result<Vec<tasks::Task>, ()> {
    let mut tasks = Vec::new();

    for &id in &ids {
        if let Ok(task) = state.task_manager.load_task(id).await {
            tasks.push(task);
        }
    }

    Ok(tasks)
}

#[tauri::command]
async fn put_task(
    state: tauri::State<'_, Arc<AppState>>,
    mut task: tasks::Task,
    make_recent: bool,
) -> Result<tasks::Task, String> {
    if task.id == tasks::TASK_ID_NONE {
        task.id = state
            .task_manager
            .next_task_id()
            .await
            .map_err(|e| e.to_string())?;
    }

    state
        .task_manager
        .save_task(&task)
        .await
        .map_err(|e| e.to_string())?;

    if make_recent {
        state
            .task_manager
            .make_recent(task.id, task.starred)
            .await
            .map_err(|e| e.to_string())?
    }

    Ok(task)
}

#[tauri::command]
async fn archive_task(
    state: tauri::State<'_, Arc<AppState>>,
    id: tasks::TaskID,
) -> Result<(), String> {
    state.task_manager.archive(id).await.map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn make_task_recent(
    state: tauri::State<'_, Arc<AppState>>,
    task: tasks::Task,
) -> Result<(), String> {
    state
        .task_manager
        .make_recent(task.id, task.starred)
        .await
        .map_err(|e| e.to_string())
}

async fn background_loop(app_state: Arc<AppState>) {
    let loop_time = std::time::Duration::from_secs(60);

    loop {
        let start = std::time::Instant::now();

        if let Err(e) = app_state.refresh_date(true, true, true).await {
            println!("error refreshing date: {}", e)
        }

        if let Err(e) = update_event_log(&app_state).await {
            println!("error updating event log: {}", e);
        }

        async_std::task::sleep(loop_time - start.elapsed()).await;
    }
}

async fn update_event_log(app_state: &AppState) -> Result<(), Box<dyn Error>> {
    {
        let event_log = app_state.event_log.read().await;

        app_state.update_notifications(&event_log).await?;
    }

    Ok(())
}

fn main() {
    let data_dir = dirs::data_dir().expect("could not find user data directory");
    let app_dir = data_dir.join("work-warden");
    let logs_dir = app_dir.join("logs");
    let tasks_dir = app_dir.join("tasks");
    let config_file = app_dir.join("config.json");

    std::fs::create_dir_all(&app_dir).expect("could not create app directory");
    std::fs::create_dir_all(&logs_dir).expect("could not create timecard logs directory");
    std::fs::create_dir_all(&tasks_dir).expect("could not create tasks directory");

    let current_date = Local::now().date_naive();
    let log_file = log_file_for_date(&logs_dir, current_date);

    let event_log = if log_file.exists() {
        async_runtime::block_on(timecard::EventLog::load(log_file.into()))
            .expect("couldn't load initial time card")
    } else {
        timecard::EventLog::new(log_file.into(), current_date, timecard::State::default())
    };

    let settings =
        async_runtime::block_on(settings::Settings::load_or_new(config_file.clone().into()))
            .expect("error loading/initializing settings");

    let task_manager =
        async_runtime::block_on(tasks::TaskManager::load_or_new(tasks_dir.clone().into()))
            .expect("error loading/initializing tasks");

    let app_state = Arc::new(AppState {
        logs_dir,
        event_log: RwLock::new(event_log),
        settings: Mutex::new(settings),
        app_handle: RwLock::new(None),
        notifier: notifications::Notifier::new(),
        task_manager,
    });

    async_runtime::block_on(async {
        app_state
            .refresh_date(false, false, false)
            .await
            .expect("error refreshing event log");

        let mut event_log = app_state.event_log.write().await;
        event_log.force_active();
        event_log
            .save()
            .await
            .expect("error refreshing event log active")
    });

    let app = tauri::Builder::default()
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            clock_in,
            clock_out,
            set_tasks,
            get_current_timecard,
            get_recents,
            get_tasks,
            put_task,
            archive_task,
            make_task_recent,
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
        let result = async_runtime::block_on(async {
            let injected_idle = app_state.refresh_date(false, !idle, false).await?;

            let mut event_log = app_state.event_log.write().await;

            if idle {
                if !injected_idle {
                    event_log.add_event(timecard::Event::idle());
                    println!("idle");
                }
            } else {
                event_log.add_event(timecard::Event::active());
                println!("active");
            }

            event_log.save().await?;
            app_state.send_event_log(&event_log).await;

            Ok::<(), Box<dyn Error>>(())
        });

        if let Err(err) = result {
            println!("error handling idle update: {}", err);
        }
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
