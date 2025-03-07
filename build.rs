/*
 * fiv - Fast Image Viewer
 * Copyright 2025  Simon Arlott
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use vergen_git2::{Emitter, Git2Builder};

fn main() -> anyhow::Result<()> {
	let path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
	let metadata = cargo_metadata::MetadataCommand::new()
		.manifest_path("./Cargo.toml")
		.current_dir(&path)
		.exec()
		.unwrap();
	let root = metadata.root_package().unwrap();

	println!("cargo::rerun-if-changed=Cargo.toml");
	println!(
		"cargo::rustc-env=COPYRIGHT_YEARS={}",
		root.metadata["copyright-years"].as_str().unwrap()
	);

	let git2 = Git2Builder::default().describe(false, true, None).build()?;

	Emitter::default().add_instructions(&git2)?.emit()
}
