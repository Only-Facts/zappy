use super::messages::{bct, msz, pin, plv, ppo, sgt, tna};
use crate::constants::MAX_FREQUENCY;
use crate::entities::player::Player;
use crate::game::GameState;
use crate::server::client::ClientId;

pub fn handle_command(game: &mut GameState, client_id: ClientId, line: &str) {
    let parts = line.split_whitespace().collect::<Vec<_>>();

    let Some(command) = parts.first().copied() else {
        game.queue_to_client(client_id, "suc\n");
        return;
    };

    match command {
        "msz" if parts.len() == 1 => {
            let response = msz(game.map.width, game.map.height);

            game.queue_to_client(client_id, &response);
        }
        "mct" if parts.len() == 1 => {
            handle_map_content(game, client_id);
        }
        "tna" if parts.len() == 1 => {
            let response = game
                .teams
                .iter()
                .map(|team| tna(&team.name))
                .collect::<String>();

            game.queue_to_client(client_id, &response);
        }
        "sgt" if parts.len() == 1 => {
            let response = sgt(game.frequency);

            game.queue_to_client(client_id, &response);
        }
        "bct" => handle_bct(game, client_id, &parts),
        "ppo" => handle_player_query(game, client_id, &parts, ppo),
        "plv" => handle_player_query(game, client_id, &parts, plv),
        "pin" => handle_player_query(game, client_id, &parts, pin),
        "sst" => handle_sst(game, client_id, &parts),
        "msz" | "mct" | "tna" | "sgt" => {
            game.queue_to_client(client_id, "sbp\n");
        }
        _ => {
            game.queue_to_client(client_id, "suc\n");
        }
    }
}

fn handle_map_content(game: &mut GameState, client_id: ClientId) {
    let mut response = String::new();

    for x in 0..game.map.width {
        for y in 0..game.map.height {
            response.push_str(&bct(&game.map, x, y));
        }
    }

    game.queue_to_client(client_id, &response);
}

fn handle_bct(game: &mut GameState, client_id: ClientId, parts: &[&str]) {
    if parts.len() != 3 {
        game.queue_to_client(client_id, "sbp\n");
        return;
    }

    let (Ok(x), Ok(y)) = (parts[1].parse::<usize>(), parts[2].parse::<usize>()) else {
        game.queue_to_client(client_id, "sbp\n");
        return;
    };

    if game.map.get_tile(x, y).is_none() {
        game.queue_to_client(client_id, "sbp\n");
        return;
    }

    let response = bct(&game.map, x, y);

    game.queue_to_client(client_id, &response);
}

fn handle_player_query(
    game: &mut GameState,
    client_id: ClientId,
    parts: &[&str],
    formatter: fn(&Player) -> String,
) {
    if parts.len() != 2 {
        game.queue_to_client(client_id, "sbp\n");
        return;
    }

    let Some(player_id) = parse_hash_id(parts[1]) else {
        game.queue_to_client(client_id, "sbp\n");
        return;
    };

    let Some(player) = game.players.get(&player_id) else {
        game.queue_to_client(client_id, "sbp\n");
        return;
    };

    let response = formatter(player);

    game.queue_to_client(client_id, &response);
}

fn handle_sst(game: &mut GameState, client_id: ClientId, parts: &[&str]) {
    if parts.len() != 2 {
        game.queue_to_client(client_id, "sbp\n");
        return;
    }

    let Ok(frequency) = parts[1].parse::<usize>() else {
        game.queue_to_client(client_id, "sbp\n");
        return;
    };

    if !(1..=MAX_FREQUENCY).contains(&frequency) {
        game.queue_to_client(client_id, "sbp\n");
        return;
    }

    game.frequency = frequency;
    game.broadcast_to_guis(&sgt(frequency));
}

fn parse_hash_id(value: &str) -> Option<usize> {
    value.strip_prefix('#').unwrap_or(value).parse().ok()
}
