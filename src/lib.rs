#![deny(
	clippy::complexity,
	clippy::correctness,
	clippy::perf,
	clippy::style,
	clippy::suspicious
)]

#[cfg(target_vendor = "apple")]
mod ffi;
#[cfg(target_vendor = "apple")]
mod logger;
#[cfg(target_vendor = "apple")]
mod visitor;
#[cfg(target_vendor = "apple")]
pub use logger::*;

#[cfg(not(target_vendor = "apple"))]
mod stub;
#[cfg(not(target_vendor = "apple"))]
pub use stub::*;
