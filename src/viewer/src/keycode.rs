use winit::event::VirtualKeyCode as Vkc;

pub fn key2byte(key: Vkc) -> Option<u8> {
	let byte = match key {
		Vkc::A => b'a',
		Vkc::B => b'b',
		Vkc::C => b'c',
		Vkc::D => b'd',
		Vkc::E => b'e',
		Vkc::F => b'f',
		Vkc::G => b'g',
		Vkc::H => b'h',
		Vkc::I => b'i',
		Vkc::J => b'j',
		Vkc::K => b'k',
		Vkc::L => b'l',
		Vkc::M => b'm',
		Vkc::N => b'n',
		Vkc::O => b'o',
		Vkc::P => b'p',
		Vkc::Q => b'q',
		Vkc::R => b'r',
		Vkc::S => b's',
		Vkc::T => b't',
		Vkc::U => b'u',
		Vkc::V => b'v',
		Vkc::W => b'w',
		Vkc::X => b'x',
		Vkc::Y => b'y',
		Vkc::Z => b'z',
		Vkc::Space => b' ',
		_ => return None
	};
	Some(byte)
}
