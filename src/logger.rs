use crate::{
	bindings::{
		__dso_handle, _os_activity_create, _os_activity_current,
		os_activity_flag_t_OS_ACTIVITY_FLAG_DEFAULT, os_activity_scope_enter,
		os_activity_scope_leave, os_activity_scope_state_s, os_activity_scope_state_t,
		os_activity_t, os_log_create, os_log_t, os_log_type_t, os_log_type_t_OS_LOG_TYPE_DEBUG,
		os_log_type_t_OS_LOG_TYPE_DEFAULT, os_log_type_t_OS_LOG_TYPE_ERROR,
		os_log_type_t_OS_LOG_TYPE_FAULT, os_log_type_t_OS_LOG_TYPE_INFO, os_release,
		wrapped_os_log_default, wrapped_os_log_with_type,
	},
	visitor::{AttributeMap, FieldVisitor},
};
use std::{
	collections::HashMap,
	ffi::CString,
	ops::Deref,
	ptr::addr_of_mut,
	sync::{Mutex, OnceLock},
};
use tracing_core::{
	span::{Attributes, Id},
	Event, Level, Subscriber,
};
use tracing_subscriber::{
	layer::{Context, Layer},
	registry::LookupSpan,
};

static NAMES: OnceLock<Mutex<HashMap<String, CString>>> = OnceLock::new();

