use std::error::Error;

use async_std::{
    fs::File,
    io::{ReadExt, WriteExt},
    path::{Path, PathBuf},
    sync::RwLock,
};
use serde::{Deserialize, Serialize};

pub struct TaskManager {
    tasks_dir: PathBuf,

    recents: RwLock<Recents>,
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

        Ok(TaskManager {
            tasks_dir,
            recents: RwLock::new(recents),
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

        let recents_vec = if starred {
            &mut recents.starred
        } else {
            &mut recents.other
        };
        let max_len = if starred { usize::MAX } else { 20 };

        let mut index = None;

        for (i, recent_id) in recents_vec.iter().enumerate() {
            if *recent_id == id {
                index = Some(i);
                break;
            }
        }

        if let Some(index) = index {
            recents_vec.remove(index);
        }

        if recents_vec.len() >= max_len {
            recents_vec.drain(0..(recents_vec.len() - max_len + 1));
        }

        recents_vec.push(id);

        self.save_recents(&recents).await?;

        Ok(())
    }

    pub async fn get_recents(&self) -> Recents {
        self.recents.read().await.clone()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct TaskID(u32);

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoryType {
    Feature,
    Bug,
    Chore,
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    id: TaskID,
    pub shortcut_id: Option<u32>,
    pub title: String,
    pub description: String,
    pub story_type: StoryType,
    pub starred: bool,
}

impl Task {
    pub fn id(&self) -> TaskID {
        self.id
    }

    pub async fn save(&self, tasks_dir: &Path) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_vec(self)?;

        let filename = tasks_dir.join(format!("{}.json", self.id.0));
        let mut file = File::create(&filename).await?;
        file.write_all(&json).await?;

        Ok(())
    }
}
