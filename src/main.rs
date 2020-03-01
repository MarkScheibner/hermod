#![feature(proc_macro_hygiene, decl_macro)]

mod tracker;

use std::sync::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

extern crate base64;
extern crate rand;
#[macro_use] extern crate rocket;

use rand::RngCore;
use rocket::State;
use rocket::response::{status, Redirect};
use rocket::http::{Cookie, Cookies};
use rocket::request::{Form, FromForm};
use rocket_contrib::templates::Template;
use serde::{Serialize, Deserialize};

use tracker::*;

type DungeonMaster = Option<(String, String)>;
type SessionManager = RwLock<HashMap<String, Player>>;

static USER_COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
	user_name: String,
	user_id: u32
}

#[derive(FromForm)]
pub struct AddEntryMessage {
	entry_name: String,
	initiative_value: u32
}


#[derive(FromForm)]
pub struct JoinMessage {
	user_name: String
}
impl From<JoinMessage> for Player {
	fn from(msg: JoinMessage) -> Player {
		Player {
			user_name: msg.user_name,
			user_id: crate::USER_COUNT.fetch_add(1, Ordering::SeqCst)
		}
	}
}


#[get("/status")]
pub fn get_status(tracker: State<Tracker>) -> String {
	let tracker = tracker.read().expect("Error trying to attain read for TrackerState");
	format!("{:?}", tracker.get_initiative_list())
}

#[get("/join")]
pub fn render_join() -> Template {
	let ctx: HashMap<String, String> = HashMap::new();
	Template::render("join", ctx)
}
#[post("/join", data="<form_data>")]
pub fn handle_join(mut cookies: Cookies,
                   form_data: Form<JoinMessage>,
                   session_manager: State<SessionManager>)
-> Redirect {
	// construct Player from given data
	let player = Player::from(form_data.into_inner());
	let cookie = generate_cookie();
	let mut sm = session_manager.write().expect("Error trying to attain lock for SessionManager");
	sm.insert(cookie.clone(), player);
	// add cookie
	cookies.add(Cookie::new("session", cookie));
	Redirect::to("/")
}

#[get("/add")]
pub fn render_add() -> Template {
	let ctx: HashMap<String, String> = HashMap::new();
	Template::render("add", ctx)
}
#[post("/add", data="<entry_data>")]
pub fn handle_add(entry_data: Form<AddEntryMessage>, tracker: State<Tracker>) -> status::Accepted<()>{
	let mut tracker = tracker.write().expect("Error trying to attain lock for TrackerState");
	let entry = InitiativeEntry::new(entry_data.into_inner());
	tracker.add_entry(entry);
	status::Accepted(Some(()))
}

#[get("/")]
pub fn show_state(session_manager: State<SessionManager>, tracker: State<Tracker>, cookies: Cookies) -> String {
	let session_manager = session_manager.read().unwrap();
	let tracker = tracker.read().unwrap();
	let user: String = session_manager.get(cookies.get("session").map(|c| c.value()).unwrap_or(""))
	                          .map(|p| p.user_name.clone())
	                          .unwrap_or_else(|| "unnamed user".into());
	format!("hello {}!\nsessions: {:?}\n\n---------\n\ninitiatives: {:?}", user, *session_manager, *tracker)
}

pub fn main() {
	rocket::ignite()
		.manage(RwLock::from(InitiativeTracker::new()))
		.manage(RwLock::from(HashMap::<String, Player>::new()))
		.manage(None::<DungeonMaster>)
		.manage(AtomicU32::from(0))
		.mount("/", routes![show_state, get_status, render_join, handle_join, render_add, handle_add])
		.attach(Template::fairing())
		.launch();
}

pub fn generate_cookie() -> String {
	let mut cookie_bytes = [0; 16];
	rand::thread_rng().fill_bytes(&mut cookie_bytes);
	base64::encode(&cookie_bytes)
}

