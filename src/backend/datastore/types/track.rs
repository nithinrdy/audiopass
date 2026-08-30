use serde::{Deserialize, Serialize};
use std::path;
use uuid::Uuid;

use crate::backend::datastore::DatastoreManager;

#[derive(Serialize, Deserialize, Clone)]
pub struct Track {
    pub id: Uuid,
    pub path: path::PathBuf,
}

pub struct TracksManager<'a> {
    datastore: &'a mut DatastoreManager,
}

impl TracksManager<'_> {
    pub fn new(d: &mut DatastoreManager) -> TracksManager<'_> {
        TracksManager { datastore: d }
    }

    pub fn get(&self) -> &[Track] {
        &self.datastore.data.track_list
    }

    pub fn remove(&mut self, uuid: Uuid) -> Result<(), String> {
        self.datastore.data.track_list.retain(|t| t.id != uuid);
        self.datastore.debounced_save()
    }

    pub fn add(&mut self, tracks_to_add: Vec<path::PathBuf>) -> Result<(), String> {
        self.datastore.data.track_list.extend(tracks_to_add.into_iter().map(|p| Track { id: Uuid::new_v4(), path: p }));
        self.datastore.debounced_save()
    }
}
