use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent, Result};
use toml;
use sozu_command::state::ConfigState;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::fs::File;
use std::io::Read;

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

                        let mut data = String::new();
                        {
                            let mut f = File::open(path).expect("could not open file");
                            f.read_to_string(&mut data).expect("could not read file");
                        }

                        let new_state: ConfigState = toml::from_str(&data).expect("could not read");
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