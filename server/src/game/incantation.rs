use crate::command::{ActiveCommand, Command};
use crate::constants::{
    ELEVATION_UNDERWAY_RESPONSE, ID_INCREMENT, INCANTATION_TIME_UNITS, KO_RESPONSE,
};
use crate::game::GameState;
use crate::gui;
use crate::server::client::ClientId;
use crate::world::map::Resource;
use std::time::Instant;

#[derive(Debug)]
pub struct Incantation {
    id: usize,
    initiator_id: usize,
    participant_ids: Vec<usize>,
    x: usize,
    y: usize,
    level: usize,
}

#[derive(Clone, Copy)]
struct IncantationRequirement {
    players: usize,
    stones: [(Resource, usize); 6],
}

struct IncantationStart {
    x: usize,
    y: usize,
    level: usize,
    participant_ids: Vec<usize>,
}

struct ValidatedIncantation {
    requirement: IncantationRequirement,
    participant_ids: Vec<usize>,
}

impl GameState {
    pub fn begin_incantation(
        &mut self,
        client_id: ClientId,
        player_id: usize,
        now: Instant,
    ) -> bool {
        let Some(start) = self.prepare_incantation(player_id) else {
            self.queue_to_client(client_id, KO_RESPONSE);
            return false;
        };

        self.launch_incantation(client_id, player_id, start, now);
        true
    }

    pub fn finish_incantation(&mut self, incantation_id: usize) {
        let Some(incantation) = self.incantations.remove(&incantation_id) else {
            return;
        };

        match self.validate_incantation(&incantation) {
            Some(validation) => {
                self.complete_incantation(incantation, validation);
            }
            None => {
                self.fail_incantation_record(incantation, None);
            }
        }
    }

    pub fn cancel_incantations_for_player(&mut self, player_id: usize) {
        let incantation_ids = self
            .incantations
            .values()
            .filter(|incantation| incantation.participant_ids.contains(&player_id))
            .map(|incantation| incantation.id)
            .collect::<Vec<_>>();

        for incantation_id in incantation_ids {
            self.cancel_incantation(incantation_id, player_id);
        }
    }

    fn prepare_incantation(&self, initiator_id: usize) -> Option<IncantationStart> {
        let player = self.players.get(&initiator_id)?;
        let position = (player.x, player.y);
        let level = player.level;
        let requirement = incantation_requirement(level)?;

        let participant_ids = self.collect_participants(initiator_id, position, level);

        if !self.incantation_conditions_are_met(position, &participant_ids, requirement) {
            return None;
        }

        Some(IncantationStart {
            x: position.0,
            y: position.1,
            level,
            participant_ids,
        })
    }

    fn collect_participants(
        &self,
        initiator_id: usize,
        position: (usize, usize),
        level: usize,
    ) -> Vec<usize> {
        let mut participant_ids = self
            .players
            .values()
            .filter(|player| {
                player.x == position.0
                    && player.y == position.1
                    && player.level == level
                    && player.frozen_by.is_none()
            })
            .map(|player| player.id)
            .collect::<Vec<_>>();

        participant_ids.sort_unstable();
        move_initiator_to_front(&mut participant_ids, initiator_id);

        participant_ids
    }

    fn incantation_conditions_are_met(
        &self,
        position: (usize, usize),
        participant_ids: &[usize],
        requirement: IncantationRequirement,
    ) -> bool {
        participant_ids.len() >= requirement.players
            && self.tile_has_requirement(position.0, position.1, requirement)
    }

    fn launch_incantation(
        &mut self,
        client_id: ClientId,
        initiator_id: usize,
        start: IncantationStart,
        now: Instant,
    ) {
        let incantation_id = self.allocate_incantation_id();
        let finishes_at = now + self.duration(INCANTATION_TIME_UNITS);

        self.freeze_participants(incantation_id, &start.participant_ids);
        self.broadcast_incantation_start(&start);
        self.store_incantation(incantation_id, initiator_id, start);
        self.schedule_incantation(client_id, incantation_id, finishes_at);
    }

