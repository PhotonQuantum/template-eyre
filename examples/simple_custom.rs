use std::io;

use eyre::{Report, WrapErr};

use template_eyre::Hook;

fn main() -> Result<(), Report> {
    Hook::new(include_str!("../src/templates/demo.hbs"), "")
        .unwrap()
        .install()
        .unwrap();
    connect_server().wrap_err("Unable to talk to daemon")?;
    Ok(())
}

fn connect_server() -> Result<(), Report> {
    perform_io().wrap_err("Failed to connect to socket")
}

fn perform_io() -> Result<(), io::Error> {
    Err(io::Error::from(io::ErrorKind::ConnectionRefused))
}
