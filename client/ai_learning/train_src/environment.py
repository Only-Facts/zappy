from constants import (RESOURCE_KEYS, TILE_SPAWN_DENSITY, ACTIONS, EVOLUTION_TABLE,
                       STONE_KEYS, MAX_VISION_TILES, DIRECTION_VECTORS)
from agent import Agent
from utils import compute_adv, compare_adv, compute_sound_direction
from broadcast import Broadcast

from copy import copy
import numpy as np

class ZappyEnvMetrics:
    def __init__(self):
        self.nb_fork = 0
        self.nb_incantation = 0
        self.nb_broadcast = 0
        self.nb_pos_broadcast = 0
        self.nb_neg_broadcast = 0
        self.useful_drops = 0
        self.useless_drops = 0
        self.objective_prog = 0
        self.objective_failed = 0
        self.teammates_reach = 0

class ZappyEnv:
    def __init__(self, map_size, episode_ticks, max_team_size, fork_mutation_std):
        self.tile_number = map_size[0] * map_size[1]
        self.map = [
            {resource: 0 for resource in RESOURCE_KEYS}
            for _ in range(map_size[0] * map_size[1])
        ]
        self.map_size = map_size
        self.episode_ticks = episode_ticks
        self.max_team_size = max_team_size
        self.fork_mutation_std = fork_mutation_std

        self.metrics = ZappyEnvMetrics()

        self._action_handlers = {
            ACTIONS["INCANTATE"]: self._incantate,
            ACTIONS["PUSH"]: self._push,
            ACTIONS["MOVE_FORWARD"]: self._move_forward,
            ACTIONS["TURN_LEFT"]: self._turn,
            ACTIONS["TURN_RIGHT"]: self._turn,
            ACTIONS["EAT"]: self._eat,
            ACTIONS["FORK"]: self._fork,
            ACTIONS["BROADCAST_ADV"]: self._broadcast,
            ACTIONS["BROADCAST_INV"]: self._broadcast
        }
        for resource in RESOURCE_KEYS:
            self._action_handlers[ACTIONS[f"TAKE_{resource.upper()}"]] = self._take_resource
            self._action_handlers[ACTIONS[f"DROP_{resource.upper()}"]] = self._drop_resource

    def update(self):
        for resource, density in TILE_SPAWN_DENSITY.items():
            target = max(int(self.tile_number * density), 1)

            current = sum(tile[resource] for tile in self.map)

            missing = target - current

            for _ in range(max(0, missing)):
                idx = np.random.randint(0, self.tile_number)
                self.map[idx][resource] += 1

    def compute_valid_mask(self, agent, team_agents, other_team):
        mask = np.ones(len(ACTIONS), dtype=bool)

        tile = self.map[agent.coordinates[0] + agent.coordinates[1] * self.map_size[0]]

        needed = EVOLUTION_TABLE[agent.level]
        agents_on_tile = [a for a in (team_agents + other_team)
                if a.coordinates == agent.coordinates
                and a.level == agent.level
                and a is not agent
                and a.survival_ticks > 0]
        stones_ok = all(tile[r] >= req
                for r, req in zip(STONE_KEYS, needed[1:]))
        mask[ACTIONS["INCANTATE"]] = (len(agents_on_tile) + 1 >= needed[0]) and stones_ok

        ennemies_on_tile = [a for a in other_team
                if a.coordinates == agent.coordinates]
        mask[ACTIONS["PUSH"]] = len(ennemies_on_tile) >= 1

        for i, resource in enumerate(RESOURCE_KEYS):
            mask[5 + i] = tile[resource] > 0

        for i, resource in enumerate(RESOURCE_KEYS):
            mask[12 + i] = agent.inventory[resource] > 0

        mask[ACTIONS["EAT"]] = agent.inventory["food"] > 0

        mask[ACTIONS["FORK"]] = len(team_agents) < self.max_team_size

        return mask

    def run(self, agents_a, agents_b):
        teams = [
            (agents_a, agents_b, 0),
            (agents_b, agents_a, 1)
        ]

        for tick in range(self.episode_ticks):
            if tick % 20 == 0:
                self.update()
            dead_agent = 0

            for team, other_team, winner in teams:
                for agent in team:
                    if not agent.update(tick):
                        dead_agent += 1
                        continue

                    if agent.metrics.current_objective is not None:
                        agent.metrics.objective_age += 1
                        if agent.metrics.objective_age > agent.metrics.max_age:
                            agent.metrics.current_objective = None
                            agent.metrics.objective_failed += 1
                            self.metrics.objective_failed += 1

                    state = agent.get_state(self, team, other_team)
                    valid_mask = self.compute_valid_mask(agent, team, other_team)
                    action = agent.act(state, valid_mask)
                    self.apply_action(agent, action, team, other_team)

                    if agent.level == 8:
                        return winner
            if dead_agent == len(agents_a) + len(agents_b):
                return -1
        return -1

    def apply_action(self, agent, action, team_agents, other_team):
        """
        action 0 is incantate / elevate
        action 1 is push
        action 2 is move forward
        action 3 to 4 is turn 0 left, 1 right
        action 5 to 11 is get a ressoure (in order of TILE_SPAWN_DENSITY keys)
        action 12 to 18 is drop a ressoure (in order of TILE_SPAWN_DENSITY keys)
        action 19 is eating
        action 20 is giving birth
        action 21 is broadcasting
        """
        tile = self.map[agent.coordinates[0] + agent.coordinates[1] * self.map_size[0]]

        self._action_handlers[action](agent, team_agents, other_team, tile, action)          

    def _incantate(self, agent, team_agents, other_team, tile, action):
        needed = EVOLUTION_TABLE[agent.level]

        agents_on_tile = [a for a in (team_agents + other_team)
                        if a.coordinates == agent.coordinates
                        and a.level == agent.level
                        and a.survival_ticks > 0]

        for resource, req in zip(STONE_KEYS, needed[1:]):
            tile[resource] -= req
        for a in agents_on_tile:
            a.level += 1
        agent.metrics.incantation += 1
        self.metrics.nb_incantation += 1

    def _push(self, agent, team_agents, other_team, tile, action):
        for a in other_team:
            if a.coordinates == agent.coordinates:
                a.move(-1, agent.direction)

    def _move_forward(self, agent, team_agents, other_team, tile, action):
        if agent.move(1):
            self.metrics.objective_prog += 1
        players_around = [a for a in team_agents
            if a.coordinates == agent.coordinates
            and a.level == agent.level
            and a.survival_ticks > 0]
        if players_around:
            self.metrics.teammates_reach += 1
            agent.metrics.current_objective = None
            agent.metrics.objective_progress += 2

    def _turn(self, agent, team_agents, other_team, tile, action):
        if agent.turn(action - 3):
            self.metrics.objective_prog += 1

    def _take_resource(self, agent, team_agents, other_team, tile, action):
        resource = list(RESOURCE_KEYS)[action - 5]
        agent.get_resource(resource)
        tile[resource] -= 1

    def _drop_resource(self, agent, team_agents, other_team, tile, action):
        resource = list(RESOURCE_KEYS)[action - 12]
        if agent.drop_resource(resource, tile):
            self.metrics.useful_drops += 1
        else:
            self.metrics.useless_drops += 1

    def _eat(self, agent, team_agents, other_team, tile, action):
        agent.eat()
    
    def _fork(self, agent, team_agents, other_team, tile, action):
        team_agents.append(Agent(brain=agent.brain.fork(self.fork_mutation_std),
                                 map_size=self.map_size))
        team_agents[-1].move_to(agent.coordinates)
        self.metrics.nb_fork += 1

    def _broadcast(self, agent, team_agents, other_team, tile, action):
        self.metrics.nb_broadcast += 1
        players_needed, missing_resources = compute_adv(agent, (team_agents + other_team), tile)
        useless = players_needed + sum(missing_resources) == 0

        if action == 21:
            BROADCAST_TYPE = "adv"
            if useless:
                agent.metrics.neg_broadcast += 2
                self.metrics.nb_neg_broadcast += 1
            if agent.received_broadcasts:
                last_msg = agent.received_broadcasts[-1]
                if last_msg.msg_type == "adv" and last_msg.level == agent.level:
                    if compare_adv(last_msg.players_needed, last_msg.missing_stones,
                                    players_needed, missing_resources,
                                    agent.level):
                        agent.metrics.neg_broadcast += 1
                        self.metrics.nb_neg_broadcast += 1
        else: # action equal to 22 there is no action after
            BROADCAST_TYPE = "inv"
            if agent.received_broadcasts:
                last_msg = agent.received_broadcasts[-1]
                if last_msg.msg_type == "adv" and last_msg.level == agent.level:
                    agent.metrics.pos_broadcast += 1
                    self.metrics.nb_pos_broadcast += 1
                else:
                    agent.metrics.neg_broadcast += 1
                    self.metrics.nb_neg_broadcast += 1
            else:
                agent.metrics.neg_broadcast += 1
                self.metrics.nb_neg_broadcast += 1

        msg = Broadcast.create_msg(agent, players_needed, missing_resources, BROADCAST_TYPE)

        for a in (team_agents + other_team):
            if a is agent:
                continue

            new_msg = copy(msg)
            new_msg.direction = compute_sound_direction(agent, a, self.map_size)

            a.received_broadcasts.append(new_msg)

    def get_visible_tiles(self, agent, team_agents, other_team):
        result = np.zeros(MAX_VISION_TILES * (len(RESOURCE_KEYS) + 2), dtype=np.float32)

        fx, fy = DIRECTION_VECTORS[agent.direction]
        rx, ry = -fy, fx

        ally_positions  = [a.coordinates for a in team_agents if a is not agent and a.dying_tick < 0]
        enemy_positions = [a.coordinates for a in other_team if a.dying_tick < 0]

        idx = 0

        for row in range(agent.level + 1):
            for col in range(-row, row + 1):
                nx = (agent.coordinates[0] + fx * row + rx * col) % agent.map_size[0]
                ny = (agent.coordinates[1] + fy * row + ry * col) % agent.map_size[1]
                tile = self.map[nx + ny * self.map_size[0]]
                for r in RESOURCE_KEYS:
                    result[idx] = min(tile[r], 10) / 10.0
                    idx += 1
                result[idx]     = min(ally_positions.count((nx, ny)), 5) / 5.0
                result[idx + 1] = min(enemy_positions.count((nx, ny)), 5) / 5.0
                idx += 2
        return result