    fn allocate_incantation_id(&mut self) -> usize {
        let incantation_id = self.next_incantation_id;

        self.next_incantation_id += ID_INCREMENT;
        incantation_id
    }

    fn freeze_participants(&mut self, incantation_id: usize, participant_ids: &[usize]) {
        for participant_id in participant_ids {
            if let Some(player) = self.players.get_mut(participant_id) {
                player.frozen_by = Some(incantation_id);
            }

            self.queue_to_player(*participant_id, ELEVATION_UNDERWAY_RESPONSE);
        }
    }

    fn broadcast_incantation_start(&mut self, start: &IncantationStart) {
        let message = gui::pic(start.x, start.y, start.level, &start.participant_ids);

        self.broadcast_to_guis(&message);
    }

    fn store_incantation(
        &mut self,
        incantation_id: usize,
        initiator_id: usize,
        start: IncantationStart,
    ) {
        self.incantations.insert(
            incantation_id,
            Incantation {
                id: incantation_id,
                initiator_id,
                participant_ids: start.participant_ids,
                x: start.x,
                y: start.y,
                level: start.level,
            },
        );
    }

    fn schedule_incantation(
        &mut self,
        client_id: ClientId,
        incantation_id: usize,
        finishes_at: Instant,
    ) {
        let Some(client) = self.clients.get_mut(&client_id) else {
            return;
        };

        let mut active = ActiveCommand::new(Command::Incantation, finishes_at);

        active.incantation_id = Some(incantation_id);
        client.active = Some(active);
    }

    fn validate_incantation(&self, incantation: &Incantation) -> Option<ValidatedIncantation> {
        let requirement = incantation_requirement(incantation.level)?;

        let participant_ids = self.collect_valid_participants(incantation);

        if !self.incantation_conditions_are_met(
            (incantation.x, incantation.y),
            &participant_ids,
            requirement,
        ) {
            return None;
        }

        Some(ValidatedIncantation {
            requirement,
            participant_ids,
        })
    }

    fn collect_valid_participants(&self, incantation: &Incantation) -> Vec<usize> {
        incantation
            .participant_ids
            .iter()
            .filter(|player_id| self.participant_is_still_valid(incantation, **player_id))
            .copied()
            .collect()
    }

    fn participant_is_still_valid(&self, incantation: &Incantation, player_id: usize) -> bool {
        self.players.get(&player_id).is_some_and(|player| {
            player.x == incantation.x
                && player.y == incantation.y
                && player.level == incantation.level
                && player.frozen_by == Some(incantation.id)
        })
    }

    fn complete_incantation(&mut self, incantation: Incantation, validation: ValidatedIncantation) {
        self.consume_incantation_stones(incantation.x, incantation.y, validation.requirement);
        self.release_incantation_participants(&incantation, &validation.participant_ids);
        self.elevate_participants(&validation.participant_ids);
        self.broadcast_incantation_success(&incantation);
        self.check_victory();
    }

    fn consume_incantation_stones(
        &mut self,
        x: usize,
        y: usize,
        requirement: IncantationRequirement,
    ) {
        let Some(tile) = self.map.get_tile_mut(x, y) else {
            return;
        };

        for (resource, amount) in requirement.stones {
            let _ = tile.remove_many(resource, amount);
        }
    }

    fn release_incantation_participants(
        &mut self,
        incantation: &Incantation,
        valid_participant_ids: &[usize],
    ) {
        for participant_id in &incantation.participant_ids {
            self.unfreeze_participant(*participant_id, incantation.id);

            if !valid_participant_ids.contains(participant_id) {
                self.queue_to_player(*participant_id, KO_RESPONSE);
            }
        }
    }

    fn unfreeze_participant(&mut self, player_id: usize, incantation_id: usize) {
        let Some(player) = self.players.get_mut(&player_id) else {
            return;
        };

        if player.frozen_by == Some(incantation_id) {
            player.frozen_by = None;
        }
    }

