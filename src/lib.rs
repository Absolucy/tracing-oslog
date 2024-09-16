#![deny(
	clippy::complexity,
	clippy::correctness,
	clippy::perf,
	clippy::style,
	clippy::suspicious
)]
cfg_if::cfg_if! {
	if #[cfg(target_vendor = "apple")] {
		mod ffi;
		mod logger;
		mod visitor;
		pub use logger::*;
	} else {
		mod stub;
		pub use stub::*;
	}
}
