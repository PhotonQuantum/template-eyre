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
    /// Try to get a value from the error report.
    fn get_section(&self, key: impl AsRef<str>) -> Option<&Value>;
    /// Try to get a value from the error report.
    fn get_section_str(&self, key: impl AsRef<str>) -> Option<&str>;
}

impl Section for Report {
    type Output = Self;

    fn section(mut self, key: impl ToString, value: impl Into<Value>) -> Self::Output {
        if let Some(handler) = self.handler_mut().downcast_mut::<Handler>() {
            handler.sections.insert(key.to_string(), value.into());
        }

        self
    }

    fn get_section(&self, key: impl AsRef<str>) -> Option<&Value> {
        self.handler()
            .downcast_ref::<Handler>()
            .and_then(|h| h.sections.get(key.as_ref()))
    }

    fn get_section_str(&self, key: impl AsRef<str>) -> Option<&str> {
        self.get_section(key).and_then(|v| v.as_str())
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

    fn get_section(&self, _key: impl AsRef<str>) -> Option<&Value> {
        None
    }

    fn get_section_str(&self, _key: impl AsRef<str>) -> Option<&str> {
        None
    }
}

#[cfg(test)]
mod tests {
    use eyre::eyre;
    use serde_json::Value;

    use crate::ext::Section;
    use crate::tests::{hack_install, AdhocError};
    use crate::Hook;

    #[test]
    fn attach_to_report() {
        let _guard = hack_install(Hook::simple());
        let report = eyre!(AdhocError::new("boom"))
            .section("a", true)
            .section("b", "key b");
        assert_eq!(report.get_section("a").unwrap(), &Value::Bool(true));
        assert_eq!(
            report.get_section("b").unwrap(),
            &Value::String("key b".into())
        );
        assert!(report.get_section_str("a").is_none());
        assert_eq!(report.get_section_str("b").unwrap(), "key b");
    }
}
