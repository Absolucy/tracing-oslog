use std::{env, path::PathBuf};

fn main() {
	if env::var("CARGO_CFG_TARGET_VENDOR").expect("failed to get target vendor") != "apple" {
		println!("cargo:warning=tracing-oslog is only available for Apple platforms, it will not log anything on other platforms!");
		return;
	}

	let mut args = Vec::<String>::new();
	if env::var("CARGO_CFG_TARGET_OS").expect("failed to get target os") == "ios" {
		let version = if "macabi" == env::var("CARGO_CFG_TARGET_ABI").unwrap_or_default() {
			"14.0"
		} else {
			"10.0"
		};
		args.push(format!("-miphoneos-version-min={version}"));
	}

	let bindings = bindgen::Builder::default()
		.header("wrapper.h")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
		.allowlist_function("_?os_activity_.*")
		.allowlist_function("os_log_.*")
		.allowlist_function("os_release")
		.allowlist_function("wrapped_.*")
		.allowlist_type("os_activity_.*")
		.allowlist_type("os_log_.*")
		.allowlist_var("_?os_activity_.*")
		.allowlist_var("__dso_handle")
		.clang_args(&args)
		.generate()
		.expect("Unable to generate bindings");

	// Write the bindings to the $OUT_DIR/bindings.rs file.
	let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
	bindings
		.write_to_file(out_path)
		.expect("Couldn't write bindings!");
	cc::Build::new().file("wrapper.c").compile("wrapper");
}
