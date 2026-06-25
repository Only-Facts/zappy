use super::incantation::Incantation;
use super::timing::duration_for;
use crate::config::Config;
use crate::constants::{
    FIRST_CLIENT_ID, FIRST_EGG_ID, FIRST_INCANTATION_ID, FIRST_PLAYER_ID, ID_INCREMENT,
    RESOURCE_RESPAWN_TIME_UNITS,
};
use crate::entities::egg::Egg;
use crate::entities::player::Player;
use crate::entities::team::Team;
use crate::server::client::{Client, ClientId, ClientState};
use crate::world::map::GameMap;
use crate::world::resources;
use rand::Rng;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct GameState {
    pub clients: HashMap<ClientId, Client>,
    pub players: HashMap<usize, Player>,
    pub eggs: HashMap<usize, Egg>,
    pub teams: Vec<Team>,
    pub map: GameMap,
    pub frequency: usize,
    pub next_client_id: usize,
    pub next_player_id: usize,
    pub next_egg_id: usize,
    pub next_incantation_id: usize,
    pub next_resource_spawn: Instant,
    pub incantations: HashMap<usize, Incantation>,
    pub winner: Option<String>,
}

impl GameState {
    pub fn new(config: &Config) -> Self {
        let now = Instant::now();
        let mut map = GameMap::new(config.width, config.height);

        resources::spawn_initial_resources(&mut map);

        let teams = config
            .teams
            .iter()
            .rev()
            .cloned()
            .map(Team::new)
            .collect::<Vec<_>>();

        let mut eggs = HashMap::new();
        let mut next_egg_id = FIRST_EGG_ID;
        let mut random = rand::thread_rng();

        for team in &teams {
            for _ in 0..config.clients_nb {
                let egg = Egg::new(
                    next_egg_id,
                    None,
                    team.name.clone(),
                    random.gen_range(0..config.width),
                    random.gen_range(0..config.height),
                );

                eggs.insert(egg.id, egg);
                next_egg_id += ID_INCREMENT;
            }
        }

        Self {
            clients: HashMap::new(),
            players: HashMap::new(),
            eggs,
            teams,
            map,
            frequency: config.freq,
            next_client_id: FIRST_CLIENT_ID,
            next_player_id: FIRST_PLAYER_ID,
            next_egg_id,
            next_incantation_id: FIRST_INCANTATION_ID,
            next_resource_spawn: now + duration_for(RESOURCE_RESPAWN_TIME_UNITS, config.freq),
            incantations: HashMap::new(),
            winner: None,
        }
    }

    pub fn allocate_client_id(&mut self) -> ClientId {
        let client_id = ClientId(self.next_client_id);

        self.next_client_id += ID_INCREMENT;
        client_id
    }

    pub fn queue_to_client(&mut self, client_id: ClientId, text: &str) {
        if let Some(client) = self.clients.get_mut(&client_id) {
            client.queue_text(text);
        }
    }

    pub fn broadcast_to_guis(&mut self, text: &str) {
        for client in self.clients.values_mut() {
            if client.state == ClientState::Gui {
                client.queue_text(text);
            }
        }
    }

    pub fn player_token(&self, player_id: usize) -> Option<ClientId> {
        self.clients.iter().find_map(|(client_id, client)| {
            (client.player_id == Some(player_id)).then_some(*client_id)
        })
    }

    pub fn is_player_frozen(&self, player_id: usize) -> bool {
        self.players
            .get(&player_id)
            .is_some_and(|player| player.frozen_by.is_some())
    }

    pub fn duration(&self, time_units: u64) -> Duration {
        duration_for(time_units, self.frequency)
    }
}
