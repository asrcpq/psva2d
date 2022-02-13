use std::os::unix::net::{UnixListener, UnixStream};
use std::time::Duration;
use std::io::{Read, Write};

use crate::Message;

pub struct SockServer {
	listener: UnixListener,
	stream: Option<UnixStream>,
}

pub struct SockClient {
	stream: Option<UnixStream>,
	buf: Vec<u8>,
}

impl Default for SockClient {
	fn default() -> Self {
		Self {
			stream: None,
			buf: vec![0u8; 10_000_000],
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
					eprintln!("{:?}", e);
					return Message::Nop
				},
			}
		}
		std::thread::sleep(std::time::Duration::from_secs(1));
		match UnixStream::connect("psva2d.socket") {
			Ok(s) => {
				s.set_nonblocking(true).unwrap();
				self.stream = Some(s);
			},
			Err(_) => {
				eprintln!("Waiting connection");
			},
		};
		Message::Nop
	}
}
