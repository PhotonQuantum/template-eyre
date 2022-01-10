use std::error::Error;
use std::fmt::Formatter;

use eyre::{EyreHandler, InstallError};
use handlebars::{Handlebars, RenderError};
use serde_json::json;

use crate::helpers::{set_decorator, IndentHelper, InlineIfHelper, StyleHelper};
use crate::templates::{COLORED_SIMPLE, SIMPLE};

mod helpers;
mod templates;
#[cfg(test)]
mod tests;

pub struct Hook {
    handlebars: Handlebars<'static>,
}

impl Hook {
    pub fn new(
        eyre_tmpl: impl AsRef<str>,
        panic_tmpl: impl AsRef<str>,
    ) -> Result<Self, RenderError> {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("eyre", eyre_tmpl)?;
        handlebars.register_template_string("panic", panic_tmpl)?;
        handlebars.register_helper("style", Box::new(StyleHelper));
        handlebars.register_helper("indent", Box::new(IndentHelper));
        handlebars.register_helper("_if", Box::new(InlineIfHelper));
        handlebars.register_decorator("set", Box::new(set_decorator));

        let hook = Self { handlebars };
        hook.validate()?;

        Ok(hook)
    }

    pub fn simple() -> Self {
        Self::new(SIMPLE, "").expect("should render")
    }

    pub fn colored_simple() -> Self {
        Self::new(COLORED_SIMPLE, "").expect("should render")
    }

    pub fn install(self) -> Result<(), InstallError> {
        eyre::set_hook(Box::new(move |e| Box::new(self.make_handler(e))))
    }

    fn validate(&self) -> Result<(), RenderError> {
        self.handlebars.render(
            "eyre",
            &json!({
                "error": "<error>",
                "sources": ["<source>"]
            }),
        )?;
        self.handlebars.render(
            "eyre",
            &json!({
                "error": "<error>",
                "sources": ["<source1>", "<source2>"]
            }),
        )?;
        self.handlebars.render(
            "eyre",
            &json!({
                "error": "<error>",
                "sources": ["<source1>", "<source2>", "<source3>"]
            }),
        )?;
        Ok(())
    }

    #[doc(hidden)]
    pub fn make_handler(&self, _e: &(dyn Error + 'static)) -> Handler {
        Handler {
            handlebars: self.handlebars.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Handler {
    handlebars: Handlebars<'static>,
}

impl EyreHandler for Handler {
    fn debug(&self, error: &(dyn Error + 'static), f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            return core::fmt::Debug::fmt(error, f);
        }

        match self.handlebars.render("eyre", &json!({
            "error": error.to_string(),
            "sources": std::iter::successors(error.source(), |e| (*e).source()).map(ToString::to_string).collect::<Vec<_>>()
        })) {
            Ok(s) => f.write_str(s.as_str()),
            Err(e) => panic!("Error occur when rendering eyre error: {}", e)
        }
    }
}
