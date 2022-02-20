fn main() {
	let mut pworld = xpbd::pworld::PWorld::default();
	pworld.init_test();
	viewer::run(pworld, Default::default(), Default::default());
}
