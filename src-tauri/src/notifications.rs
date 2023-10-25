use std::error::Error;

use async_std::sync::Mutex;
use chrono::prelude::*;
use notify_rust::{Hint, Notification, NotificationHandle, Timeout};

struct OverNotification {
    handle: NotificationHandle,
    render: Box<dyn Fn(std::time::Duration) -> (String, String) + Send + Sync>,
    since: chrono::DateTime<Local>,
    accumulated: std::time::Duration,
}

impl OverNotification {
    async fn new<F>(
        accumulated: std::time::Duration,
        render_func: F,
    ) -> Result<OverNotification, Box<dyn Error>>
    where
        F: Fn(std::time::Duration) -> (String, String) + Send + Sync + 'static,
    {
        let (summary, body) = render_func(accumulated);

        let handle = Notification::new()
            .summary(&summary)
            .body(&body)
            .hint(Hint::Resident(true))
            .timeout(Timeout::Never)
            .show_async()
            .await?;

        Ok(OverNotification {
            handle,
            render: Box::new(render_func),
            since: Local::now(),
            accumulated,
        })
    }

    async fn refresh(&self) -> Result<(), Box<dyn Error>> {
        let total = self.accumulated + (Local::now() - self.since).to_std().unwrap();
        let (summary, body) = (self.render)(total);

        Notification::new()
            .id(self.handle.id())
            .summary(&summary)
            .body(&body)
            .hint(Hint::Resident(true))
            .timeout(Timeout::Never)
            .show_async()
            .await?;

        Ok(())
    }
}

pub struct Notifier {
    overtime: Mutex<Option<OverNotification>>,
    long_lunch: Mutex<Option<OverNotification>>,
    long_break: Mutex<Option<OverNotification>>,
}

impl Notifier {
    pub fn new() -> Notifier {
        Notifier {
            overtime: Mutex::new(None),
            long_lunch: Mutex::new(None),
            long_break: Mutex::new(None),
        }
    }

    pub async fn show_overtime(&self, over: std::time::Duration) -> Result<(), Box<dyn Error>> {
        let mut overtime = self.overtime.lock().await;

        if let Some(overtime) = &mut *overtime {
            overtime.accumulated = over;
            overtime.since = Local::now();
            overtime.refresh().await?;
        } else {
            *overtime = Some(
                OverNotification::new(over, |dur| {
                    (
                        "Working overtime".to_owned(),
                        format!("{} overtime worked today", format_duration_minutes(dur)),
                    )
                })
                .await?,
            )
        }

        Ok(())
    }

    pub async fn clear_overtime(&self) {
        let notification = self.overtime.lock().await.take();

        if let Some(notification) = notification {
            notification.handle.close();
        }
    }

    pub async fn show_long_lunch(&self, over: std::time::Duration) -> Result<(), Box<dyn Error>> {
        let mut long_lunch = self.long_lunch.lock().await;

        if let Some(long_lunch) = &mut *long_lunch {
            long_lunch.accumulated = over;
            long_lunch.since = Local::now();
            long_lunch.refresh().await?;
        } else {
            *long_lunch = Some(
                OverNotification::new(over, |dur| {
                    (
                        "Long lunch".to_owned(),
                        format!("Over by {} for lunch today", format_duration_minutes(dur)),
                    )
                })
                .await?,
            )
        }

        Ok(())
    }

    pub async fn clear_long_lunch(&self) {
        let notification = self.long_lunch.lock().await.take();

        if let Some(notification) = notification {
            notification.handle.close();
        }
    }

    pub async fn show_long_break(&self, over: std::time::Duration) -> Result<(), Box<dyn Error>> {
        let mut long_break = self.long_break.lock().await;

        if let Some(long_break) = &mut *long_break {
            long_break.accumulated = over;
            long_break.since = Local::now();
            long_break.refresh().await?;
        } else {
            *long_break = Some(
                OverNotification::new(over, |dur| {
                    (
                        "Over break time".to_owned(),
                        format!(
                            "Over by {} for break time today",
                            format_duration_minutes(dur)
                        ),
                    )
                })
                .await?,
            )
        }

        Ok(())
    }

    pub async fn clear_long_break(&self) {
        let notification = self.long_break.lock().await.take();

        if let Some(notification) = notification {
            notification.handle.close();
        }
    }
}

fn format_duration_minutes(dur: std::time::Duration) -> String {
    let mut minutes = dur.as_secs() / 60;

    if minutes == 0 {
        return "0m".to_owned();
    }

    let mut out = String::new();

    if minutes > 60 * 24 * 7 {
        out += &format!("{}w", minutes / (60 * 24 * 7));
        minutes %= 60 * 24 * 7;
    }

    if minutes > 60 * 24 {
        out += &format!("{}d", minutes / (60 * 24));
        minutes %= 60 * 24;
    }

    if minutes > 60 {
        out += &format!("{}h", minutes / 60);
        minutes %= 60;
    }

    if minutes > 0 {
        out += &format!("{}m", minutes);
    }

    out
}
