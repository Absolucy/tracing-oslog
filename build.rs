use std::{env, path::PathBuf};

fn main() {
	if env::var("CARGO_CFG_TARGET_VENDOR").expect("failed to get target vendor") != "apple" {
		println!("cargo:warning=tracing-oslog is only available for Apple platforms, it will not log anything on other platforms!");
		return;
	}

	let bindings = bindgen::Builder::default()
		.header("wrapper.h")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks))
		.allowlist_function("_?os_activity_.*")
		.allowlist_function("os_log_.*")
		.allowlist_function("os_release")
		.allowlist_function("wrapped_.*")
		.allowlist_type("os_activity_.*")
		.allowlist_type("os_log_.*")
		.allowlist_var("_?os_activity_.*")
		.allowlist_var("__dso_handle")
		.generate()
		.expect("Unable to generate bindings");

	// Write the bindings to the $OUT_DIR/bindings.rs file.
	let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
	bindings
		.write_to_file(out_path.join("bindings.rs"))
		.expect("Couldn't write bindings!");
	bindings
		.write_to_file("bindings.rs")
		.expect("Couldn't write bindings!");
	cc::Build::new().file("wrapper.c").compile("wrapper");
}
