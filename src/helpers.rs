use std::fmt;
use std::fmt::Write;

use console::Style;
use handlebars::{
    handlebars_helper, Context, Handlebars, Helper, HelperDef, JsonValue, PathAndJson,
    RenderContext, RenderError, ScopedJson,
};
use indenter::{indented, Format, Inserter};
use serde_json::Value;

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
