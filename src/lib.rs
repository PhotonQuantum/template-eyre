#![allow(clippy::default_trait_access)]
#![warn(missing_docs)]
#![doc = include_str ! ("../README.md")]

use std::error::Error;
use std::fmt::Formatter;

use eyre::{EyreHandler, InstallError};
use handlebars::{Handlebars, RenderError};
use serde_json::{Map, Value};

use crate::helpers::{set_decorator, ConcatHelper, IndentHelper, InlineIfHelper, StyleHelper};
use crate::templates::{COLORED_SIMPLE, SIMPLE};

pub mod ext;
mod helpers;
mod templates;
#[cfg(test)]
mod tests;

/// An eyre reporting hook.
pub struct Hook {
    handlebars: Handlebars<'static>,
}

impl Hook {
    /// Create an eyre handler hook with the given handlebars template.
    ///
    /// # Errors
    /// `RenderError` if given template is invalid.
    pub fn new(eyre_tmpl: impl AsRef<str>) -> Result<Self, RenderError> {
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(id_escape);
        handlebars.register_template_string("eyre", eyre_tmpl)?;
        handlebars.register_helper("style", Box::new(StyleHelper));
        handlebars.register_helper("indent", Box::new(IndentHelper));
        handlebars.register_helper("_if", Box::new(InlineIfHelper));
        handlebars.register_decorator("set", Box::new(set_decorator));
        handlebars.register_helper("concat", Box::new(ConcatHelper));

        let hook = Self { handlebars };

        Ok(hook)
    }

    /// Create a hook to construct a simple eyre error handler.
    #[must_use]
    pub fn simple() -> Self {
        Self::new(SIMPLE).expect("should render")
    }

    /// Create a hook to construct a simple eyre error handler with color support.
    #[must_use]
    pub fn colored_simple() -> Self {
        Self::new(COLORED_SIMPLE).expect("should render")
    }

    /// Install self as the global eyre handling hook via `eyre::set_hook`.
    ///
    /// # Errors
    /// `InstallError` if failed to install self.
    pub fn install(self) -> Result<(), InstallError> {
        eyre::set_hook(Box::new(move |e| Box::new(self.make_handler(e))))
    }

    #[doc(hidden)]
    pub fn make_handler(&self, _e: &(dyn Error + 'static)) -> Handler {
        Handler {
            handlebars: self.handlebars.clone(),
            sections: Default::default(),
        }
    }
}

fn id_escape(s: &str) -> String {
    s.to_string()
}

/// An eyre error handler which reports errors with given handlebars templates.
#[derive(Debug)]
pub struct Handler {
    handlebars: Handlebars<'static>,
    sections: Map<String, Value>,
}

impl EyreHandler for Handler {
    fn debug(&self, error: &(dyn Error + 'static), f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            return core::fmt::Debug::fmt(error, f);
        }

        let fields = {
            let mut map = self.sections.clone();
            map.insert(String::from("error"), error.to_string().into());
            map.insert(
                String::from("sources"),
                std::iter::successors(error.source(), |e| (*e).source())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .into(),
            );
            Value::Object(map)
        };

        match self.handlebars.render("eyre", &fields) {
            Ok(s) => f.write_str(s.as_str()),
            Err(e) => panic!("Error occur when rendering eyre error: {}", e),
        }
    }
}
