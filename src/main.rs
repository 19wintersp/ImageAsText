use std::fs;
use std::io::{ self, Read };
use std::str::FromStr;

use braille::{ BRAILLE, BOX };
use clap::{ clap_app, Error, ErrorKind };
use image::{ DynamicImage, GenericImageView, imageops, Rgba };

const RED_LUM: f32   = 0.299f32;
const GREEN_LUM: f32 = 0.587f32;
const BLUE_LUM: f32  = 0.114f32;
const THRESHOLD: u8  = 96u8;

const ASCII_CHARS: [char; 8] = [' ', '.', ',', '-', '/', 'O', '#', '@'];

fn main() {
	let matches = clap_app!(app =>
		(name: env!("CARGO_PKG_NAME"))
		(version: env!("CARGO_PKG_VERSION"))
		(author: env!("CARGO_PKG_AUTHORS"))
		(about: env!("CARGO_PKG_DESCRIPTION"))
		(@setting ArgRequiredElseHelp)
		(@arg INPUT: +required +takes_value "Specify input file")
		(@arg output: -o --output +takes_value "Specify output file")
		(@arg size: -s --size +takes_value "Specify maximum dimension size")
		(@arg threshold: -t --threshold +takes_value "Specify brightness threshold")
		(@arg double: -d --("double-width") "Write characters twice")
		(@arg braille: -b --braille "Use braille instead of ASCII")
		(@arg blocks: -B --blocks conflicts_with[braille] "Use blocks instead of ASCII")
	).get_matches();

	let input = matches.value_of("INPUT").unwrap();

	let input_data = if input == "." {
		let mut buf = Vec::new();
		io::stdin()
			.read_to_end(&mut buf)
			.map(|_| buf)
	} else {
		fs::read(input)
	};

	let input_data = match input_data {
		Ok(data) => data,
		Err(error) => {
			let error: Error = error.into();
			error.exit()
		},
	};

	let input_data = input_data.as_slice();
	let mut image = match image::load_from_memory(input_data) {
		Ok(image) => image,
		Err(error) => {
			Error {
				kind: ErrorKind::Io,
				message: error.to_string(),
				info: None,
			}.exit()
		},
	};

	if let Some(size) = matches.value_of("size") {
		let dim = match u32::from_str(size) {
			Ok(int) => int,
			Err(_) => Error {
				kind: ErrorKind::InvalidValue,
				message: "Value for size is not a valid integer".into(),
				info: None,
			}.exit(),
		};

		image = image.resize(
			dim, dim,
			imageops::FilterType::CatmullRom
		);
	}

	let thresh = if let Some(thresh) = matches.value_of("threshold") {
		match u8::from_str(thresh) {
			Ok(int) => int,
			Err(_) => Error {
				kind: ErrorKind::InvalidValue,
				message: "Value for threshold is not a valid integer".into(),
				info: None,
			}.exit(),
		}
	} else {
		THRESHOLD
	};
	
	let double = matches.is_present("double");

	println!(
		"{}",
		if matches.is_present("braille") {
			to_braille(image, thresh, double)
		} else if matches.is_present("blocks") {
			to_blocks(image, thresh, double)
		} else {
			to_ascii(image, double)
		},
	);
}

fn pixel_brightness(pixel: Rgba<u8>) -> u8 {
	let lum = (RED_LUM * pixel[0] as f32)
		+ (GREEN_LUM * pixel[1] as f32)
		+ (BLUE_LUM * pixel[2] as f32);
	
	lum as u8
}

fn is_dark(image: DynamicImage, x: u32, y: u32, t: u8) -> usize {
	let pixel = if image.in_bounds(x, y) {
		image.get_pixel(x, y)
	} else {
		Rgba([ 255, 255, 255, 0 ])
	};

	let lum = (RED_LUM * pixel[0] as f32)
		+ (GREEN_LUM * pixel[1] as f32)
		+ (BLUE_LUM * pixel[2] as f32);
	
	if (lum as u8) < t { 1 } else { 0 }
}

fn to_braille(image: DynamicImage, t: u8, double: bool) -> String {
	let mut out = String::new();

	let ch = (image.height() as f32 / 4f32).ceil() as u32;
	let cw = (image.width() as f32 / 2f32).ceil() as u32;

	for cy in 0..ch {
		for cx in 0..cw {
			let x = cx * 2;
			let y = cy * 4;

			let ch = BRAILLE
					[is_dark(image.clone(), x + 0, y + 0, t)][is_dark(image.clone(), x + 1, y + 0, t)]
					[is_dark(image.clone(), x + 0, y + 1, t)][is_dark(image.clone(), x + 1, y + 1, t)]
					[is_dark(image.clone(), x + 0, y + 2, t)][is_dark(image.clone(), x + 1, y + 2, t)]
					[is_dark(image.clone(), x + 0, y + 3, t)][is_dark(image.clone(), x + 1, y + 3, t)];
			if double { out.push(ch); }
			out.push(ch);
		}

		out.push('\n')
	}

	out
}

fn to_blocks(image: DynamicImage, t: u8, double: bool) -> String {
	let mut out = String::new();

	let ch = (image.height() as f32 / 2f32).ceil() as u32;
	let cw = (image.width() as f32 / 2f32).ceil() as u32;

	for cy in 0..ch {
		for cx in 0..cw {
			let x = cx * 2;
			let y = cy * 2;

			let ch = BOX
					[is_dark(image.clone(), x + 0, y + 0, t)][is_dark(image.clone(), x + 1, y + 0, t)]
					[is_dark(image.clone(), x + 0, y + 1, t)][is_dark(image.clone(), x + 1, y + 1, t)];
			if double { out.push(ch); }
			out.push(ch);
		}

		out.push('\n')
	}

	out
}

fn to_ascii(image: DynamicImage, double: bool) -> String {
	let mut out = String::new();

	for y in 0..image.height() {
		for x in 0..image.width() {
			let brightness = pixel_brightness(image.get_pixel(x, y));

			let ch = ASCII_CHARS[(brightness / 32) as usize];
			if double { out.push(ch); }
			out.push(ch);
		}

		out.push('\n');
	}

	out
}
