use std::path::{Path, PathBuf};
use std::io::{BufReader, BufRead, BufWriter, Write};

pub struct Satt {
	files: Vec<Satf>,
}

struct Satf {
	path: String,
	lines: Vec<Vec<u8>>,
	no_eol: bool,
}

fn get_filelist(root: &PathBuf) -> Vec<PathBuf> {
	let mut queue: Vec<PathBuf> = vec![root.clone()];
	let mut filelist: Vec<PathBuf> = Vec::new();
	while let Some(object) = queue.pop() {
		for entry in std::fs::read_dir(object).unwrap() {
			let entry = entry.unwrap();
			let ty = entry.file_type().unwrap();
			let path = entry.path();
			if ty.is_file() {
				filelist.push(path.strip_prefix(&root).unwrap().to_path_buf());
			} else if ty.is_dir() {
				queue.push(path);
			} else {
				panic!("Unknown type: {:?}", path)
			}
		}
	}
	filelist
}

impl Satt {
	pub fn check(&self) {
		for file in self.files.iter() {
			assert!(!file.path.starts_with('/'));
		}
	}

	pub fn archive_filelist(root: &PathBuf, filelist: &[PathBuf]) -> Self {
		let mut files = Vec::new();
		let mut linebuf = Vec::new();
		let mut no_eol = false;
		for filename in filelist.into_iter() {
			let f = std::fs::File::open(root.join(&filename)).unwrap();
			let mut f = BufReader::new(f);
			let mut lines = Vec::new();
			loop {
				let buflen = f.read_until(b'\n', &mut linebuf).unwrap();
				if buflen == 0 {break}
				if linebuf[buflen - 1] == b'\n' {
					linebuf.pop();
				} else {
					no_eol = true;
				}
				lines.push(std::mem::take(&mut linebuf));
			}
			let path = filename.clone()
				.into_os_string()
				.into_string()
				.unwrap();
			let satf = Satf {
				path,
				lines,
				no_eol,
			};
			files.push(satf);
		}
		Satt {files}
	}

	pub fn archive_root(root: &PathBuf) -> Self {
		let mut filelist = get_filelist(root);
		filelist.sort_unstable();
		Self::archive_filelist(root, &filelist)
	}

	pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
		let f = std::fs::File::create(path)?;
		let mut f = BufWriter::new(f);
		for file in self.files.iter() {
			writeln!(f, "{} {}", file.path, file.lines.len())?;
			for line in file.lines.iter() {
				f.write(&line)?;
				writeln!(f)?;
			}
		}
		Ok(())
	}

	pub fn load(path: &str) -> Result<Self, std::io::Error> {
		let f = std::fs::File::open(path)?;
		let mut f = BufReader::new(f);
		let mut linebuf = Vec::new();
		let mut files = Vec::new();
		loop {
			let bufsize = f.read_until(b'\n', &mut linebuf)?;
			if bufsize == 0 {
				break
			}
			let string = String::from_utf8(std::mem::take(&mut linebuf)).unwrap();
			let mut iter = string.split_whitespace();
			let path = iter.next().unwrap().to_string();
			let line_count = iter.next().unwrap().parse::<usize>().unwrap();
			let no_eol = iter.next().is_some();
			let mut lines = Vec::new();
			for _ in 0..line_count {
				let buflen = f.read_until(b'\n', &mut linebuf)?;
				// this test 1. non empty read, 2. last eol
				assert_eq!(linebuf[buflen - 1], b'\n');
				linebuf.pop();
				lines.push(std::mem::take(&mut linebuf));
			}
			let file = Satf {path, lines, no_eol};
			files.push(file);
		}
		let satt = Satt {files};
		satt.check();
		Ok(satt)
	}

	pub fn to_lines(&self) -> Vec<Vec<u8>> {
		let mut result = Vec::new();
		for file in self.files.iter() {
			let mut line = format!("{} {}", file.path, file.lines.len()).into_bytes();
			if file.no_eol {
				line.extend(b" noeol");
			}
			result.push(line);
			for line in file.lines.iter() {
				result.push(line.clone());
			}
		}
		result
	}

	pub fn unarchive(&self, root: &PathBuf) -> Result<(), std::io::Error> {
		if !root.exists() {
			std::fs::create_dir(root)?;
		}
		assert!(root.read_dir().unwrap().next().is_none());
		for file in self.files.iter() {
			let full_path = root.join(&file.path);
			let parent = full_path.parent().unwrap();
			std::fs::create_dir_all(parent)?;
			let f = std::fs::OpenOptions::new()
				.create_new(true)
				.write(true)
				.open(&full_path)
				.unwrap();
			let mut f = BufWriter::new(f);
			let mut first_line = true;
			for line in file.lines.iter() {
				if first_line {
					first_line = false;
				} else {
					writeln!(f)?;
				}
				f.write(line)?;
			}
			if !file.no_eol {
				writeln!(f)?;
			}
		}
		Ok(())
	}
}
