fn main() {
	let mut iter = std::env::args();
	iter.next();
	let mut dir = iter.next().unwrap();
	let mut satt = iter.next().unwrap();
	let mut archive = true;
	if dir.ends_with(".satt") {
		std::mem::swap(&mut dir, &mut satt);
		archive = false;
	}
	if archive {
		satt::Satt::archive_root(&dir).save(&satt).unwrap();
	} else {
		satt::Satt::load(&satt).unwrap().unarchive(&dir).unwrap();
	}
}
