use std::path::PathBuf;
use std::str::FromStr;

fn main() {
	let mut iter = std::env::args();
	iter.next();
	let src = PathBuf::from_str(&iter.next().unwrap()).unwrap();
	let satt = iter.next().unwrap();
	let dst = PathBuf::from_str(&iter.next().unwrap()).unwrap();
	satt::Satt::archive_root(&src).save(&satt).unwrap();
	satt::Satt::load(&satt).unwrap().unarchive(&dst).unwrap();
}
