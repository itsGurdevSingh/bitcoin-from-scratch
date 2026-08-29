use std::collections::{HashMap, HashSet};
use tokio::time::Instant;

use crate::network::{InventoryVector, peer::PeerId};

pub struct InventoryManager {
    entries: HashMap<InventoryVector, InventoryEntry>,
}

pub struct InventoryEntry {
    pub state: InventoryState,
    pub announced_by: HashSet<PeerId>,
    pub requested_from: Option<PeerId>,
    pub requested_at: Option<Instant>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InventoryState {
    Unknown,
    Requested,
    Processing,
}

impl InventoryManager {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn inster_entry(
        &mut self,
        inv_vec: InventoryVector,
        inv_entry: InventoryEntry,
    ) -> Option<InventoryEntry> {
        self.entries.insert(inv_vec, inv_entry)
    }

    pub fn has_entry(&self, inv_vec: &InventoryVector) -> bool {
        self.entries.contains_key(inv_vec)
    }

    pub fn get_entry(&mut self, inv_vec: &InventoryVector) -> Option<&InventoryEntry> {
        self.entries.get(inv_vec)
    }
    pub fn remove_entry(&mut self, inv_vec: &InventoryVector) -> Option<InventoryEntry> {
        self.entries.remove(inv_vec)
    }

    pub fn mark_requested(&mut self, inv_vec: &InventoryVector, requested_form: PeerId) {
        if let Some(entry) = self.entries.get_mut(inv_vec) {
            entry.requested_from = Some(requested_form);
            entry.requested_at = Some(Instant::now());
            entry.state = InventoryState::Requested;
        };
    }
    pub fn request(&mut self, inv_vec: &InventoryVector, requested_form: PeerId) {
        let mut announced_by = HashSet::new();
        announced_by.insert(requested_form);

        self.entries.insert(
            inv_vec.clone(),
            InventoryEntry {
                state: InventoryState::Unknown,
                announced_by,
                requested_from: Some(requested_form),
                requested_at: Some(Instant::now()),
            },
        );
    }
    pub fn mark_processing(&mut self, inv_vec: &InventoryVector) {
        if let Some(entry) = self.entries.get_mut(inv_vec) {
            entry.state = InventoryState::Processing;
        };
    }

    pub fn add_announcer(&mut self, inv_vec: &InventoryVector, peer_id: PeerId) {
        if let Some(entry) = self.entries.get_mut(inv_vec) {
            entry.announced_by.insert(peer_id);
            return;
        }

        let mut announced_by = HashSet::new();
        announced_by.insert(peer_id);

        self.entries.insert(
            inv_vec.clone(),
            InventoryEntry {
                state: InventoryState::Unknown,
                announced_by,
                requested_from: None,
                requested_at: None,
            },
        );
    }
}
