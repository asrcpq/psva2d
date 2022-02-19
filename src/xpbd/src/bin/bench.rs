use std::time::SystemTime;
use xpbd::pworld::PWorld;

fn main() {
	let start = SystemTime::now();
	let mut pworld = PWorld::default();
	pworld.init_test();
	let rframes = 100;
	for _ in 0..rframes {
		pworld.run();
	}
	let time = rframes as f32 * pworld.dt * pworld.ppr as f32;
	let duration = SystemTime::now().duration_since(start).unwrap().as_micros();
	eprintln!("{:.3}%", duration as f32 / time / 1e4);
}
