use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
};

use crate::tasks::TaskID;
use async_std::prelude::*;
use async_std::{fs::File, path::PathBuf};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct TrackedTime {
    since: Option<chrono::DateTime<FixedOffset>>,
    accumulated: std::time::Duration,
}

impl TrackedTime {
    pub fn start_at(&mut self, time: chrono::DateTime<FixedOffset>) {
        if self.since.is_none() {
            self.since = Some(time)
        }
    }

    pub fn end_at(&mut self, time: chrono::DateTime<FixedOffset>) {
        if let Some(start) = self.since {
            self.accumulated += (time - start).to_std().unwrap_or(std::time::Duration::ZERO);
            self.since = None;
        }
    }

    pub fn active(&self) -> bool {
        self.since.is_some()
    }

    pub fn elapsed_for_date(&self, date: NaiveDate) -> std::time::Duration {
        let mut elapsed = self.accumulated;

        if let Some(mut since) = self.since {
            let now = Local::now();

            let day_start = NaiveDateTime::new(date, NaiveTime::MIN)
                .and_local_timezone(now.timezone())
                .unwrap();
            let day_end = NaiveDateTime::new(date + chrono::Days::new(1), NaiveTime::MIN)
                .and_local_timezone(now.timezone())
                .unwrap();

            let end = std::cmp::min(now, day_end);

            if since < end {
                if since < day_start {
                    since = day_start.fixed_offset();
                }

                elapsed += (end.fixed_offset() - since).to_std().unwrap();
            }
        }

        elapsed
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TrackedMultiTime<T: Ord> {
    since: Option<chrono::DateTime<FixedOffset>>,
    ids: BTreeSet<T>,
    accumulated: BTreeMap<T, std::time::Duration>,
}

impl<T: Ord> Default for TrackedMultiTime<T> {
    fn default() -> Self {
        TrackedMultiTime {
            since: None,
            ids: BTreeSet::new(),
            accumulated: BTreeMap::new(),
        }
    }
}

impl<T: Ord + Copy> TrackedMultiTime<T> {
    pub fn set_tracked(&mut self, time: chrono::DateTime<FixedOffset>, ids: BTreeSet<T>) {
        let paused = self.pause(time);

        self.ids = ids;

        if paused {
            self.resume(time);
        }
    }

    pub fn pause(&mut self, time: chrono::DateTime<FixedOffset>) -> bool {
        if let Some(start) = self.since.take() {
            // Accumulate time
            if !self.ids.is_empty() {
                let per_id_time = (time - start).to_std().unwrap_or(std::time::Duration::ZERO)
                    / (self.ids.len() as u32);

                for &past_id in &self.ids {
                    self.accumulated
                        .entry(past_id)
                        .and_modify(|d| *d += per_id_time)
                        .or_insert(per_id_time);
                }
            }

            true
        } else {
            false
        }
    }

    pub fn resume(&mut self, time: chrono::DateTime<FixedOffset>) -> bool {
        if self.since.is_none() {
            self.since = Some(time);

            true
        } else {
            false
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct State {
    working: TrackedTime,
    on_break: TrackedTime,
    on_lunch: TrackedTime,

    idle_work: TrackedTime,
    active_until: Option<chrono::DateTime<FixedOffset>>,

    tasks: TrackedMultiTime<TaskID>,
}

impl State {
    pub fn reset_accumulations(&mut self) {
        self.working.accumulated = std::time::Duration::ZERO;
        self.on_break.accumulated = std::time::Duration::ZERO;
        self.on_lunch.accumulated = std::time::Duration::ZERO;
        self.idle_work.accumulated = std::time::Duration::ZERO;
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventLog {
    initial_state: State,
    current_state: State,
    events: BTreeSet<Event>,

    date: NaiveDate,

    #[serde(skip)]
    filename: PathBuf,
}

impl EventLog {
    pub fn new(filename: PathBuf, date: NaiveDate, initial_state: State) -> EventLog {
        EventLog {
            current_state: initial_state.clone(),
            initial_state,
            events: BTreeSet::new(),
            date,
            filename,
        }
    }

    pub fn add_event(&mut self, event: Event) {
        match &event {
            Event::ClockIn { clock, time } => match clock {
                ClockType::Day => {
                    self.current_state.working.start_at(*time);
                    self.current_state.tasks.resume(*time);
                }
                ClockType::Break => {
                    self.current_state.on_break.start_at(*time);
                    self.current_state.working.start_at(*time);
                    self.current_state.on_lunch.end_at(*time);
                    self.current_state.idle_work.end_at(*time);
                    self.current_state.tasks.pause(*time);
                }
                ClockType::Lunch => {
                    self.current_state.on_lunch.start_at(*time);
                    self.current_state.working.start_at(*time);
                    self.current_state.on_break.end_at(*time);
                    self.current_state.idle_work.end_at(*time);
                    self.current_state.tasks.pause(*time);
                }
            },
            Event::ClockOut { clock, time } => match clock {
                ClockType::Day => {
                    self.current_state.working.end_at(*time);
                    self.current_state.on_break.end_at(*time);
                    self.current_state.on_lunch.end_at(*time);
                    self.current_state.idle_work.end_at(*time);
                    self.current_state.tasks.pause(*time);
                }
                ClockType::Break => {
                    self.current_state.on_break.end_at(*time);
                    self.current_state.tasks.resume(*time);
                }
                ClockType::Lunch => {
                    self.current_state.on_lunch.end_at(*time);
                    self.current_state.tasks.resume(*time);
                }
            },
            Event::Active { time } => {
                self.current_state.active_until = Some(*time);
                self.current_state.idle_work.end_at(*time);
            }
            Event::Idle { time } => {
                self.current_state.active_until = None;

                if self.current_state.working.active()
                    && !self.current_state.on_break.active()
                    && !self.current_state.on_lunch.active()
                {
                    self.current_state.idle_work.start_at(*time);
                }
            }
            Event::Tasks { tasks, time } => {
                self.current_state.tasks.set_tracked(*time, tasks.clone())
            }
        }

        self.events.insert(event);
    }

    pub async fn save(&self) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_vec(self)?;

        let mut file = File::create(&self.filename).await?;
        file.write_all(&json).await?;

        Ok(())
    }

    pub async fn load(filename: PathBuf) -> Result<EventLog, Box<dyn Error>> {
        let mut file = File::open(&filename).await?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;

        let mut event_log: EventLog = serde_json::from_slice(&buf)?;
        event_log.filename = filename;
        Ok(event_log)
    }

    pub fn get_state(&self) -> State {
        self.current_state.clone()
    }

    pub fn infer_idle(&mut self) -> bool {
        let now = Local::now().fixed_offset();

        if let Some(active_until) = &self.current_state.active_until {
            // App was closed or something, add an idle event from last known active time
            if now - active_until > chrono::Duration::minutes(5) {
                println!("was last active >5m ago, injecting idle event");
                self.add_event(Event::Idle {
                    time: *active_until,
                });
                self.current_state.active_until = None;

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn force_active(&mut self) {
        let now = Local::now().fixed_offset();

        if self.current_state.active_until.is_none() {
            self.add_event(Event::Active { time: now });
        }

        self.current_state.active_until = Some(now);
    }

    pub async fn refresh_active(&mut self) -> Result<(), Box<dyn Error>> {
        if self.current_state.active_until.is_some() {
            self.current_state.active_until = Some(Local::now().fixed_offset());
            self.save().await?;
        }

        Ok(())
    }

    pub fn elapsed(&self) -> ElapsedSummary {
        ElapsedSummary {
            work_time: self.current_state.working.elapsed_for_date(self.date),
            break_time: self.current_state.on_break.elapsed_for_date(self.date),
            lunch_time: self.current_state.on_lunch.elapsed_for_date(self.date),
            idle_work_time: self.current_state.idle_work.elapsed_for_date(self.date),

            working: self.current_state.working.active(),
            on_break: self.current_state.on_break.active(),
            on_lunch: self.current_state.on_lunch.active(),
            idle_work: self.current_state.idle_work.active(),
        }
    }
}

pub struct ElapsedSummary {
    pub work_time: std::time::Duration,
    pub break_time: std::time::Duration,
    pub lunch_time: std::time::Duration,
    pub idle_work_time: std::time::Duration,

    pub working: bool,
    pub on_break: bool,
    pub on_lunch: bool,
    pub idle_work: bool,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    ClockIn {
        time: DateTime<FixedOffset>,
        clock: ClockType,
    },
    ClockOut {
        time: DateTime<FixedOffset>,
        clock: ClockType,
    },
    Active {
        time: DateTime<FixedOffset>,
    },
    Idle {
        time: DateTime<FixedOffset>,
    },
    Tasks {
        time: DateTime<FixedOffset>,
        tasks: BTreeSet<TaskID>,
    },
}

impl std::cmp::Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_time = self.time();
        let other_time = other.time();

        let time_cmp = self_time.cmp(&other_time);
        if time_cmp != std::cmp::Ordering::Equal {
            return time_cmp;
        }

        let self_variant = self.variant_id();
        let other_variant = other.variant_id();

        let variant_cmp = self_variant.cmp(&other_variant);
        if variant_cmp != std::cmp::Ordering::Equal {
            return variant_cmp;
        }

        match (self, other) {
            (
                Event::ClockIn {
                    clock: self_clock, ..
                },
                Event::ClockIn {
                    clock: other_clock, ..
                },
            ) => self_clock.cmp(other_clock),
            (
                Event::ClockOut {
                    clock: self_clock, ..
                },
                Event::ClockOut {
                    clock: other_clock, ..
                },
            ) => self_clock.cmp(other_clock),
            (Event::Idle { .. }, Event::Idle { .. }) => std::cmp::Ordering::Equal,
            (Event::Active { .. }, Event::Active { .. }) => std::cmp::Ordering::Equal,
            (
                Event::Tasks {
                    tasks: self_tasks, ..
                },
                Event::Tasks {
                    tasks: other_tasks, ..
                },
            ) => self_tasks.cmp(other_tasks),
            _ => unreachable!(),
        }
    }
}

impl std::cmp::PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Event {
    pub fn clock_in(clock: ClockType) -> Event {
        Event::ClockIn {
            time: Local::now().fixed_offset(),
            clock,
        }
    }

    pub fn clock_out(clock: ClockType) -> Event {
        Event::ClockOut {
            time: Local::now().fixed_offset(),
            clock,
        }
    }

    pub fn active() -> Event {
        Event::Active {
            time: Local::now().fixed_offset(),
        }
    }

    pub fn idle() -> Event {
        Event::Idle {
            time: Local::now().fixed_offset(),
        }
    }

    pub fn tasks(tasks: BTreeSet<TaskID>) -> Event {
        Event::Tasks {
            time: Local::now().fixed_offset(),
            tasks,
        }
    }

    pub fn time(&self) -> DateTime<FixedOffset> {
        match *self {
            Event::ClockIn { time, .. } => time,
            Event::ClockOut { time, .. } => time,
            Event::Active { time } => time,
            Event::Idle { time } => time,
            Event::Tasks { time, .. } => time,
        }
    }

    fn variant_id(&self) -> u32 {
        match &self {
            Event::ClockIn { .. } => 0,
            Event::ClockOut { .. } => 1,
            Event::Active { .. } => 2,
            Event::Idle { .. } => 3,
            Event::Tasks { .. } => 4,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ClockType {
    Day,
    Break,
    Lunch,
}
