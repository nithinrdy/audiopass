use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    runtime::{Builder, Runtime},
    sync::watch,
    time::timeout,
};
pub mod types;
use crate::backend::datastore::types::track::TracksManager;
use types::track;

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedData {
    track_list: Vec<track::Track>,
}

pub struct DatastoreManager {
    data: SavedData,
    save_sender: watch::Sender<Option<Vec<u8>>>,
    _runtime: Runtime,
}

impl DatastoreManager {
    pub fn tracks(&mut self) -> TracksManager<'_> {
        TracksManager::new(self)
    }
}

fn get_save_path() -> PathBuf {
    let config_directory = env::var_os("XDG_CONFIG_HOME")
        .filter(|env_val| !env_val.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").filter(|env_val| !env_val.is_empty()).map(|directory| PathBuf::from(directory).join(".config")))
        .unwrap_or_else(|| PathBuf::from(".config"));

    config_directory.join("audiopass").join("data.json")
}

impl DatastoreManager {
    pub fn new() -> Self {
        let data: SavedData = { std::fs::read(get_save_path()).ok().and_then(|contents| serde_json::from_slice(&contents).ok()).unwrap_or_default() };
        let runtime = Builder::new_multi_thread().worker_threads(1).enable_time().build().unwrap();

        let (save_sender, save_receiver) = watch::channel(None);
        runtime.spawn(save_worker(save_receiver, get_save_path()));

        Self { data, save_sender, _runtime: runtime }
    }

    fn debounced_save(&self) -> Result<(), String> {
        let serialized = match serde_json::to_vec_pretty(&self.data) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                return Err(format!("Failed to serialize preferences data: {error}"));
            }
        };

        match self.save_sender.send(Some(serialized)) {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to queue AudioPass preferences save: save worker stopped".to_string()),
        }
    }
}

async fn save_worker(mut receiver: watch::Receiver<Option<Vec<u8>>>, preferences_path: PathBuf) {
    while receiver.changed().await.is_ok() {
        // borrow_and_update() instead of borrow() because https://docs.rs/tokio/1.50.0/tokio/sync/watch/index.html#borrow_and_update-versus-borrow
        let Some(mut pending_snapshot) = receiver.borrow_and_update().clone() else {
            continue;
        };

        loop {
            match timeout(Duration::from_millis(500), receiver.changed()).await {
                Ok(an_update_received_within_timeout) => match an_update_received_within_timeout {
                    Ok(_an_ok_update) => {
                        if let Some(new_snapshot) = receiver.borrow_and_update().clone() {
                            pending_snapshot = new_snapshot;
                        }
                    }
                    Err(_a_recv_error_update) => {
                        // TODO: handle better
                        let _ = save_snapshot(&preferences_path, &pending_snapshot).await;
                        return;
                    }
                },
                Err(_no_update_received_within_timeout) => {
                    // TODO: handle better
                    let _ = save_snapshot(&preferences_path, &pending_snapshot).await;
                    break;
                }
            }
        }
    }
}

async fn save_snapshot(preferences_path: &Path, snapshot: &[u8]) -> Result<(), String> {
    println!("saving");
    match {
        if let Some(parent_dir) = preferences_path.parent() {
            let _ = tokio::fs::create_dir_all(parent_dir).await;
        }
        tokio::fs::write(preferences_path, snapshot)
    }
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to save AudioPass preferences: {e}")),
    }
}
