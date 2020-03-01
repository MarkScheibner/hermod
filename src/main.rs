#![feature(proc_macro_hygiene, decl_macro)]

use std::sync::RwLock;
use std::collections::HashMap;

#[macro_use] extern crate rocket;
use serde::{Serialize, Deserialize};
use rocket::{State, response::status};
use rocket_contrib::json::Json;
use rocket_contrib::templates::Template;

type TrackerState = RwLock<InitiativeTracker>;
type DungeonMaster = Option<(String, String)>;
type SessionManager = RwLock<HashMap<String, Player>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct InitiativeTracker {
	initiative_entries: Vec<InitiativeEntry>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitiativeEntry {
	entry_name: String,
	initiative: u32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
	user_name: String,
	user_id: u32
}



#[get("/status")]
pub fn get_status(tracker: State<TrackerState>) -> String {
	let tracker = tracker.read().expect("Error trying to attain read for TrackerState");
	format!("{:?}", tracker.initiative_entries)
}

#[get("/join")]
pub fn render_join() -> Template {
	let ctx: HashMap<String, String> = HashMap::new();
	Template::render("join", ctx)
}

#[get("/add")]
pub fn render_add() -> Template {
	let ctx: HashMap<String, String> = HashMap::new();
	Template::render("add", ctx)
}
#[post("/add", data="<entry_data>")]
pub fn handle_add(entry_data: Json<InitiativeEntry>, tracker: State<TrackerState>) -> status::Accepted<()>{
	let mut tracker = tracker.write().expect("Error trying to attain lock for TrackerState");
	tracker.initiative_entries.push(entry_data.into_inner());
	status::Accepted(Some(()))
}

pub fn main() {
	rocket::ignite()
		.manage(RwLock::from(InitiativeTracker { initiative_entries: Vec::new() }))
		.manage(RwLock::from(HashMap::<String, Player>::new()))
		.manage(None::<DungeonMaster>)
		.mount("/", routes![get_status, render_join, render_add, handle_add])
		.attach(Template::fairing())
		.launch();
}

