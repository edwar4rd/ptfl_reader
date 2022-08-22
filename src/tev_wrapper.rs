use std::io::ErrorKind;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use tev_client::{PacketOpenImage, TevClient};

pub struct TevWrappedClient {
    client: Option<(TevClient, Child)>,
}

impl TevWrappedClient {
    pub fn new() -> TevWrappedClient {
        TevWrappedClient { client: None }
    }

    // code is from lib.rs of the tev_client crate
    // modified to better handle tev sub process
    pub fn start_client(&mut self) -> Result<(), String> {
        // check self is already connected to a running client
        match &mut self.client {
            Some((_, child)) => match child.try_wait() {
                Ok(Some(_)) => {
                    self.client = None;
                }
                Ok(None) => {
                    return Ok(());
                }
                Err(_) => {
                    println!("Instance have a unknown status, killing old tev instance");
                    match child.kill() {
                        Ok(_) => {}
                        Err(err) => match err.kind() {
                            ErrorKind::InvalidInput => {}
                            _ => {return Err("Failed ending previous process".to_string())},
                        },
                    };
                    self.client = None;
                }
            },
            None => {}
        }

        println!("Starting new tev client...");
        let mut child = match Command::new("tev")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(err) => {
                return Err(format!("Failed spawning tev, {}", err.to_string()));
            }
        };

        let reader = BufReader::new(child.stdout.take().unwrap());

        let mut read = String::new();
        for line in reader.lines() {
            const PATTERNS: &[&str] = &[
                "Initialized IPC, listening on ",
                "Connected to primary instance at ",
            ];

            let line = line.unwrap();

            for pattern in PATTERNS {
                if let Some(start) = line.find(pattern) {
                    let rest = &line[start + pattern.len()..];

                    // cut of any trailing terminal escape codes
                    let end = rest.find('\u{1b}').unwrap_or(rest.len());
                    let host = &rest[..end];

                    let socket = match TcpStream::connect(host) {
                        Ok(socker) => socker,
                        Err(err) => {
                            return Err(format!("Failed starting TcpStream, {}", err.to_string()));
                        }
                    };
                    self.client = Some((TevClient::wrap(socket), child));
                    return Ok(());
                }
            }

            read.push_str(&line);
            read.push('\n');
        }

        Err("Failed reading IPC Address from tev".to_string())
    }

    pub fn open_image(&mut self, path: String) -> Result<(), String> {
        match self.start_client() {
            Ok(_) => match &mut self.client {
                Some((tev_client, _)) => {
                    match tev_client.send(PacketOpenImage {
                        channel_selector: "",
                        image_name: &path,
                        grab_focus: false,
                    }) {
                        Ok(_) => Ok(()),
                        Err(err) => Err(err.to_string()),
                    }
                }
                None => {
                    panic!("Get a None for self.client after successful call to start_client");
                }
            },

            Err(err) => Err(err),
        }
    }
}
