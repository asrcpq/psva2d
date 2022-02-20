use material::image_model::ImageModelBuilder;
use xpbd::V2;

fn main() {
	let mut iter = std::env::args();
	iter.next();
	let image_path = iter.next().unwrap();
	let mut pworld = xpbd::pworld::PWorld::default();
	let mut imbuilder = ImageModelBuilder::load_image(&image_path);
	imbuilder.compute_cells();
	let pmodel = imbuilder.build_physical_model();
	pworld.add_model(pmodel, V2::new(0.0, 1.0));
	viewer::run(pworld);
}
