use gtk4::prelude::*;

fn main() {
	let application = gtk4::Application::builder()
		.build();

	application.connect_activate(|app| {
		let window = gtk4::Window::builder()
			.application(app)
			.default_width(640)
			.default_height(480)
			.build();

		let darea = gtk4::DrawingArea::new();
		darea.set_draw_func(|_, ext, _, _| {
			ext.arc(40f64, 40f64, 10f64, 0f64, 2f64 * std::f64::consts::PI);
			ext.fill().unwrap();
			eprintln!("draw");
		});
		window.set_child(Some(&darea));
		let eck = gtk4::EventControllerKey::new();
		eck.connect_key_pressed(glib::clone!(@weak window =>
			@default-return gtk4::Inhibit(false),
			move |_, key, _, _| {
			eprintln!("{:?}", key);
			if key.to_unicode() == Some('q') {
				window.destroy();
			}
			gtk4::Inhibit(false)
		}));
		window.add_controller(&eck);
		window.show();
	});

	application.run();
}
