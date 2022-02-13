pub mod sock;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Message {
	WorldUpdate(Vec<[f32; 2]>),
	Nop,
}

impl Message {
	pub fn to_bytes(&self) -> Vec<u8> {
		bincode::serialize(&self).unwrap()
	}

	pub fn from_bytes(bytes: &[u8]) -> Self {
		bincode::deserialize(&bytes[..]).unwrap()
	}
}
