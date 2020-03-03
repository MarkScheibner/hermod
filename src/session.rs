use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::JoinMessage;

use rocket::State;
use rocket::request::{Request, FromRequest, Outcome};
use serde::{Serialize, Deserialize};

pub type MasterCookie = RwLock<Option<String>>;
pub type SessionManager = RwLock<HashMap<String, Player>>;


static USER_COUNT: AtomicU32 = AtomicU32::new(1);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
	pub user_name: String,
	pub user_id: u32
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
impl From<JoinMessage> for Player {
	fn from(msg: JoinMessage) -> Player {
		Player {
			user_name: msg.user_name,
			user_id: USER_COUNT.fetch_add(1, Ordering::SeqCst)
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DungeonMaster(Player);
impl<'a, 'r> FromRequest<'a, 'r> for DungeonMaster {
	type Error = ();
	
	fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
		let master_cookie = req.guard::<State<MasterCookie>>()?;
		let master_cookie = master_cookie.read().unwrap(); // TODO handle this?
		let session_manager = req.guard::<State<SessionManager>>()?;
		let session_manager = session_manager.read().unwrap(); // TODO handle this?
		
		let user_cookie = req.cookies().get("session").map(|c| c.value().to_string());
		let p = user_cookie.clone().and_then(|c| session_manager.get(&c));
		match (master_cookie.clone(), p) {
			(None, _) | (_, None) => Outcome::Forward(()),
			(Some(cookie), Some(player)) if cookie.eq(&user_cookie.unwrap()) // unwrap is fine, p is not None
			  => Outcome::Success(DungeonMaster(player.clone())),
			_ => Outcome::Forward(())
		}
	}
}