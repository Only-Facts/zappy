from constants import BROADCAST_TABLE, EVOLUTION_TABLE, STONE_KEYS

from math import degrees, atan2
from numpy import random

def compute_sound_direction(sender, receiver, map_size):
    if sender.coordinates == receiver.coordinates:
        return 0

    w, h = map_size

    dx = sender.coordinates[0] - receiver.coordinates[0]
    dy = sender.coordinates[1] - receiver.coordinates[1]

    if dx > w / 2:
        dx -= w
    elif dx < -w / 2:
        dx += w

    if dy > h / 2:
        dy -= h
    elif dy < -h / 2:
        dy += h

    angle = degrees(atan2(-dy, dx))
    angle = (angle + 360) % 360

    sector = round(angle / 45) % 8

    return BROADCAST_TABLE[receiver.direction][sector]

def compute_adv(agent, agents, tile):
    needed = EVOLUTION_TABLE[agent.level]

    players_around = [a for a in agents
        if a.coordinates == agent.coordinates
        and a.level == agent.level
        and a.survival_ticks > 0]

    missing_resources = [
        max(0, needed[i + 1] - tile[key])
        for i, key in enumerate(STONE_KEYS)
    ]
    return max(0, needed[0] - len(players_around)), missing_resources

def compare_adv(missing_players_1, missing_stone_1, missing_players_2, missing_stone_2, level):
    needed = EVOLUTION_TABLE[level]
    max_players = needed[0]
    max_stones  = sum(needed[1:])

    score_1 = missing_players_1 / max(max_players, 1) + sum(missing_stone_1) / max(max_stones, 1)
    score_2 = missing_players_2 / max(max_players, 1) + sum(missing_stone_2) / max(max_stones, 1)
    return score_1 >= score_2
