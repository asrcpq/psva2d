use crate::V2;

use std::cell::RefCell;
use std::rc::Rc;

pub type PRef = Rc<RefCell<Particle>>;

#[derive(Copy, Clone)]
pub struct Particle {
	imass: f32,
	pos: V2,
	ppos: V2,
	accel: V2,
}

impl Particle {
	pub fn new_ref(mass: f32, pos: V2, accel: V2) -> PRef {
		let result = Self {
			imass: 1f32 / mass, // inf is handled
			pos,
			ppos: pos,
			accel,
		};
		Rc::new(RefCell::new(result))
	}

	pub fn get_pos(&self) -> V2 { self.pos }

	pub fn update(&mut self, t: f32) {
		if self.imass == 0f32 { return } // fixed
		let ppos = self.pos;
		let mut dv = self.accel * t;
		self.pos += self.pos - self.ppos + dv * t;
		self.ppos = ppos;
		eprintln!("{:?} {:?}", dv, self.pos);
	}
}
