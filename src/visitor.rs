use fnv::FnvHashMap;
use std::fmt::Debug;
use tracing_core::field::{Field, Visit};

pub type AttrMap = FnvHashMap<String, String>;

pub struct FieldVisitor<'a> {
	output: &'a mut AttrMap,
}

impl<'a> FieldVisitor<'a> {
	pub fn new(output: &'a mut AttrMap) -> Self {
		FieldVisitor { output }
	}
}

impl<'a> Visit for FieldVisitor<'a> {
	fn record_i64(&mut self, field: &Field, value: i64) {
		self.output
			.insert(field.name().to_string(), value.to_string());
	}

	fn record_u64(&mut self, field: &Field, value: u64) {
		self.output
			.insert(field.name().to_string(), value.to_string());
	}

	fn record_bool(&mut self, field: &Field, value: bool) {
		self.output
			.insert(field.name().to_string(), value.to_string());
	}

	fn record_str(&mut self, field: &Field, value: &str) {
		self.output
			.insert(field.name().to_string(), value.to_string());
	}

	fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
		self.output
			.insert(field.name().to_string(), format!("{:?}", value));
	}
}
