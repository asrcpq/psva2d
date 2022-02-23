use crate::V2;

pub struct Posbox {
	pub xmin: f32,
	pub xmax: f32,
	pub ymin: f32,
	pub ymax: f32,
}

impl Posbox {
	pub fn apply(&self, pos: &mut V2) -> bool {
		let xmin = self.xmin;
		let xmax = self.xmax;
		let ymin = self.ymin;
		let ymax = self.ymax;
		let mut flag = false;
		if pos[0] < xmin {
			pos[0] = xmin;
			flag = true;
		} else if pos[0] > xmax {
			pos[0] = xmax;
			flag = true;
		};
		if pos[1] < ymin {
			pos[1] = ymin;
			flag = true;
		} else if pos[1] > ymax {
			pos[1] = ymax;
			flag = true;
		};
		flag
	}
}