struct Activity(os_activity_t);
// lol
unsafe impl Send for Activity {}
unsafe impl Sync for Activity {}
impl Deref for Activity {
	type Target = os_activity_t;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl Drop for Activity {
	fn drop(&mut self) {
		unsafe {
			os_release(self.0 as *mut _);
		}
	}
}

pub struct OsLogger {
	logger: os_log_t,
}

impl Default for OsLogger {
	fn default() -> Self {
		Self {
			logger: unsafe { wrapped_os_log_default() },
		}
	}
}

impl OsLogger {
	/// Initialize a new `OsLogger`, which will output [tracing] events to
	/// os_log on Apple platforms.
	///
	/// # Arguments
	///
	/// * `subsystem` - An identifier string, in reverse DNS notation, that
	///   represents the subsystem that’s performing logging, for example,
	///   `com.your_company.your_subsystem_name`. The subsystem is used for
	///   categorization and filtering of related log messages, as well as for
	///   grouping related logging settings.
	/// * `category` - A category within the specified subsystem. The system
	///   uses the category to categorize and filter related log messages, as
	///   well as to group related logging settings within the subsystem’s
	///   settings. A category’s logging settings override those of the parent
	///   subsystem.
	pub fn new<S, C>(subsystem: S, category: C) -> Self
	where
		S: AsRef<str>,
		C: AsRef<str>,
	{
		let subsystem = CString::new(subsystem.as_ref())
			.expect("failed to construct C string from subsystem name");
		let category = CString::new(category.as_ref())
			.expect("failed to construct C string from category name");
		let logger = unsafe { os_log_create(subsystem.as_ptr(), category.as_ptr()) };
		Self { logger }
	}
}

unsafe impl Sync for OsLogger {}
unsafe impl Send for OsLogger {}

impl<S> Layer<S> for OsLogger
where
	S: Subscriber + for<'a> LookupSpan<'a>,
{
	fn on_new_span(&self, attrs: &Attributes, id: &Id, ctx: Context<S>) {
		let span = ctx.span(id).expect("invalid span, this shouldn't happen");
		let mut extensions = span.extensions_mut();
		if extensions.get_mut::<Activity>().is_none() {
			let mut names = NAMES.get_or_init(Mutex::default).lock().unwrap();
			let metadata = span.metadata();
			let parent_activity = match span.parent() {
				Some(parent) => **parent
					.extensions()
					.get::<Activity>()
					.expect("parent span didn't contain activity wtf"),
				None => addr_of_mut!(_os_activity_current),
			};
			let mut attributes = AttributeMap::default();
			let mut attr_visitor = FieldVisitor::new(&mut attributes);
			attrs.record(&mut attr_visitor);
			let name = {
				let function_name = [metadata.target(), metadata.name()].join("::");
				let full_name = format!(
					"{}({})",
					function_name,
					attributes
						.into_iter()
						.map(|(k, v)| format!("{}: {}", k, v))
						.collect::<Vec<_>>()
						.join(", ")
				);
				names.entry(full_name.clone()).or_insert_with(|| {
					CString::new(full_name).expect("failed to construct C string from span name")
				})
			};
			let activity = unsafe {
				_os_activity_create(
					addr_of_mut!(__dso_handle) as *mut _,
					name.as_ptr(),
					parent_activity,
					os_activity_flag_t_OS_ACTIVITY_FLAG_DEFAULT,
				)
			};
			extensions.insert(Activity(activity));
		}
	}

	fn on_event(&self, event: &Event, ctx: Context<S>) {
		let metadata = event.metadata();
		let level = tracing_level_to_oslog_level(*metadata.level());
		let mut attributes = AttributeMap::default();
		let mut attr_visitor = FieldVisitor::new(&mut attributes);
		event.record(&mut attr_visitor);
		let mut message = String::new();
		if let Some(value) = attributes.remove("message") {
			message = value;
			message.push_str("  ");
		}
		message.push_str(
			&attributes
				.into_iter()
				.map(|(k, v)| format!("{}={}", k, v))
				.collect::<Vec<_>>()
				.join(" "),
		);
		message.retain(|c| c != '\0');
		let message =
			CString::new(message).expect("failed to convert formatted message to a C string");
		if let Some(parent_id) = ctx.current_span().id() {
			let span = ctx
				.span(parent_id)
				.expect("invalid span, this shouldn't happen");
			let mut extensions = span.extensions_mut();
			let activity = extensions
				.get_mut::<Activity>()
				.expect("span didn't contain activity wtf");

			let raw_state = [0u64; 2usize];
			unsafe {
				let state: os_activity_scope_state_s = std::mem::transmute(raw_state);
				let state: os_activity_scope_state_t = &state as *const _ as *mut _;
				os_activity_scope_enter(**activity, state);
				wrapped_os_log_with_type(self.logger, level, message.as_ptr());
				os_activity_scope_leave(state);
			}
		} else {
			unsafe { wrapped_os_log_with_type(self.logger, level, message.as_ptr()) };
		}
	}

	fn on_enter(&self, _id: &Id, _ctx: Context<S>) {}

	fn on_exit(&self, _id: &Id, _ctx: Context<S>) {}

	fn on_close(&self, id: Id, ctx: Context<S>) {
		let span = ctx.span(&id).expect("invalid span, this shouldn't happen");
		let mut extensions = span.extensions_mut();
		extensions
			.remove::<Activity>()
			.expect("span didn't contain activity wtf");
	}
}

impl Drop for OsLogger {
	fn drop(&mut self) {
		unsafe {
			os_release(self.logger as *mut _);
		}
	}
}

/// Convert `tracing::Level` to an os_log-compatible level.
///
/// Note that the semantics of these log levels don't match up 1-to-1, because
/// Apple's os_log is fairly old. See also
/// <https://github.com/Absolucy/tracing-oslog/issues/14>.
fn tracing_level_to_oslog_level(level: Level) -> os_log_type_t {
	match level {
		// Documented semantics:
		// > Use this level to capture information that may be useful
		// > during development or while troubleshooting a specific
		// > problem.
		//
		// This matches pretty well with `Level::TRACE`, especially since
		// `OS_LOG_TYPE_DEBUG` are only emitted through a configuration
		// change (similarly to how `Level::TRACE` is often disabled at
		// compile-time by default).
		Level::TRACE => os_log_type_t_OS_LOG_TYPE_DEBUG,
		// Documented semantics:
		// > Use this level to capture information that may be helpful,
		// > but not essential, for troubleshooting errors.
		//
		// I guess this could match semantically with either
		// `Level::DEBUG` or `Level::INFO`.
		Level::DEBUG => os_log_type_t_OS_LOG_TYPE_INFO,
		// Documented semantics:
		// > Use this level to capture information about things that might
		// > result in a failure.
		//
		// This is arguably slightly incorrect compared to the semantics
		// of `Level::INFO`, however we choose to do this since it's the
		// level that `NSLog` operates on.
		Level::INFO => os_log_type_t_OS_LOG_TYPE_DEFAULT,
		// Documented semantics:
		// > Use this log level to report process-level errors.
		//
		// However, we choose to veer from these, since in practice, users
		// expect warnings to show up as orange in Console.app / Xcode (which
		// this does).
		//
		// Note that this is also the leve that Swift's `Logger::warning`
		// emits at:
		// https://developer.apple.com/documentation/os/logger/warning(_:)
		Level::WARN => os_log_type_t_OS_LOG_TYPE_ERROR,
		// Documented semantics:
		// > Use this level only to capture system-level or multi-process
		// > information when reporting system errors.
		//
		// However, in practice many macOS applications use this logging level
		// for "weaker" errors that this.
		//
		// This logging level also makes the log entry red in Console.app /
		// Xcode, which is what a user would expect of a `Level::ERROR` entry.
		Level::ERROR => os_log_type_t_OS_LOG_TYPE_FAULT,
	}
}
