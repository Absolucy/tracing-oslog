use std::env;

fn main() {
	println!("cargo:rerun-if-changed=build.rs"); // Enable change-tracking

	if env::var("CARGO_CFG_TARGET_VENDOR").expect("failed to get target vendor") != "apple" {
		println!(
			"cargo:warning=tracing-oslog is only available for Apple platforms, it will not log \
			 anything on other platforms!"
		);
		return;
	}

	println!("cargo:rerun-if-changed=wrapper.h");
	println!("cargo:rerun-if-changed=wrapper.c");
	cc::Build::new().file("wrapper.c").compile("wrapper");
}
