use std::fmt;
use std::fmt::Write;

use console::Style;
use handlebars::{
    handlebars_helper, Context, Decorator, Handlebars, Helper, HelperDef, JsonValue, PathAndJson,
    RenderContext, RenderError, ScopedJson,
};
use indenter::{indented, Format, Inserter};
use itertools::Itertools;
use serde_json::Value;

handlebars_helper!(InlineIfHelper: |condition: bool, if_branch: Json, else_branch: Json| {
    if condition { if_branch } else { else_branch }.clone()
});

handlebars_helper!(StyleHelper: |style: str, content: str| {
    Style::from_dotted_str(style).apply_to(content).to_string()
});

pub struct IndentHelper;

impl HelperDef for IndentHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        let params: Vec<_> = h.params().iter().map(PathAndJson::value).collect();
        let styled =
            match &params[..] {
                [v] | [Value::Null, v] => indent(as_content(v)?, None),
                [Value::Number(n), v] => indent(
                    as_content(v)?,
                    Some(Format::Numbered {
                        ind: usize::try_from(n.as_u64().ok_or_else(|| {
                            RenderError::new("`indent` helper: number doesn't fit in u64 range")
                        })?)
                        .map_err(|_| {
                            RenderError::new("`indent` helper: number doesn't fit in u64 range")
                        })?,
                    }),
                ),
                [Value::String(s), v] => indent(
                    as_content(v)?,
                    Some(Format::Custom {
                        inserter: &mut *uniform(s.clone()),
                    }),
                ),
                [] => return Err(RenderError::new("`indent` helper: too few parameters")),
                [_, _] => return Err(RenderError::new(
                    "`indent` helper: unexpected parameter type. Accepted: number, string or null.",
                )),
                _ => return Err(RenderError::new("`indent` helper: too many parameters")),
            }?;

        Ok(ScopedJson::Derived(JsonValue::from(styled)))
    }
}

fn uniform(s: String) -> Box<Inserter> {
    Box::new(move |_ln: usize, fmt: &mut dyn fmt::Write| -> fmt::Result {
        fmt.write_str(s.as_str())
    })
}

fn as_content(v: &Value) -> Result<&str, RenderError> {
    v.as_str()
        .ok_or_else(|| RenderError::new("`indent` helper: content is not a string"))
}

fn indent(s: &str, f: Option<Format>) -> Result<String, RenderError> {
    let mut buffer = String::new();
    let mut indent = if let Some(f) = f {
        indented(&mut buffer).with_format(f)
    } else {
        indented(&mut buffer)
    };
    indent
        .write_str(s)
        .map_err(|e| RenderError::from_error("`indent` helper: cannot generate output", e))?;
    Ok(buffer)
}

// a decorator mutates current context data
// adopted from https://github.com/sunng87/handlebars-rust/blob/master/examples/decorator.rs.
pub fn set_decorator(
    d: &Decorator,
    _: &Handlebars,
    ctx: &Context,
    rc: &mut RenderContext,
) -> Result<(), RenderError> {
    // get the input of decorator
    let data_to_set = d.hash();
    // retrieve the json value in current context
    let ctx_data = ctx.data();

    if let Value::Object(m) = ctx_data {
        let mut new_ctx_data = m.clone();

        for (k, v) in data_to_set {
            new_ctx_data.insert((*k).to_string(), v.value().clone());
        }

        rc.set_context(Context::wraps(new_ctx_data)?);
        Ok(())
    } else {
        Err(RenderError::new(
            "`set` decorator: Cannot extend non-object data",
        ))
    }
}

pub struct ConcatHelper;

impl HelperDef for ConcatHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        Ok(ScopedJson::Derived(Value::String(
            h.params()
                .iter()
                .map(|i| i.value().as_str())
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| {
                    RenderError::new("`concat` helper: only accepts string parameter(s)")
                })?
                .into_iter()
                .join(""),
        )))
    }
}

#[cfg(test)]
mod tests {
    use console::style as styled;
    use handlebars::Handlebars;
    use serde_json::{json, Value};

    use crate::helpers::ConcatHelper;
    use crate::{set_decorator, IndentHelper, InlineIfHelper, StyleHelper};

    #[test]
    fn inline_if() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("_if", Box::new(InlineIfHelper));
        handlebars
            .register_template_string("test", r#"{{_if branch "a" "b"}}"#)
            .unwrap();
        assert_eq!(
            handlebars.render("test", &json!({"branch": true})).unwrap(),
            "a"
        );
        assert_eq!(
            handlebars
                .render("test", &json!({"branch": false}))
                .unwrap(),
            "b"
        );
    }

    #[test]
    fn style() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("style", Box::new(StyleHelper));
        assert_eq!(
            handlebars
                .render_template(r#"{{style "cyan" "meow"}}"#, &Value::Null)
                .unwrap(),
            styled("meow").cyan().to_string()
        );
    }

    #[test]
    fn indent() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("indent", Box::new(IndentHelper));
        assert_eq!(
            handlebars
                .render_template(r#"{{indent "meow"}}"#, &Value::Null)
                .unwrap(),
            "    meow"
        );
    }

    #[test]
    fn set() {
        let mut handlebars = Handlebars::new();
        handlebars.register_decorator("set", Box::new(set_decorator));
        assert_eq!(
            handlebars
                .render_template("{{*set cat=\"meow\"}}\n{{cat}}", &json!({}))
                .unwrap(),
            "meow"
        );
    }

    #[test]
    fn concat() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("concat", Box::new(ConcatHelper));
        assert_eq!(
            handlebars
                .render_template(r#"{{concat "a" b}}"#, &json!({"b": "meow"}))
                .unwrap(),
            r#"ameow"#
        );
    }
}
