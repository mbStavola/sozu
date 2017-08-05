use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent, Result};
use toml;
use sozu::messages::{CertFingerprint, CertificateAndKey, Order, HttpFront, HttpsFront, Instance};
use sozu_command::state::{AppId, ConfigState, ConfigStateBuilder};
use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use serde::Deserialize;

pub fn watch(config_file: &str, update_interval: Duration) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, update_interval)?;
    watcher.watch(config_file, RecursiveMode::NonRecursive)?;

    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::Write(path) => {
                        println!("File written, generating diff.");
                        let new_state: ConfigState = parse_config_file(&path);
                        let old_state: ConfigState = unimplemented!();

                        println!("Sending new configuration to server.");
                        let orders = old_state.diff(&new_state);
                    }
                    DebouncedEvent::Rename(old_path, new_path) => {
                        // Track changed filename
                        println!("File renamed:\n\tOld path: {}\n\tNew path: {}",
                                 old_path.to_str().expect("missing old path"),
                                 new_path.to_str().expect("missing new path")
                        );
                        watcher.unwatch(old_path)?;
                        watcher.watch(new_path, RecursiveMode::NonRecursive)?;
                    }
                    _ => {
                        // Error
                    }
                }
            }
            Err(e) => {
                println!("watch error: {:?}", e);
                //break;
            }
        }
    }
}

fn parse_config_file(path: &PathBuf) -> ConfigState {
    let mut data = String::new();
    {
        let mut f = File::open(path).expect("could not open file");
        f.read_to_string(&mut data).expect("could not read file");
    }

    parse_config(&data)
}

fn parse_config(data: &str) -> ConfigState {
    let tables: Vec<RoutingConfig> = toml::from_str(data).expect("could not parse config");

    let mut instances: HashMap<AppId, Vec<Instance>> = HashMap::new();
    let mut http_fronts: HashMap<AppId, Vec<HttpFront>> = HashMap::new();
    let mut https_fronts: HashMap<AppId, Vec<HttpsFront>> = HashMap::new();
    let mut certificates: HashMap<CertFingerprint, CertificateAndKey> = HashMap::new();
    let mut http_addresses: Vec<(String, u16)> = Vec::new();
    let mut https_addresses: Vec<(String, u16)> = Vec::new();

    for table in tables {
        let app_id = &table.app_id.to_owned();
        let hostname = &table.hostname.to_owned();
        let path_begin = &table.path_begin.unwrap_or("/").to_owned();
        table.certificate;

        let mut authorities: Vec<(String, u16)> = table.backends.iter().map(|authority| {
            let mut split = authority.split(":");

            let host = split.next().expect("host is required").to_owned();
            let port = split.next().unwrap_or("80").parse::<u16>().expect("could not parse port");

            (host, port)
        }).collect();

        if table.frontends.contains(&"HTTP") {
            http_fronts.entry(app_id.clone())
                .or_insert(Vec::new())
                .push(HttpFront {
                    app_id: app_id.clone(),
                    hostname: hostname.clone(),
                    path_begin: path_begin.clone()
                });

            http_addresses.append(&mut authorities)
        }

        //        if table.frontends.contains(&"HTTPS") {
        //            https_fronts.entry(app_id.clone())
        //                .or_insert(Vec::new())
        //                .push(HttpsFront {
        //                    app_id: app_id.clone(),
        //                    hostname: hostname.clone(),
        //                    path_begin: path_begin.clone(),
        //                    fingerprint:
        //                });
        //
        //            https_addresses.append(&mut authorities)
        //        }

        {
            let mut backends: Vec<Instance> = authorities.iter().map(|authority| {
                let (ref host, port): (String, u16) = *authority;

                Instance {
                    app_id: app_id.clone(),
                    ip_address: host.clone(),
                    port: port
                }
            }).collect();

            instances.entry(app_id.clone()).or_insert(Vec::new()).append(&mut backends);
        }
    }

    ConfigStateBuilder::new(instances)
        .http_fronts(http_fronts)
        .https_fronts(https_fronts)
        .certificates(certificates)
        .http_addresses(http_addresses)
        .https_addresses(https_addresses)
        .build()
}

/**
hostname   = "lolcatho.st"
path_begin = "/api" # optional
certificate = "../lib/assets/certificate.pem" # optional
key = "../lib/assets/key.pem" # optional
certificate_chain = "../lib/assets/certificate_chain.pem" # optional
frontends = ["HTTP", "HTTPS"] # list of proxy tags
backends  = [ "127.0.0.1:1026" ] # list of IP/port
*/
#[derive(Debug, Default, Clone, Deserialize)]
struct RoutingConfig<'a> {
    app_id: &'a str,
    hostname: &'a str,
    path_begin: Option<&'a str>,
    certificate: Option<&'a str>,
    key: Option<&'a str>,
    certificate_chain: Option<&'a str>,
    frontends: Vec<&'a str>,
    backends: Vec<&'a str>
}