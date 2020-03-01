use crate::AddEntryMessage; // TODO move this here?

use std::collections::BinaryHeap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU32, Ordering as SOrdering};
use std::cmp::{Ord, Ordering};

static ENTRY_COUNT: AtomicU32 = AtomicU32::new(0);

pub type Tracker = RwLock<InitiativeTracker>;

#[derive(Default, Debug)]
pub struct InitiativeTracker {
	initiatives: BinaryHeap<InitiativeEntry>,
	offset: usize
}
impl InitiativeTracker {
	pub fn new() -> InitiativeTracker {
		InitiativeTracker {
			initiatives: BinaryHeap::new(),
			offset: 0
		}
	}
	
	pub fn add_entry(&mut self, entry: InitiativeEntry) {
		self.initiatives.push(entry)
	}
	
	pub fn get_initiative_list(&self) -> Vec<InitiativeEntry> {
		let mut initative_list = self.initiatives.clone().into_sorted_vec();
		initative_list.rotate_left(self.offset);
		initative_list
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitiativeEntry {
	entry_name: String,
	entry_id: u32,
	player_id: u32,
	initiative: u32
}
impl InitiativeEntry {
	pub fn new(entry_data: AddEntryMessage) -> InitiativeEntry {
		InitiativeEntry {
			entry_name: entry_data.entry_name,
			entry_id: ENTRY_COUNT.fetch_add(1, SOrdering::SeqCst),
			player_id: 0,
			initiative: entry_data.initiative_value
		}
	}
}
impl Ord for InitiativeEntry {
	fn cmp(&self, other: &Self) -> Ordering {
		self.initiative.cmp(&other.initiative)
	}
}
impl PartialOrd for InitiativeEntry {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}