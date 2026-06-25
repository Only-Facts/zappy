use crate::command::Command;
use crate::constants::{GRAPHIC_TEAM_NAME, KO_RESPONSE};
use crate::game::GameState;
use crate::gui;
use crate::server::client::ClientId;
use crate::server::client::ClientState;

pub fn handle_complete_client_lines(game: &mut GameState, token: ClientId) {
    let lines = extract_complete_lines(game, token);
    for line in lines {
        handle_client_line(game, token, &line);
    }
}

fn extract_complete_lines(game: &mut GameState, token: ClientId) -> Vec<String> {
    let Some(client) = game.clients.get_mut(&token) else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    while let Some(line_end) = client.input.iter().position(|byte| *byte == b'\n') {
        let bytes = client.input.drain(..=line_end).collect::<Vec<_>>();
        let mut line = String::from_utf8_lossy(&bytes[..bytes.len() - 1]).to_string();
        if line.ends_with('\r') {
            line.pop();
        }
        lines.push(line);
    }
    lines
}

fn handle_client_line(game: &mut GameState, token: ClientId, line: &str) {
    let Some(state) = game.clients.get(&token).map(|client| client.state) else {
        return;
    };
    match state {
        ClientState::WaitingTeamName => handle_handshake(game, token, line),
        ClientState::Ai => queue_ai_command(game, token, line),
        ClientState::Gui => gui::handle_command(game, token, line),
    }
}

fn handle_handshake(game: &mut GameState, token: ClientId, line: &str) {
    if line == GRAPHIC_TEAM_NAME {
        game.authenticate_gui(token);
        return;
    }
    if line.is_empty() {
        game.queue_to_client(token, KO_RESPONSE);
        return;
    }
    let _ = game.authenticate_ai(token, line);
}

fn queue_ai_command(game: &mut GameState, token: ClientId, line: &str) {
    let command = Command::parse(line);
    if let Some(client) = game.clients.get_mut(&token) {
        let _ = client.enqueue_command(command);
    }
}
