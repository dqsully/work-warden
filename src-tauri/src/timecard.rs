use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    ffi::OsString,
};

use async_std::fs::File;
use async_std::prelude::*;
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
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TrackedMultiTime<T: Ord> {
    since: Option<(chrono::DateTime<FixedOffset>, BTreeSet<T>)>,
    accumulated: BTreeMap<T, std::time::Duration>,
}

impl<T: Ord> Default for TrackedMultiTime<T> {
    fn default() -> Self {
        TrackedMultiTime {
            since: None,
            accumulated: BTreeMap::new(),
        }
    }
}

impl<T: Ord> TrackedMultiTime<T> {
    pub fn set_tracked(&mut self, time: chrono::DateTime<FixedOffset>, ids: BTreeSet<T>)
    {
        if let Some((start, past_ids)) = self.since.take() {
            let per_id_time = (time - start).to_std().unwrap_or(std::time::Duration::ZERO) / (past_ids.len() as u32);

            for past_id in past_ids {
                self.accumulated.entry(past_id)
                    .and_modify(|d| *d += per_id_time)
                    .or_insert(per_id_time);
            }
        }

        if !ids.is_empty() {
            self.since = Some((time, ids));
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct State {
    working: TrackedTime,
    #[serde(rename = "onBreak")]
    on_break: TrackedTime,
    #[serde(rename = "onLunch")]
    on_lunch: TrackedTime,

    #[serde(rename = "idleWork")]
    idle_work: TrackedTime,
    #[serde(rename = "isIdle")]
    is_idle: bool,

    tasks: TrackedMultiTime<TaskID>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EventLog {
    #[serde(rename = "initialState")]
    initial_state: State,
    #[serde(rename = "currentState")]
    current_state: State,
    events: BTreeSet<Event>,

    #[serde(skip)]
    filename: OsString,
}

impl EventLog {
    pub fn new(filename: OsString, initial_state: State) -> EventLog {
        EventLog {
            current_state: initial_state.clone(),
            initial_state,
            events: BTreeSet::new(),
            filename,
        }
    }

    pub fn add_event(&mut self, event: Event) {
        match &event {
            Event::ClockIn { clock, time } => match clock {
                ClockType::Day => {
                    self.current_state.working.start_at(*time);

                    if self.current_state.is_idle {
                        self.current_state.idle_work.start_at(*time);
                    }
                },
                ClockType::Break => {
                    self.current_state.on_break.start_at(*time);
                    self.current_state.on_lunch.end_at(*time);
                },
                ClockType::Lunch => {
                    self.current_state.on_lunch.start_at(*time);
                    self.current_state.on_break.end_at(*time);
                },
            },
            Event::ClockOut { clock, time } => match clock {
                ClockType::Day => {
                    self.current_state.working.end_at(*time);
                    self.current_state.on_break.end_at(*time);
                    self.current_state.on_lunch.end_at(*time);
                    self.current_state.idle_work.end_at(*time);
                },
                ClockType::Break => self.current_state.on_break.end_at(*time),
                ClockType::Lunch => self.current_state.on_lunch.end_at(*time),
            },
            Event::Active { time } => {
                self.current_state.is_idle = false;
                self.current_state.idle_work.end_at(*time);
            },
            Event::Idle { time } => {
                self.current_state.is_idle = true;

                if self.current_state.working.active() {
                    self.current_state.idle_work.start_at(*time);
                }
            },
            Event::Tasks { tasks, time } => self.current_state.tasks.set_tracked(*time, tasks.clone()),
        }

        self.events.insert(event);
    }

    pub async fn save(&self) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string(self)?;

        let mut file = File::create(&self.filename).await?;
        file.write_all(json.as_bytes()).await?;

        Ok(())
    }

    pub async fn load(filename: OsString) -> Result<EventLog, Box<dyn Error>> {
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TaskID(u64);

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    id: TaskID,
    #[serde(rename = "shortcutID")]
    shortcut_id: Option<u64>,
    title: String,
    description: String,
}
