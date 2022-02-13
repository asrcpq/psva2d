use xpbd::world::World;

fn main() {
	let mut world = World::default();
	world.init_test();
	world.run();
}
