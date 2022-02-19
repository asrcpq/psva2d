use std::time::SystemTime;
use xpbd::world::World;

fn main() {
	let start = SystemTime::now();
	let mut world = World::default();
	world.init_test();
	let rframes = 100;
	for _ in 0..rframes {
		world.run();
	}
	let time = rframes as f32 * world.dt * world.ppr as f32;
	let duration = SystemTime::now().duration_since(start).unwrap().as_micros();
	eprintln!("{:.3}%", duration as f32 / time / 1e4);
}
