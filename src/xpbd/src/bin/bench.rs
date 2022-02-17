use std::time::SystemTime;
use xpbd::world::World;

fn main() {
	let start = SystemTime::now();
	let mut world = World::default();
	world.init_test();
	let sec = 5;
	world.run(0.005, 200 * sec, 10);
	let duration = SystemTime::now().duration_since(start).unwrap().as_micros();
	eprintln!("{:.3}%", duration as f32 / sec as f32 / 1e4);
}
