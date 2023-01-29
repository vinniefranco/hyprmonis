use std::env;
use std::io::BufRead;
use std::io::BufReader;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::process;

#[derive(Clone, Debug)]
pub struct Config {
    pub read_socket: String,
    pub write_socket: String,
}

impl Config {
    pub fn build() -> Result<Config, &'static str> {
        match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(instance_sig) => Ok(Config {
                read_socket: format!("/tmp/hypr/{instance_sig}/.socket2.sock"),
                write_socket: format!("/tmp/hypr/{instance_sig}/.socket.sock"),
            }),
            Err(_) => Err("Hyprland instange not found"),
        }
    }
}

fn hypr_socket_command(command: &str, socket: &String) -> std::io::Result<String> {
    let mut socket: UnixStream = UnixStream::connect(socket)?;
    socket.write_all(command.as_bytes())?;

    let mut response = String::new();
    socket.read_to_string(&mut response)?;

    Ok(response)
}

fn get_initial_state(config: &Config) -> std::io::Result<Vec<String>> {
    let response = hypr_socket_command("monitors", &config.write_socket)?;
    let mut monitor_ids: Vec<String> = Vec::new();

    for line in response.lines() {
        if line.starts_with("Monitor") {
            let monitor_id = &line.split(' ').collect::<Vec<&str>>()[1];
            monitor_ids.push(monitor_id.to_string());
        }
    }

    if monitor_ids.is_empty() {
        eprintln!("Could not find any configured monitors!");
        process::exit(1);
    }

    Ok(monitor_ids)
}

fn listen_to_hyperland(monitors: Vec<String>, config: &Config) -> std::io::Result<()> {
    let socket: UnixStream = UnixStream::connect(&config.read_socket)?;
    let reader = BufReader::new(socket);

    if monitors.len() == 1 {
        hypr_socket_command(
            "keyword monitor eDP-1,highres,0x0,1.88",
            &config.write_socket,
        )?;
    }

    for response in reader.lines().flatten() {
        if response.starts_with("monitoradded") {
            hypr_socket_command("keyword monitor eDP-1,disable", &config.write_socket)?;
        }
    }

    Ok(())
}

pub fn run(config: Config) -> std::io::Result<()> {
    let initial_state = get_initial_state(&config)?;
    listen_to_hyperland(initial_state, &config)?;

    Ok(())
}
