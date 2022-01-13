//! Helpers for adding custom fields to error reports

use eyre::Report;
use serde_json::Value;

use crate::Handler;

/// A helper trait to attach additional fields to error reports to be referenced in the handlebars template.
pub trait Section {
    /// The output type
    type Output;
    /// Attach a kv pair to the final error report.
    fn section(self, key: impl ToString, value: impl Into<Value>) -> Self::Output;
}

impl Section for Report {
    type Output = Self;

    fn section(mut self, key: impl ToString, value: impl Into<Value>) -> Self::Output {
        if let Some(handler) = self.handler_mut().downcast_mut::<Handler>() {
            handler.sections.insert(key.to_string(), value.into());
        }

        self
    }
}

impl<T, E> Section for Result<T, E>
where
    E: Into<Report>,
{
    type Output = Result<T, Report>;

    fn section(self, key: impl ToString, value: impl Into<Value>) -> Self::Output {
        self.map_err(Into::into).map_err(|e| e.section(key, value))
    }
}
