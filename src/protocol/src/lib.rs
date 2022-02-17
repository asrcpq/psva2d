pub mod sock;
pub mod pr_model;
use pr_model::PrModel;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Message {
	WorldUpdate(PrModel),
	Nop,
}

impl Message {
	pub fn to_bytes(&self) -> Vec<u8> {
		bincode::serialize(&self).unwrap()
	}

	pub fn from_bytes(bytes: &[u8]) -> Self {
		bincode::deserialize(bytes).unwrap()
	}
}
