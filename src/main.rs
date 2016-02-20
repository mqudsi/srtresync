use std::env;
use std::fmt;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::io::prelude::*;
use std::ops::{Add, Sub};

#[derive(Debug)]
struct Timestamp {
	total_milliseconds: i32
}

impl Timestamp {
	fn hours(&self) -> i32 {
		self.total_milliseconds.abs() / (60 * 60 * 1000)
	}

	fn mins(&self) -> i32 {
		(self.total_milliseconds.abs() % (60 * 60 * 1000)) / (60 * 1000)
	}

	fn secs(&self) -> i32 {
		(self.total_milliseconds.abs() % (60 * 1000)) / 1000
	}

	fn msecs(&self) -> i32 {
		self.total_milliseconds.abs() % 1000
	}

	fn parse(text: &str) -> Option<Timestamp> {
		if text.len() == 0 {
			return None;
		}

		let (sign_ptr, mut remainder_ptr) = text.split_at(1);
		let sign = match sign_ptr {
			"-" => -1,
			"+" => 1,
			_ => {
				remainder_ptr = text;
				1
			}
		};

		let mut new_ms = 0;
		let mut i = 0;
		let chunks: std::vec::Vec<&str> = remainder_ptr.split(':').rev().collect();
		for chunk in chunks {
			match i {
				0 => {
						let mut j = 0;
						let temp: std::vec::Vec<&str> = chunk.split(|c| c == ',' || c == '.').collect();
						for chunk2 in temp {
							match j {
								0 => new_ms = new_ms + 1000 * match chunk2.parse::<i32>() {
									Ok(value) => value,
									_ => return None
								},
								1 => new_ms = new_ms + match chunk2.parse::<i32>() {
									Ok(value) => value,
									_ => return None
								},
								_ => return None
							}
							j = j + 1;
						}
				},
				1 => new_ms = new_ms + 60 * 1000 * match chunk.parse::<i32>() {
						Ok(value) => value,
						_ => return None
					},
				2 => new_ms = new_ms + 60 * 60 * 1000 * match chunk.parse::<i32>() {
						Ok(value) => value,
						_ => return None
					},
				_ => return None
			}
			i = i + 1;
		}

		Some(Timestamp {
			total_milliseconds: new_ms * sign
		})
	}

	fn from(hours: i32, mins: i32, secs: i32, msecs: i32) -> Timestamp {
		Timestamp {
			total_milliseconds: hours * (60 * 60 * 1000) + mins * (60 * 1000) + secs * 1000 + msecs
		}
	}

	fn new(msecs: i32) -> Timestamp {
		Timestamp {
			total_milliseconds: msecs
		}
	}
}

impl fmt::Display for Timestamp {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{sign}{hh:>02}:{mm:02}:{ss:02},{ms:03}", sign = if self.total_milliseconds < 0 { "-" } else { "" }, hh = self.hours(), mm = self.mins(), ss = self.secs(), ms = self.msecs())
	}
}

impl<'a> std::ops::Add for &'a Timestamp {
	type Output = Timestamp;
 
	fn add(self, other: &Timestamp) -> Timestamp {
		Timestamp {
			total_milliseconds: self.total_milliseconds + other.total_milliseconds
		}
	}
}

impl<'a> std::ops::Sub for &'a Timestamp {
	type Output = Timestamp;
 
	fn sub(self, other: &Timestamp) -> Timestamp {
		Timestamp {
			total_milliseconds: self.total_milliseconds - other.total_milliseconds
		}
	}
}

fn print_usage() {
	println!("srtresync 1.0 by NeoSmart Technologies. Copyright © 2016.");
	println!("srtresync ./subs.srt [+/-]hh:mm:ss,xxx to correct fixed offset");
	println!("srtresync ./subs.srt hh:mm:ss,xxx-hh:mm:ss,xxx hh:mm:ss,xxx-hh:mm:ss,xxx to correct linear drift");
}

fn calculate_drift(t1: (&Timestamp, &Timestamp), t2: (&Timestamp, &Timestamp)) -> (f32, i32) {
	let drift1 = (t1.1.total_milliseconds - t1.0.total_milliseconds) as f32;
	let drift2 = (t2.1.total_milliseconds - t2.0.total_milliseconds) as f32;

	let x = (drift1 - drift2) as f32 / (t1.0.total_milliseconds - t2.0.total_milliseconds) as f32; //slope, aka drift accumulated over time
	let y = (drift1 - (x * t1.0.total_milliseconds as f32)) as i32;

	(x, y)
}

fn apply_drift(drift: (f32, i32), ts: Timestamp) -> Timestamp {
	let offset = (drift.0 * (ts.total_milliseconds as f32)) as i32 + drift.1;
	Timestamp::new(ts.total_milliseconds + offset)
}

fn invalid_timestamp_format() -> &'static str {
	"The provided timestamp format is incorrect!"
}

fn main() {
	let args: Vec<String> = env::args().collect();

	let (x, y) = match args.len() { 
		1 => {
			return print_usage();
		}
		3 => {
			let offset = match Timestamp::parse(&args[2]) {
				Some(x) => x,
				None => panic!(invalid_timestamp_format())
			};
			(1f32, offset.total_milliseconds)
		},
		4 => {
			let pair1 = &args[2].split("-").map(|x| match Timestamp::parse(x) {
				Some(x) => x,
				None => panic!(invalid_timestamp_format())
			}).collect::<Vec<Timestamp>>();
			let t1 = (&pair1[0], &pair1[1]);

			let pair2 = &args[3].split("-").map(|x| match Timestamp::parse(x) {
				Some(x) => x,
				None => panic!(invalid_timestamp_format())
			}).collect::<Vec<Timestamp>>();
			let t2 = (&pair2[0], &pair2[1]);

			calculate_drift(t1, t2)
		},
		_ => {
			writeln!(std::io::stderr(), "Must provide filename and offset argument(s)!").unwrap();
			return print_usage();
		}
	};

	writeln!(std::io::stderr(), "Offset will be {} scaled by time at a rate of {}", Timestamp::new(0-y), 0f32-x).unwrap();

	let path = Path::new(&args[1]);
	let fd = match File::open(&path) {
		Err(why) => panic!("Couldn't open {}: {}", path.display(), Error::description(&why)),
		Ok(file) => file,
	};

	//match 00:01:25,160 --> 00:01:26,500

	let mut subtitle_index = 0;
	let mut print_next_line = false;
	let file = BufReader::new(&fd);
	for l in file.lines().map(|x| x.unwrap()) {
		let mut timestamp_line: bool;

		if print_next_line && l.len() > 0 {
			println!("{}", &l);
			continue;
		}

		print_next_line = false;
		let captures = l.split("-->").collect::<Vec<&str>>();
		if captures.len() == 2 {
			timestamp_line = true;

			let t1 = match Timestamp::parse(captures[0].trim()) {
				Some(x) => x,
				None => {
					timestamp_line = false;
					Timestamp::new(0)
				}
			};
			let t2 = match Timestamp::parse(captures[1].trim()) {
				Some(x) => x,
				None => {
					timestamp_line = false;
					Timestamp::new(0)
				}
			};

			if timestamp_line {
				let timestamps = (t1, t2);

				let new_ts = (apply_drift((x, y), timestamps.0), apply_drift((x, y), timestamps.1));
				println!("\n{}", subtitle_index);
				println!("{0} --> {1}", new_ts.0, new_ts.1);
				print_next_line = true;

				subtitle_index = subtitle_index + 1;
				continue;
			}
		}
	}
	writeln!(std::io::stderr(), "Processed {} captions", subtitle_index).unwrap();
}
