#![deny(
	clippy::complexity,
	clippy::correctness,
	clippy::perf,
	clippy::style,
	clippy::suspicious
)]
cfg_if::cfg_if! {
	if #[cfg(target_vendor = "apple")] {
		#[allow(
			non_upper_case_globals,
			non_camel_case_types,
			non_snake_case,
			dead_code
		)]
		mod bindings;
		mod logger;
		mod visitor;
		pub use logger::*;
	} else {
		mod stub;
		pub use stub::*;
	}
}
