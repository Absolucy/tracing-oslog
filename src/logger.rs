use crate::{
	ffi::{
		__dso_handle, _os_activity_create, _os_activity_current, mach_header,
		os_activity_flag_t_OS_ACTIVITY_FLAG_DEFAULT, os_activity_scope_enter,
		os_activity_scope_leave, os_activity_scope_state_s, os_activity_t, os_log_create, os_log_t,
		os_log_type_t_OS_LOG_TYPE_DEBUG, os_log_type_t_OS_LOG_TYPE_ERROR,
		os_log_type_t_OS_LOG_TYPE_INFO, os_release, wrapped_os_log_with_type,
	},
	visitor::{AttributeMap, FieldVisitor},
};
use fnv::FnvHashMap;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::{ffi::CString, ops::Deref};
use tracing_core::{
	span::{Attributes, Id},
	Event, Level, Subscriber,
};
use tracing_subscriber::{
	layer::{Context, Layer},
	registry::LookupSpan,
};

static NAMES: Lazy<Mutex<FnvHashMap<String, CString>>> =
	Lazy::new(|| Mutex::new(FnvHashMap::default()));

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
	state: os_activity_scope_state_s,
}

impl OsLogger {
	/// Initialize a new `OsLogger`, which will output [tracing] events to os_log on Apple platforms.
	///
	/// # Arguments
	///
	/// * `subsystem` - An identifier string, in reverse DNS notation, that represents the subsystem that’s performing logging, for example, `com.your_company.your_subsystem_name`. The subsystem is used for categorization and filtering of related log messages, as well as for grouping related logging settings.
	/// * `category` - A category within the specified subsystem. The system uses the category to categorize and filter related log messages, as well as to group related logging settings within the subsystem’s settings. A category’s logging settings override those of the parent subsystem.
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
		let state = unsafe { std::mem::zeroed() };
		Self { logger, state }
	}
}

unsafe impl Sync for OsLogger {}
unsafe impl Send for OsLogger {}

impl<S> Layer<S> for OsLogger
where
	S: Subscriber + for<'a> LookupSpan<'a>,
{
	fn new_span(&self, attrs: &Attributes, id: &Id, ctx: Context<S>) {
		let span = ctx.span(id).expect("invalid span, this shouldn't happen");
		let mut extensions = span.extensions_mut();
		if extensions.get_mut::<Activity>().is_none() {
			let mut names = NAMES.lock();
			let metadata = span.metadata();
			let parent_activity = match span.parent() {
				Some(parent) => **parent
					.extensions()
					.get::<Activity>()
					.expect("parent span didn't contain activity wtf"),
				None => unsafe { &mut _os_activity_current as *mut _ },
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
					&mut __dso_handle as *mut mach_header as *mut _,
					name.as_ptr(),
					parent_activity,
					os_activity_flag_t_OS_ACTIVITY_FLAG_DEFAULT,
				)
			};
			extensions.insert(Activity(activity));
		}
	}

	fn on_event(&self, event: &Event, _ctx: Context<S>) {
		let metadata = event.metadata();
		let level = match *metadata.level() {
			Level::TRACE => os_log_type_t_OS_LOG_TYPE_DEBUG,
			Level::DEBUG => os_log_type_t_OS_LOG_TYPE_DEBUG,
			Level::INFO => os_log_type_t_OS_LOG_TYPE_INFO,
			Level::WARN => os_log_type_t_OS_LOG_TYPE_ERROR,
			Level::ERROR => os_log_type_t_OS_LOG_TYPE_ERROR,
		};
		let mut attributes = AttributeMap::default();
		let mut attr_visitor = FieldVisitor::new(&mut attributes);
		event.record(&mut attr_visitor);
		let mut message = String::new();
		if let Some(value) = attributes.remove(&"message".to_string()) {
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
		let message =
			CString::new(message).expect("failed to convert formatted message to a C string");
		unsafe { wrapped_os_log_with_type(self.logger, level, message.as_ptr()) };
	}

	fn on_enter(&self, id: &Id, ctx: Context<S>) {
		let span = ctx.span(id).expect("invalid span, this shouldn't happen");
		let mut extensions = span.extensions_mut();
		let activity = extensions
			.get_mut::<Activity>()
			.expect("span didn't contain activity wtf");
		unsafe {
			os_activity_scope_enter(**activity, &self.state as *const _ as *mut _);
		}
	}

	fn on_exit(&self, _id: &Id, _ctx: Context<S>) {
		unsafe {
			os_activity_scope_leave(&self.state as *const _ as *mut _);
		}
	}

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
