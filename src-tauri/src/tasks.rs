use std::error::Error;

use async_std::{
    fs::File,
    io::{ReadExt, WriteExt},
    path::PathBuf,
    sync::RwLock,
};
use serde::{Deserialize, Serialize};

pub struct TaskManager {
    tasks_dir: PathBuf,

    recents: RwLock<Recents>,
    next_id: RwLock<TaskID>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Recents {
    starred: Vec<TaskID>,
    other: Vec<TaskID>,
}

impl TaskManager {
    pub async fn load_or_new(tasks_dir: PathBuf) -> Result<TaskManager, Box<dyn Error>> {
        let recents_filename = tasks_dir.join("recents.json");

        let recents: Recents = if recents_filename.exists().await {
            let mut file = File::open(&recents_filename).await?;

            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await?;

            serde_json::from_slice(&buf)?
        } else {
            Recents {
                starred: Vec::new(),
                other: Vec::new(),
            }
        };

        let next_id_filename = tasks_dir.join("next-id");

        let next_id: TaskID = if next_id_filename.exists().await {
            let mut file = File::open(&next_id_filename).await?;

            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await?;

            serde_json::from_slice(&buf)?
        } else {
            TaskID(1)
        };

        Ok(TaskManager {
            tasks_dir,
            recents: RwLock::new(recents),
            next_id: RwLock::new(next_id),
        })
    }

    async fn save_recents(&self, recents: &Recents) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_vec(recents)?;

        let mut file = File::create(self.tasks_dir.join("recents.json")).await?;
        file.write_all(&json).await?;

        Ok(())
    }

    pub async fn make_recent(&self, id: TaskID, starred: bool) -> Result<(), Box<dyn Error>> {
        let mut recents = self.recents.write().await;

        recents.starred = recents
            .starred
            .iter()
            .copied()
            .filter(|&recent_id| recent_id != id)
            .collect();
        recents.other = recents
            .other
            .iter()
            .copied()
            .filter(|&recent_id| recent_id != id)
            .collect();

        if starred {
            recents.starred.push(id);
        } else {
            recents.other.push(id);
        };

        self.save_recents(&recents).await?;

        Ok(())
    }

    pub async fn archive(&self, id: TaskID) -> Result<(), Box<dyn Error>> {
        let mut recents = self.recents.write().await;

        recents.starred = recents
            .starred
            .iter()
            .copied()
            .filter(|&recent_id| recent_id != id)
            .collect();
        recents.other = recents
            .other
            .iter()
            .copied()
            .filter(|&recent_id| recent_id != id)
            .collect();

        self.save_recents(&recents).await?;

        Ok(())
    }

    pub async fn get_recents(&self) -> Recents {
        self.recents.read().await.clone()
    }

    pub async fn next_task_id(&self) -> Result<TaskID, Box<dyn Error>> {
        let mut next_id = self.next_id.write().await;

        let id = *next_id;
        next_id.0 += 1;

        let mut next_id_file = File::create(self.tasks_dir.join("next-id")).await?;
        let json = serde_json::to_vec(&*next_id)?;

        next_id_file.write_all(&json).await?;

        Ok(id)
    }

    pub async fn save_task(&self, task: &Task) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_vec(task)?;

        let filename = self.tasks_dir.join(format!("{}.json", task.id.0));
        let mut file = File::create(&filename).await?;
        file.write_all(&json).await?;

        Ok(())
    }

    pub async fn load_task(&self, id: TaskID) -> Result<Task, Box<dyn Error>> {
        let filename = self.tasks_dir.join(format!("{}.json", id.0));
        let mut file = File::open(&filename).await?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;

        let task = serde_json::from_slice(&buf)?;
        Ok(task)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash, Default)]
pub struct TaskID(u32);

pub const TASK_ID_NONE: TaskID = TaskID(0);

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StoryType {
    #[default]
    Feature,
    Bug,
    Chore,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: TaskID,
    pub shortcut_id: Option<u32>,
    pub title: String,
    pub description: String,
    pub story_type: StoryType,
    pub starred: bool,
}
