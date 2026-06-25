use crate::command::Command;
use crate::constants::{
    DEAD_RESPONSE, FOOD_LIFETIME_TIME_UNITS, MAX_PLAYER_LEVEL, RESOURCE_RESPAWN_TIME_UNITS,
    WINNING_PLAYER_COUNT,
};
use crate::game::GameState;
use crate::gui;
use crate::server::client::ClientId;
use crate::world::map::Resource;
use crate::world::resources;
use std::time::Instant;

impl GameState {
    pub fn process_food(&mut self, now: Instant) -> usize {
        let player_ids = self.players.keys().copied().collect::<Vec<_>>();
        let mut processed = 0;
        for player_id in player_ids {
            loop {
                let due = self
                    .players
                    .get(&player_id)
                    .is_some_and(|player| player.next_food_tick <= now);
                if !due {
                    break;
                }
                let consumed = self
                    .players
                    .get_mut(&player_id)
                    .is_some_and(|player| player.inventory.remove(Resource::Food));
                if !consumed {
                    self.kill_player(player_id);
                    processed += 1;
                    break;
                }
                let duration = self.duration(FOOD_LIFETIME_TIME_UNITS);
                if let Some(player) = self.players.get_mut(&player_id) {
                    player.next_food_tick += duration;
                }
                let message = self.players.get(&player_id).map(gui::pin);
                if let Some(message) = message {
                    self.broadcast_to_guis(&message);
                }
                processed += 1;
            }
        }
        processed
    }

    pub fn process_resource_respawn(&mut self, now: Instant) -> usize {
        let mut processed = 0;
        while self.next_resource_spawn <= now {
            let changed = resources::refill_missing_resources(&mut self.map);
            let messages = changed
                .into_iter()
                .map(|(x, y)| gui::bct(&self.map, x, y))
                .collect::<String>();
            self.broadcast_to_guis(&messages);
            self.next_resource_spawn += self.duration(RESOURCE_RESPAWN_TIME_UNITS);
            processed += 1;
        }
        processed
    }

    pub fn kill_player(&mut self, player_id: usize) {
        self.cancel_incantations_for_player(player_id);
        let Some(_player) = self.players.remove(&player_id) else {
            return;
        };
        if let Some(token) = self.player_token(player_id) {
            if let Some(client) = self.clients.get_mut(&token) {
                client.queue_text(DEAD_RESPONSE);
                client.player_id = None;
                client.clear_commands();
                client.close_after_flush = true;
            }
        }
        self.broadcast_to_guis(&gui::pdi(player_id));
    }

    pub fn handle_client_disconnect(&mut self, token: ClientId) {
        let player_id = self.clients.get(&token).and_then(|client| client.player_id);
        if let Some(player_id) = player_id {
            self.cancel_incantations_for_player(player_id);
            if self.players.remove(&player_id).is_some() {
                self.broadcast_to_guis(&gui::pdi(player_id));
            }
        }
    }

    pub fn next_deadline(&self) -> Option<Instant> {
        let command_deadline = self
            .clients
            .values()
            .filter_map(|client| {
                let active = client.active.as_ref()?;
                let player_is_frozen = client
                    .player_id
                    .is_some_and(|player_id| self.is_player_frozen(player_id));
                if player_is_frozen && active.command != Command::Incantation {
                    None
                } else {
                    Some(active.finishes_at)
                }
            })
            .min();
        let food_deadline = self
            .players
            .values()
            .map(|player| player.next_food_tick)
            .min();
        [
            command_deadline,
            food_deadline,
            Some(self.next_resource_spawn),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    pub fn check_victory(&mut self) {
        if self.winner.is_some() {
            return;
        }
        let winner = self.teams.iter().find_map(|team| {
            let elevated = self
                .players
                .values()
                .filter(|player| player.team_name == team.name && player.level == MAX_PLAYER_LEVEL)
                .count();
            (elevated >= WINNING_PLAYER_COUNT).then(|| team.name.clone())
        });
        if let Some(team_name) = winner {
            self.winner = Some(team_name.clone());
            self.broadcast_to_guis(&gui::seg(&team_name));
        }
    }
}