    fn elevate_participants(&mut self, participant_ids: &[usize]) {
        for participant_id in participant_ids {
            self.elevate_player(*participant_id);
        }
    }

    fn elevate_player(&mut self, player_id: usize) {
        let Some(player) = self.players.get_mut(&player_id) else {
            return;
        };

        player.level += 1;

        let new_level = player.level;
        let gui_message = gui::plv(player);
        let response = format!("Current level: {new_level}\n");

        self.queue_to_player(player_id, &response);
        self.broadcast_to_guis(&gui_message);
    }

    fn broadcast_incantation_success(&mut self, incantation: &Incantation) {
        self.broadcast_to_guis(&gui::pie(incantation.x, incantation.y, true));

        let tile_message = gui::bct(&self.map, incantation.x, incantation.y);

        self.broadcast_to_guis(&tile_message);
    }

    fn fail_incantation_record(
        &mut self,
        incantation: Incantation,
        excluded_player_id: Option<usize>,
    ) {
        for participant_id in &incantation.participant_ids {
            self.unfreeze_participant(*participant_id, incantation.id);

            match excluded_player_id {
                Some(excluded) if excluded == *participant_id => {}
                _ => {
                    self.queue_to_player(*participant_id, KO_RESPONSE);
                }
            }
        }

        self.broadcast_to_guis(&gui::pie(incantation.x, incantation.y, false));
    }

    fn cancel_incantation(&mut self, incantation_id: usize, excluded_player_id: usize) {
        let Some(incantation) = self.incantations.remove(&incantation_id) else {
            return;
        };

        self.clear_active_incantation(&incantation);
        self.fail_incantation_record(incantation, Some(excluded_player_id));
    }

    fn clear_active_incantation(&mut self, incantation: &Incantation) {
        let Some(client_id) = self.player_token(incantation.initiator_id) else {
            return;
        };

        let Some(client) = self.clients.get_mut(&client_id) else {
            return;
        };

        let belongs_to_incantation = client
            .active
            .as_ref()
            .is_some_and(|active| active.incantation_id == Some(incantation.id));

        if belongs_to_incantation {
            client.active = None;
        }
    }

    fn queue_to_player(&mut self, player_id: usize, response: &str) {
        let Some(client_id) = self.player_token(player_id) else {
            return;
        };

        self.queue_to_client(client_id, response);
    }

    fn tile_has_requirement(
        &self,
        x: usize,
        y: usize,
        requirement: IncantationRequirement,
    ) -> bool {
        self.map.get_tile(x, y).is_some_and(|tile| {
            requirement
                .stones
                .iter()
                .all(|(resource, amount)| tile.resource_count(*resource) >= *amount)
        })
    }
}

fn move_initiator_to_front(participant_ids: &mut [usize], initiator_id: usize) {
    let position = participant_ids
        .iter()
        .position(|player_id| *player_id == initiator_id);

    if let Some(position) = position {
        participant_ids.swap(0, position);
    }
}

fn incantation_requirement(level: usize) -> Option<IncantationRequirement> {
    let values = match level {
        1 => (1, [1, 0, 0, 0, 0, 0]),
        2 => (2, [1, 1, 1, 0, 0, 0]),
        3 => (2, [2, 0, 1, 0, 2, 0]),
        4 => (4, [1, 1, 2, 0, 1, 0]),
        5 => (4, [1, 2, 1, 3, 0, 0]),
        6 => (6, [1, 2, 3, 0, 1, 0]),
        7 => (6, [2, 2, 2, 2, 2, 1]),
        _ => return None,
    };

    let resources = [
        Resource::Linemate,
        Resource::Deraumere,
        Resource::Sibur,
        Resource::Mendiane,
        Resource::Phiras,
        Resource::Thystame,
    ];

    Some(IncantationRequirement {
        players: values.0,
        stones: std::array::from_fn(|index| (resources[index], values.1[index])),
    })
}
