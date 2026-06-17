use crate::client::{Client, ClientState};
use crate::config::Config;
use crate::constants::{
    CARRIAGE_RETURN, EMPTY_LINE, GRAPHIC_TEAM_NAME, KO_RESPONSE, LINE_DELIMITER, RESPONSE_END,
    RESPONSE_SEPARATOR,
};
use std::io::Write;

pub fn handle_complete_client_lines(client: &mut Client, config: &Config) {
    while let Some(line_end_index) = client.input_buffer.find(LINE_DELIMITER) {
        let line = extract_client_line(client, line_end_index);

        if line == EMPTY_LINE {
            continue;
        }

        handle_client_line(client, &line, config);
    }
}

fn extract_client_line(client: &mut Client, line_end_index: usize) -> String {
    let mut line = client.input_buffer[..line_end_index].to_string();

    client.input_buffer.drain(..=line_end_index);

    if line.ends_with(CARRIAGE_RETURN) {
        line.pop();
    }

    line
}

fn handle_client_line(client: &mut Client, line: &str, config: &Config) {
    match client.state {
        ClientState::WaitingTeamName => {
            handle_handshake_line(client, line, config);
        }
        ClientState::Ai => {
            println!("AI command from {:?}: {}", client.team_name, line);
        }
        ClientState::Gui => {
            println!("GUI command: {}", line);
        }
    }
}

fn handle_handshake_line(client: &mut Client, line: &str, config: &Config) {
    if line == GRAPHIC_TEAM_NAME {
        authenticate_gui_client(client);
        return;
    }

    if is_valid_team_name(line, config) {
        authenticate_ai_client(client, line, config);
        return;
    }

    reject_unknown_team(client, line);
}

fn authenticate_gui_client(client: &mut Client) {
    client.state = ClientState::Gui;
    client.team_name = None;

    println!("GUI client authenticated");
}

fn is_valid_team_name(team_name: &str, config: &Config) -> bool {
    config.teams.iter().any(|team| team == team_name)
}

fn authenticate_ai_client(client: &mut Client, team_name: &str, config: &Config) {
    client.state = ClientState::Ai;
    client.team_name = Some(team_name.to_string());

    let response = format!(
        "{}{}{}{}{}{}",
        config.clients_nb,
        RESPONSE_END,
        config.width,
        RESPONSE_SEPARATOR,
        config.height,
        RESPONSE_END
    );

    if let Err(error) = client.socket.write_all(response.as_bytes()) {
        eprintln!("Failed to send AI handshake response: {}", error);
    }

    println!("AI client authenticated for team {}", team_name);
}

fn reject_unknown_team(client: &mut Client, team_name: &str) {
    eprintln!("Unknown team name: {}", team_name);

    if let Err(error) = client.socket.write_all(KO_RESPONSE) {
        eprintln!("Failed to send rejection response: {}", error);
    }
}
