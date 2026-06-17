mod client;
mod config;
mod constants;
mod network;
mod protocol;

use crate::client::Client;
use crate::config::parse_args;
use crate::constants::{ERROR_EXIT, EVENTS_CAPACITY, FIRST_CLIENT_TOKEN_ID, SERVER_TOKEN, USAGE};
use crate::network::{accept_new_clients, create_listener, read_from_client};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io;

fn main() -> io::Result<()> {
    let config = match parse_args() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Error: {}", error);
            eprintln!("{}", USAGE);
            std::process::exit(ERROR_EXIT);
        }
    };

    println!("Starting server with config: {:?}", config);

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(EVENTS_CAPACITY);

    let mut listener = create_listener(config.port)?;

    poll.registry()
        .register(&mut listener, SERVER_TOKEN, Interest::READABLE)?;

    let mut clients: HashMap<Token, Client> = HashMap::new();
    let mut next_token_id = FIRST_CLIENT_TOKEN_ID;

    println!("Server listening on port {}", config.port);

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            match event.token() {
                SERVER_TOKEN => {
                    accept_new_clients(&mut listener, &mut poll, &mut clients, &mut next_token_id);
                }
                client_token => {
                    read_from_client(client_token, &mut clients, &config);
                }
            }
        }
    }
}
