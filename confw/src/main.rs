#[macro_use]
extern crate clap;
extern crate notify;
extern crate toml;

#[macro_use]
extern crate sozu_lib as sozu;
extern crate sozu_command_lib as sozu_command;

mod watcher;

use clap::{App, Arg};

use std::time::Duration;

fn main() {
    let matches = App::new("sozuconfw")
        .version(crate_version!())
        .about("watch sozu app routing configs for updates")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true)
            .required(true)
        )
        .arg(Arg::with_name("interval")
            .short("i")
            .long("interval")
            .value_name("SECONDS")
            .help("How often to check for file changes (in seconds). Default is 5 seconds.")
            .default_value("5")
            .takes_value(true)
            .required(false)
        )
        .get_matches();

    let config_file = matches.value_of("config").expect("required config file");
    let update_interval = matches.value_of("interval").map(|value| {
        let parsed_value = value.parse::<u64>().expect("interval must be an integer");
        Duration::from_secs(parsed_value)
    }).expect("required interval");

    watcher::watch(config_file, update_interval);
}