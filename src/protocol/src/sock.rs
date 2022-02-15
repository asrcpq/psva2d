use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};

use crate::Message;

pub struct SockServer {
	listener: UnixListener,
	stream: Option<UnixStream>,
}

impl Default for SockServer {
	fn default() -> Self {
		let _ = std::fs::remove_file("psva2d.socket");
		let listener = UnixListener::bind("psva2d.socket").unwrap();
		Self {
			listener,
			stream: None,
		}
	}
}

impl SockServer {
	fn listen(&mut self) {
		let stream = self.listener.incoming().next().unwrap().unwrap();
		self.stream = Some(stream);
	}

	pub fn send_msg(&mut self, msg: &[u8]) {
		loop {
			if let Some(stream) = self.stream.as_mut() {
				if stream.write_all(msg).is_ok() {
					return;
				}
			}
			eprintln!("Waiting");
			self.listen();
			eprintln!("Connected");
		}
	}
}

pub struct SockClient {
	stream: Option<UnixStream>,
	buf: Vec<u8>,
	connected: bool,
}

impl Default for SockClient {
	fn default() -> Self {
		Self {
			stream: None,
			buf: vec![0u8; 10_000_000],
			connected: false,
		}
	}
}

impl SockClient {
	pub fn read_msg(&mut self) -> Message {
		if let Some(stream) = self.stream.as_mut() {
			match stream.read(&mut self.buf) {
				Ok(buflen) => {
					if buflen > 0 {
						return Message::from_bytes(&self.buf[..buflen]);
					}
				}
				Err(e) => {
					if e.kind() == std::io::ErrorKind::WouldBlock {
						return Message::Nop;
					}
					panic!("{:?}", e);
				}
			}
		}
		match UnixStream::connect("psva2d.socket") {
			Ok(s) => {
				if !self.connected {
					eprintln!("Connected");
					self.connected = true;
				}
				s.set_nonblocking(true).unwrap();
				self.stream = Some(s);
			}
			Err(e) => {
				if self.connected {
					eprintln!("Disconnected: {:?}", e);
					self.connected = false;
				}
			}
		};
		Message::Nop
	}
}
