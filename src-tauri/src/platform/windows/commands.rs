use std::process::{Command, Output};

pub fn executable_name(name: &str) -> String {
    if cfg!(windows) && name.eq_ignore_ascii_case("npm") {
        "npm.cmd".to_string()
    } else {
        name.to_string()
    }
}

pub fn command_output(name: &str, arguments: &[&str]) -> std::io::Result<Output> {
    Command::new(executable_name(name)).args(arguments).output()
}

pub fn command_succeeds(name: &str, arguments: &[&str]) -> bool {
    command_output(name, arguments)
        .map(|output| output.status.success())
        .unwrap_or(false)
}
