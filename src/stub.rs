use tracing_core::Subscriber;
use tracing_subscriber::{layer::Layer, registry::LookupSpan};

pub struct OsLogger;

impl OsLogger {
	pub fn new<S, C>(_subsystem: S, _category: C) -> Self
	where
		S: AsRef<str>,
		C: AsRef<str>,
	{
		eprintln!("Initializing OsLogger on non-Apple platform! Nothing will be logged by it!");
		Self
	}
}

impl<S> Layer<S> for OsLogger where S: Subscriber + for<'a> LookupSpan<'a> {}
