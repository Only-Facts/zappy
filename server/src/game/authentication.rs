use crate::constants::{FOOD_LIFETIME_TIME_UNITS, ID_INCREMENT, KO_RESPONSE};
use crate::entities::egg::Egg;
use crate::entities::player::{Orientation, Player};
use crate::game::GameState;
use crate::gui;
use crate::server::client::{ClientId, ClientState};
use rand::seq::SliceRandom;
use std::time::Instant;

struct AiAuthentication {
    egg_id: usize,
    player: Player,
}

impl GameState {
    pub fn available_egg_count(&self, team_name: &str) -> usize {
        self.eggs
            .values()
            .filter(|egg| egg.team_name == team_name)
            .count()
    }

    pub fn has_team(&self, team_name: &str) -> bool {
        self.teams.iter().any(|team| team.name == team_name)
    }

    pub fn authenticate_gui(&mut self, client_id: ClientId) {
        if let Some(client) = self.clients.get_mut(&client_id) {
            client.state = ClientState::Gui;
            client.team_name = None;
            client.player_id = None;
        }

        let snapshot = gui::initial_snapshot(self);

        self.queue_to_client(client_id, &snapshot);
    }

    pub fn authenticate_ai(&mut self, client_id: ClientId, team_name: &str) -> bool {
        match self.prepare_ai_authentication(client_id, team_name) {
            Some(authentication) => {
                self.complete_ai_authentication(client_id, authentication);
                true
            }
            None => {
                self.reject_authentication(client_id);
                false
            }
        }
    }

    fn prepare_ai_authentication(
        &mut self,
        client_id: ClientId,
        team_name: &str,
    ) -> Option<AiAuthentication> {
        if !self.clients.contains_key(&client_id) || !self.has_team(team_name) {
            return None;
        }

        let egg = self.take_random_team_egg(team_name)?;
        let player = self.create_player_from_egg(team_name, &egg);

        Some(AiAuthentication {
            egg_id: egg.id,
            player,
        })
    }

    fn take_random_team_egg(&mut self, team_name: &str) -> Option<Egg> {
        let egg_id = self.random_team_egg_id(team_name)?;

        self.eggs.remove(&egg_id)
    }

    fn random_team_egg_id(&self, team_name: &str) -> Option<usize> {
        let egg_ids = self
            .eggs
            .values()
            .filter(|egg| egg.team_name == team_name)
            .map(|egg| egg.id)
            .collect::<Vec<_>>();

        egg_ids.choose(&mut rand::thread_rng()).copied()
    }

    fn create_player_from_egg(&mut self, team_name: &str, egg: &Egg) -> Player {
        let player_id = self.allocate_player_id();
        let next_food_tick = Instant::now() + self.duration(FOOD_LIFETIME_TIME_UNITS);

        Player::new(
            player_id,
            team_name.to_string(),
            egg.x,
            egg.y,
            Orientation::random(),
            next_food_tick,
        )
    }

    fn allocate_player_id(&mut self) -> usize {
        let player_id = self.next_player_id;

        self.next_player_id += ID_INCREMENT;
        player_id
    }

    fn complete_ai_authentication(
        &mut self,
        client_id: ClientId,
        authentication: AiAuthentication,
    ) {
        let player_id = authentication.player.id;
        let team_name = authentication.player.team_name.clone();

        self.configure_ai_client(client_id, player_id, &team_name);
        self.players.insert(player_id, authentication.player);

        self.send_ai_connection_response(client_id, &team_name);
        self.broadcast_ai_connection(player_id, authentication.egg_id);
    }

    fn configure_ai_client(&mut self, client_id: ClientId, player_id: usize, team_name: &str) {
        let Some(client) = self.clients.get_mut(&client_id) else {
            return;
        };

        client.state = ClientState::Ai;
        client.team_name = Some(team_name.to_string());
        client.player_id = Some(player_id);
    }

    fn send_ai_connection_response(&mut self, client_id: ClientId, team_name: &str) {
        let available_slots = self.available_egg_count(team_name);
        let response = format!(
            "{available_slots}\n{} {}\n",
            self.map.width, self.map.height,
        );

        self.queue_to_client(client_id, &response);
    }

    fn broadcast_ai_connection(&mut self, player_id: usize, egg_id: usize) {
        let Some(player) = self.players.get(&player_id) else {
            return;
        };

        let messages = format!(
            "{}{}{}",
            gui::pnw(player),
            gui::pin(player),
            gui::ebo(egg_id),
        );

        self.broadcast_to_guis(&messages);
    }

    fn reject_authentication(&mut self, client_id: ClientId) {
        self.queue_to_client(client_id, KO_RESPONSE);
    }
}
