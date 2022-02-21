use material::face::TextureData;
use material::image_model::ImageModelBuilder;
use material::texture_indexer::TextureIndexer;
use xpbd::V2;

fn main() {
	let mut iter = std::env::args();
	iter.next();
	let mut textures: Vec<TextureData> = vec![];
	let image_path = iter.next().unwrap();
	let mut pworld = xpbd::pworld::PWorld::default().with_paused();
	let indexer = TextureIndexer::default();
	let mut imbuilder = ImageModelBuilder::new(0, indexer, &image_path);
	imbuilder.compute_cells();
	let pmodel = imbuilder.build_physical_model();
	pworld.add_model(pmodel, V2::new(0.0, -3.0));
	imbuilder.compute_cells();
	let pmodel = imbuilder.build_physical_model();
	pworld.add_model(pmodel, V2::new(0.5, -5.0));
	let (texture_data, indexer) = imbuilder.finish();
	textures.push(texture_data);
	viewer::run(pworld, indexer, textures);
}
