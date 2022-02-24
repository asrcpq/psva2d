use material::face::TextureData;
use material::image_model::ImageModelBuilder;
use material::texture_indexer::TextureIndexer;
use xpbd::V2;

fn main() {
	let mut iter = std::env::args();
	iter.next();
	let mut textures: Vec<TextureData> = vec![];
	let mut indexer = TextureIndexer::default();
	let mut pworld = xpbd::pworld::PWorld::default().with_paused();
	let mut y = -4.0;
	for (tid, image_path) in iter.enumerate() {
		let mut imbuilder = ImageModelBuilder::new(tid as i32, &image_path);
		imbuilder.compute_cells();
		imbuilder.expand_cells();
		let pmodel = imbuilder.build_physical_model();
		let cids = pworld.add_model(pmodel.clone(), V2::new(-0., y));
		y -= 4.0;
		let (texture_data, faces) = imbuilder.finish();
		indexer.add_faces(cids, faces);
		textures.push(texture_data);
	}
	viewer::viewer::Viewer::new(pworld, indexer.into_ref(), textures).run();
}
