use std::error::Error;

use async_std::{
    fs::File,
    io::{ReadExt, WriteExt},
    path::PathBuf,
};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(skip)]
    filename: PathBuf,

    pub current_date: chrono::NaiveDate,
    pub work_target: std::time::Duration,
    pub lunch_target: std::time::Duration,
    pub break_target: std::time::Duration,
}

impl Settings {
    pub async fn load_or_new(filename: PathBuf) -> Result<Settings, Box<dyn Error>> {
        if filename.exists().await {
            let mut file = File::open(&filename).await?;

            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await?;

            let mut settings: Settings = serde_json::from_slice(&buf)?;
            settings.filename = filename;
            Ok(settings)
        } else {
            let settings = Settings {
                filename,

                current_date: Local::now().date_naive(),
                work_target: std::time::Duration::from_secs(8 * 60 * 60),
                lunch_target: std::time::Duration::from_secs(60 * 60),
                break_target: std::time::Duration::from_secs(30 * 60),
            };

            settings.save().await?;

            Ok(settings)
        }
    }

    pub async fn save(&self) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_vec(self)?;

        let mut file = File::create(&self.filename).await?;
        file.write_all(&json).await?;

        Ok(())
    }
}
