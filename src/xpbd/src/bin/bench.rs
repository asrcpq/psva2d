use std::time::SystemTime;
use xpbd::world::World;

fn main() {
	let start = SystemTime::now();
	let mut world = World::default();
	world.run(0.005, 500, 10);
	let duration = SystemTime::now().duration_since(start).unwrap().as_micros();
	eprintln!("{}", duration)
}
