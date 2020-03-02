#![feature(never_type, proc_macro_hygiene, decl_macro)]

mod tracker;

use std::sync::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

extern crate base64;
extern crate rand;
#[macro_use] extern crate rocket;

use rand::RngCore;
use rocket::State;
use rocket::response::Redirect;
use rocket::http::{Cookie, Cookies};
use rocket::request::{Form, FromForm, Request, FromRequest, Outcome};
use rocket_contrib::templates::Template;
use serde::{Serialize, Deserialize};

use tracker::*;

type DungeonMaster = Option<(String, String)>;
type SessionManager = RwLock<HashMap<String, Player>>;

static USER_COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
	user_name: String,
	user_id: u32
}
impl<'a, 'r> FromRequest<'a, 'r> for Player {
	type Error = ();
	
	fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
		let session_manager = req.guard::<State<SessionManager>>()?;
		let session_manager = session_manager.read().unwrap(); // TODO handle this?
		let p = req.cookies().get("session").map(|c| c.value()).and_then(|c| session_manager.get(c));
		match p {
			Some(player) => Outcome::Success(player.clone()),
			None => Outcome::Forward(())
		}
	}
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


#[get("/", rank = 2)]
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
pub fn handle_add(sender: Player, entry_data: Form<AddEntryMessage>, tracker: State<Tracker>) -> Redirect{
	let mut tracker = tracker.write().expect("Error trying to attain lock for TrackerState");
	let entry = InitiativeEntry::new(entry_data.into_inner(), &sender);
	tracker.add_entry(entry);
	Redirect::to("/")
}

#[get("/")]
pub fn show_state(player: Player, tracker: State<Tracker>) -> String {
	let user = player.user_name;
	let tracker = tracker.read().unwrap();
	format!("hello {}!\ninitiatives: {:?}", user, tracker.get_initiative_list())
}

pub fn main() {
	rocket::ignite()
		.manage(RwLock::from(InitiativeTracker::new()))
		.manage(RwLock::from(HashMap::<String, Player>::new()))
		.manage(None::<DungeonMaster>)
		.manage(AtomicU32::from(0))
		.mount("/", routes![render_join, handle_join, render_add, handle_add, show_state])
		.attach(Template::fairing())
		.launch();
}

pub fn generate_cookie() -> String {
	let mut cookie_bytes = [0; 16];
	rand::thread_rng().fill_bytes(&mut cookie_bytes);
	base64::encode(&cookie_bytes)
}

