use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::JoinMessage;

use rocket::State;
use rocket::request::{Request, FromRequest, Outcome};
use serde::{Serialize, Deserialize};
use rand::RngCore;

pub type SessionState = RwLock<SessionManager>;

static USER_COUNT: AtomicU32 = AtomicU32::new(1);

#[derive(Default)]
pub struct SessionManager {
	sessions: HashMap<String, Player>,
	master_cookie: Option<String>
}
impl SessionManager {
	pub fn get_session(&self, cookie: &str) -> Option<&Player> {
		self.sessions.get(cookie)
	}
	pub fn add_session(&mut self, cookie: String, session: Player) {
		self.sessions.insert(cookie, session);
	}
	
	pub fn is_master_session(&self, cookie: &str) -> bool {
		self.master_cookie.is_some() && self.master_cookie.as_ref().unwrap().eq(cookie)
	}
	pub fn set_master_cookie(&mut self, cookie: String) {
		self.master_cookie = Some(cookie);
	}
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Player {
	pub user_name: String,
	pub user_id: u32
}
impl<'a, 'r> FromRequest<'a, 'r> for Player {
	type Error = ();
	
	fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
		let sessions = req.guard::<State<SessionState>>()?;
		let sessions = sessions.read().unwrap(); // TODO handle this?
		let user = req.cookies().get("session").map(|c| c.value()).and_then(|c| sessions.get_session(c));
		match user {
			Some(player) => Outcome::Success(player.clone()),
			None => Outcome::Forward(())
		}
	}
}
impl From<JoinMessage> for Player {
	fn from(msg: JoinMessage) -> Player {
		Player {
			user_name: msg.user_name,
			user_id: USER_COUNT.fetch_add(1, Ordering::SeqCst)
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DungeonMaster();
impl<'a, 'r> FromRequest<'a, 'r> for DungeonMaster {
	type Error = ();
	
	fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
		let sessions = req.guard::<State<SessionState>>()?;
		let sessions = sessions.read().unwrap(); // TODO handle this?
		
		let user_cookie = req.cookies().get("session").map(|c| c.value().to_string());
		match user_cookie.map(|c| sessions.is_master_session(&c)) {
			None | Some(false) => Outcome::Forward(()),
			Some(true) => Outcome::Success(DungeonMaster())
		}
	}
}

pub fn generate_cookie() -> String {
	let mut cookie_bytes = [0; 16];
	rand::thread_rng().fill_bytes(&mut cookie_bytes);
	base64::encode(&cookie_bytes)
}