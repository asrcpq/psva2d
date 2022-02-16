use std::time::SystemTime;
use xpbd::world::World;

fn main() {
	let start = SystemTime::now();
	let mut world = World::default().bench_mode(100);
	world.run();
	let duration = SystemTime::now().duration_since(start).unwrap().as_micros();
	eprintln!("{}", duration)
}
