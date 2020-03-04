#![feature(never_type, proc_macro_hygiene, decl_macro)]

mod tracker;
mod session;

use std::collections::HashMap;

#[macro_use] extern crate rocket;

use rocket::State;
use rocket::response::Redirect;
use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{Form, FromForm};
use rocket_contrib::{json::Json, templates::Template};

use tracker::*;
use session::*;

#[derive(FromForm)]
pub struct AddEntryMessage {
	entry_name: String,
	initiative_value: i32
}

#[derive(FromForm)]
pub struct JoinMessage {
	user_name: String
}



#[get("/join")]
pub fn render_join() -> Template {
	let ctx: HashMap<String, String> = HashMap::new();
	Template::render("join", ctx)
}
#[post("/join", data="<form_data>")]
pub fn handle_join(mut cookies: Cookies,
                   form_data: Form<JoinMessage>,
                   sessions: State<SessionState>)
-> Redirect {
	// construct Player from given data
	let player = Player::from(form_data.into_inner());
	
	// add player to session manager with given cookie 
	let cookie = generate_cookie();
	let mut sessions = sessions.write().expect("Error trying to attain lock for SessionManager");
	sessions.add_session(cookie.clone(), player);
	// add cookie
	cookies.add(Cookie::new("session", cookie));
	Redirect::to("/")
}

#[get("/masterjoin")]
pub fn render_master_join() -> Template {
	let ctx: HashMap<String, String> = HashMap::new();
	Template::render("masterjoin", ctx)
}
#[post("/masterjoin", data="<form_data>")]
pub fn handle_master_join(mut cookies: Cookies,
                   form_data: Form<JoinMessage>,
                   sessions: State<SessionState>)
-> Redirect {
	// construct Player from given data
	let player = Player::from(form_data.into_inner());
	
	// add player to session manager with given cookie and save cookie of DM-session
	let cookie = generate_cookie();
	let mut sessions = sessions.write().expect("Error trying to attain lock for SessionManager");
	sessions.add_session(cookie.clone(), player);
	sessions.set_master_cookie(cookie.clone());
	
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
pub fn handle_add(sender: Player, entry_data: Form<AddEntryMessage>, tracker: State<Tracker>) -> Redirect {
	let mut tracker = tracker.write().expect("Error trying to attain lock for TrackerState");
	let entry = InitiativeEntry::new(entry_data.into_inner(), &sender);
	tracker.add_entry(entry);
	Redirect::to("/")
}

#[post("/remove/all")]
pub fn handle_remove_all(_dm: DungeonMaster, tracker: State<Tracker>) -> Status {
	let mut tracker = tracker.write().unwrap();
	tracker.remove_all();
	Status::NoContent
}
#[post("/remove/<entry_id>")]
pub fn handle_remove_by_dm(_dm: DungeonMaster, entry_id: u32, tracker: State<Tracker>) -> Status {
	let mut tracker = tracker.write().unwrap();
	tracker.remove(entry_id);
	Status::NoContent
}
#[post("/remove/<entry_id>", rank = 2)]
pub fn handle_remove(initiator: Player, entry_id: u32, tracker: State<Tracker>) -> Status {
	let mut tracker = tracker.write().unwrap();
	match tracker.get_entry_by_id(entry_id) {
		Some(entry) if entry.owned_by(&initiator) => {
			tracker.remove(entry_id);
			Status::NoContent
		},
		Some(_) => Status::Unauthorized,
		_ => Status::NotFound
	}
}

#[post("/next")]
pub fn handle_next(_dm: DungeonMaster, tracker:State<Tracker>) -> Status {
	let mut tracker = tracker.write().unwrap();
	tracker.next();
	Status::NoContent
}

#[get("/")]
pub fn render_dm_state(_dm: DungeonMaster) -> Template {
	let ctx: HashMap<String, String> = HashMap::new();
	Template::render("status", ctx)
}
#[get("/", rank = 2)]
pub fn render_state(_player: Player) -> Template {
	let ctx: HashMap<String, String> = HashMap::new();
	Template::render("status", ctx)
}
#[get("/", rank = 3)]
pub fn redirect_join() -> Redirect {
	Redirect::to("/join")
}

#[get("/tracker")]
pub fn get_tracker(tracker: State<Tracker>) -> Json<TrackerState> {
	let tracker = tracker.read().unwrap().clone(); // TODO handle this
	Json((tracker.get_initiative_list(), tracker.get_offset()))
}

pub fn main() {
	rocket::ignite()
		.manage(Tracker::default())
		.manage(SessionState::default())
		.mount("/", routes![render_join, handle_join, render_master_join, handle_master_join,
		                    render_add, handle_add,
		                    handle_remove_by_dm, handle_remove, handle_remove_all,
		                    render_dm_state, render_state, redirect_join,
		                    get_tracker, handle_next])
		.attach(Template::fairing())
		.launch();
}
