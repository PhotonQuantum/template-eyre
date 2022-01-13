use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Mutex, MutexGuard};

use eyre::{eyre, EyreHandler, Report};
use once_cell::sync::Lazy;

use crate::ext::Section;
use crate::templates::{COLORED_SIMPLE, SIMPLE};
use crate::Hook;

const DEMO: &str = include_str!("templates/demo.hbs");

macro_rules! report {
    [$($e:literal),+] => {
        _report(&[$($e),+])
    }
}

#[test]
fn simple() {
    must_report("simple", SIMPLE);
    must_report("colored_simple", COLORED_SIMPLE);
    must_report("demo", DEMO);
}

#[test]
fn section() {
    let _guard = hack_install(Hook::new("{{error}} {{key}}").unwrap());

    let r = eyre!("boom").section("key", "somevalue");
    assert_eq!(format!("{:?}", r), "boom somevalue");

    let e: Result<(), _> = Err(AdhocError::new("e")).section("key", "somevalue");
    assert_eq!(format!("{:?}", e.unwrap_err()), "e somevalue");
}

macro_rules! assert_snapshot {
    ($name: expr, $value: expr) => {
        insta::assert_snapshot!($name, format!("{:?}", $value));
    };
}

fn must_report(name: &str, template: &str) {
    let _guard = hack_install(Hook::new(template).unwrap());
    console::set_colors_enabled(true);

    let report = report!["Unable to talk to daemon"];
    println!("{:?}", report);
    assert_snapshot!(format!("{}_1", name), report);

    let report = report![
        "Unable to talk to daemon",
        "Connection refused (os error 61)"
    ];
    println!("{:?}", report);
    assert_snapshot!(format!("{}_2", name), report);

    let report = report![
        "Unable to talk to daemon",
        "Connection refused",
        "os error 61"
    ];
    println!("{:?}", report);
    assert_snapshot!(format!("{}_3", name), report);
}

type ErrorHook =
    Box<dyn Fn(&(dyn Error + 'static)) -> Box<dyn EyreHandler> + Sync + Send + 'static>;

static DYNAMIC_HOOK: Lazy<Mutex<Option<ErrorHook>>> = Lazy::new(|| Mutex::new(None));
static TEST_GUARD: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Debug)]
pub struct AdhocError(pub String);

impl AdhocError {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(e: impl ToString) -> Self {
        Self(e.to_string())
    }
}

impl Display for AdhocError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for AdhocError {}

pub fn hack_install(hook: Hook) -> MutexGuard<'static, ()> {
    let guard = TEST_GUARD.lock().unwrap();
    let mut dynamic_hook = DYNAMIC_HOOK.lock().unwrap();
    if dynamic_hook.is_none() {
        eyre::set_hook(Box::new(|e| {
            (DYNAMIC_HOOK.lock().unwrap().as_ref().unwrap())(e)
        }))
        .unwrap();
    }
    *dynamic_hook = Some(Box::new(move |e| Box::new(hook.make_handler(e))));
    guard
}

fn _report(errors: &'static [&'static str]) -> Report {
    errors
        .iter()
        .rfold::<Option<Report>, _>(None, |x, acc| {
            x.map_or_else(|| Some(eyre!(acc)), |x| Some(x.wrap_err(acc)))
        })
        .unwrap()
}
