use super::messages::{bct, enw, msz, pin, plv, pnw, sgt, tna};
use crate::game::GameState;

pub fn initial_snapshot(game: &GameState) -> String {
    let mut output = format!(
        "{}{}",
        msz(game.map.width, game.map.height),
        sgt(game.frequency)
    );

    append_map(game, &mut output);
    append_teams(game, &mut output);
    append_players(game, &mut output);
    append_eggs(game, &mut output);

    output
}

fn append_map(game: &GameState, output: &mut String) {
    for x in 0..game.map.width {
        for y in 0..game.map.height {
            output.push_str(&bct(&game.map, x, y));
        }
    }
}

fn append_teams(game: &GameState, output: &mut String) {
    for team in &game.teams {
        output.push_str(&tna(&team.name));
    }
}

fn append_players(game: &GameState, output: &mut String) {
    let mut player_ids = game.players.keys().copied().collect::<Vec<_>>();

    player_ids.sort_unstable();

    for player_id in player_ids {
        if let Some(player) = game.players.get(&player_id) {
            output.push_str(&pnw(player));
            output.push_str(&pin(player));
            output.push_str(&plv(player));
        }
    }
}

fn append_eggs(game: &GameState, output: &mut String) {
    let mut egg_ids = game.eggs.keys().copied().collect::<Vec<_>>();

    egg_ids.sort_unstable();

    for egg_id in egg_ids {
        if let Some(egg) = game.eggs.get(&egg_id) {
            output.push_str(&enw(egg));
        }
    }
}
