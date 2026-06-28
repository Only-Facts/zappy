from constants import (RESOURCE_KEYS, DIRECTION_VECTORS, MAX_SURVIVAL,
                       STONE_KEYS, EVOLUTION_TABLE, LEFT_ROT, RIGHT_ROT)
from utils import compute_adv

import numpy as np

class AgentMetrics:
    def __init__(self):
        self.useful_drop = 0
        self.useful_drop_weak = 0
        self.useless_drop = 0
        self.incantation = 0
        self.pos_broadcast = 0
        self.neg_broadcast = 0

        self.current_objective = None
        self.max_age = 0
        self.objective_age = 0
        self.objective_progress = 0
        self.objective_failed = 0
        self.priority = float("inf")
    
    def fitness(self, team, tile, agent):
        needed = EVOLUTION_TABLE[agent.level]
        valid_allies = [
            a for a in team
            if a.coordinates == agent.coordinates
            and a.level == agent.level
        ]
        ally_progress = min(len(valid_allies) / needed[0], 1.0)

        stone_progress = sum(
            min(tile[r] / req, 1.0)
            for r, req in zip(STONE_KEYS, needed[1:])
            if req > 0
        )
        stone_progress /= sum(
            1 for req in needed[1:] if req > 0
        )

        return (
            (agent.level**2) * 1000
            + ally_progress * 300
            + stone_progress * 300
            + self.useful_drop * 50
            + min(self.useful_drop_weak, 20) * 20
            - self.useless_drop * 20
            + self.incantation * 300
            + self.pos_broadcast * 20
            - self.neg_broadcast * 30
            + self.objective_progress * 0.5
            - self.objective_failed * 20
            + agent.survival_ticks * 0.1
        )

    @staticmethod
    def broadcast_priority(players_needed, missing_stones):
        return players_needed * 50 + sum(missing_stones) # Maybe even try with 100 later

    def compute_max_age(self, level):
        return min(150, 30 + self.priority * 5 + level * 20)

class StateEncoder:
    @staticmethod
    def encode_broadcast(env, agent, teams):
        best = None
        tile = env.map[agent.coordinates[0] + agent.coordinates[1] * env.map_size[0]]
        players_needed, missing_stones = compute_adv(agent, (teams), tile)
        best_priority = AgentMetrics.broadcast_priority(players_needed, missing_stones)
        inv_received = []
        for msg in agent.received_broadcasts:
            if msg.msg_type != "adv":
                inv_received.append(msg)
                continue
            if msg.level != agent.level:
                continue

            priority = AgentMetrics.broadcast_priority(msg.players_needed, msg.missing_stones)
            if priority == 0:
                continue
            if priority < best_priority:
                best = msg
                best_priority = priority

        broadcast = np.zeros(20)
        if best is not None:
            agent.metrics.current_objective = best.direction
            agent.metrics.priority = best_priority
            agent.metrics.objective_age = 0
            agent.metrics.max_age = agent.metrics.compute_max_age(agent.level)
            broadcast = best.normalize(broadcast)
        else:
            best_rest = float("inf")
            best_inv = None
            for msg in inv_received:
                rest = 0
                for i in range(len(STONE_KEYS)):
                    rest += max(0, missing_stones[i] - msg.missing_stones[i])
                if rest < best_rest:
                    best_rest = rest
                    best_inv = msg
            if best_inv is not None:
                broadcast = best_inv.normalize(broadcast)

        agent.received_broadcasts.clear()
        # Broadcasts are single-use observations and cleared once encoded.
        return broadcast

    @staticmethod
    def encode(agent, env, team, enemies):
        pos = np.array([
            agent.coordinates[0] / agent.map_size[0],
            agent.coordinates[1] / agent.map_size[1],
        ])

        direction = np.zeros(4)
        direction[agent.direction] = 1.0

        level = np.array([agent.level / 8.0])

        survival = np.array([agent.survival_ticks / MAX_SURVIVAL])

        inventory = np.array([
            min(agent.inventory[r], 20) / 20.0 for r in RESOURCE_KEYS
        ])

        broadcast = StateEncoder.encode_broadcast(env, agent, team + enemies)

        vision = env.get_visible_tiles(agent, team, enemies)

        return np.concatenate([pos, direction, level, survival, inventory, broadcast, vision])

class Agent:
    def __init__(self, brain, map_size):
        self.brain = brain
        self.map_size = map_size
        self.coordinates = (np.random.randint(map_size[0]), np.random.randint(map_size[1]))
        self.direction = np.random.randint(4)
        self.survival_ticks = MAX_SURVIVAL
        self.level = 1
        self.inventory = {resource: 0 for resource in RESOURCE_KEYS}
        self.dying_tick = -1
        self.received_broadcasts = []

        self.metrics = AgentMetrics()

    def act(self, state, valid_mask):
        logits = self.brain.forward(state)
        logits = np.where(valid_mask, logits, -np.inf)
        return np.argmax(logits)

    def get_state(self, env, team_agents, other_team):
        return StateEncoder.encode(self, env, team_agents, other_team)

    def reward_objective_progress(self, factor):
        if self.metrics.current_objective == 1:
            self.metrics.objective_progress += (1 / factor)
            return True
        elif self.metrics.current_objective in (2, 8):
            self.metrics.objective_progress += (0.5 / factor)
            return True
        return False

    def move(self, move, facing=None):
        dx, dy = DIRECTION_VECTORS[facing if facing is not None else self.direction]
        if move == -1:
            dx, dy = -dx, -dy
        
        self.coordinates = (
            (self.coordinates[0] + dx) % self.map_size[0],
            (self.coordinates[1] + dy) % self.map_size[1],
        )

        if move == 1 and self.metrics.current_objective:
            return self.reward_objective_progress(1)
        return False

    def move_to(self, coordinates):
        self.coordinates = coordinates

    def turn(self, direction):
        self.direction = (self.direction + (1 if direction == 1 else -1)) % 4

        if (self.metrics.current_objective):
            if direction == 0:
                self.metrics.current_objective = LEFT_ROT[self.metrics.current_objective]
            else:
                self.metrics.current_objective = RIGHT_ROT[self.metrics.current_objective]
            
            return self.reward_objective_progress(2)
        return False

    def get_resource(self, resource):
        self.inventory[resource] += 1

    def drop_resource(self, resource, tile):
        self.inventory[resource] -= 1
        tile[resource] += 1
        if resource != "food":
            idx = STONE_KEYS.index(resource)
            needed_rock = sum(
                EVOLUTION_TABLE[level][1 + idx]
                for level in range(self.level + 1, 8)
            )
            if tile[resource] < EVOLUTION_TABLE[self.level][1 + idx]:
                self.metrics.useful_drop += 1
                return True
            elif tile[resource] < needed_rock:
                self.metrics.useful_drop_weak += 1
                return True
        self.metrics.useless_drop += 1
        return False

    def eat(self):
        self.inventory["food"] -= 1
        self.survival_ticks = MAX_SURVIVAL

    def update(self, tick):
        self.survival_ticks -= 1
        if self.dying_tick >= 0:
            return False
        if self.survival_ticks <= 0:
            self.dying_tick = tick
            return False
        return True
