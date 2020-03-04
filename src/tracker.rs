use crate::AddEntryMessage; // TODO move this here?
use crate::session::Player;

use std::collections::BinaryHeap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU32, Ordering as SOrdering};
use std::cmp::{Ord, Ordering};

use serde::Serialize;

static ENTRY_COUNT: AtomicU32 = AtomicU32::new(0);

pub type Tracker = RwLock<InitiativeTracker>;
pub type TrackerState = (Vec<InitiativeEntry>, usize);

// TODO this shouldn't derive Clone if possible
#[derive(Clone, Default, Debug, Serialize)]
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
		self.initiatives.clone().into_sorted_vec()
	}
	
	pub fn get_entry_by_id(&self, entry_id: u32) -> Option<&InitiativeEntry> {
		let mut element_it = self.initiatives.iter().filter(|e| e.entry_id == entry_id);
		element_it.next()
	}
	pub fn remove(&mut self, entry_id: u32) {
		// clone all elements except the one to remove into iterator
		let element_it = self.initiatives.clone().into_iter().filter(|e| e.entry_id != entry_id);
		// clear initiative list
		self.initiatives.clear();
		// add all elements left in iterator
		for e in element_it {
				self.initiatives.push(e);
		}
	}
	
	pub fn remove_all(&mut self) {
		self.initiatives.clear();
	}
	
	pub fn next(&mut self) {
		self.offset = (self.offset + 1) % self.initiatives.len();
	}
	
	pub fn get_offset(&self) -> usize {
		self.offset
	}
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InitiativeEntry {
	entry_name: String,
	entry_id: u32,
	creator_name: String,
	creator_id: u32,
	initiative: i32
}
impl InitiativeEntry {
	pub fn new(entry_data: AddEntryMessage, creator: &Player) -> InitiativeEntry {
		InitiativeEntry {
			entry_name: entry_data.entry_name,
			entry_id: ENTRY_COUNT.fetch_add(1, SOrdering::SeqCst),
			creator_name: creator.user_name.clone(),
			creator_id: creator.user_id,
			initiative: entry_data.initiative_value
		}
	}
	
	pub fn owned_by(&self, player: &Player) -> bool {
		player.user_id == self.entry_id
	}
}
impl Ord for InitiativeEntry {
	fn cmp(&self, other: &Self) -> Ordering {
		other.initiative.cmp(&self.initiative)
	}
}
impl PartialOrd for InitiativeEntry {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}