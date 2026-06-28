from agent import Agent
from environment import ZappyEnv
from network import MLP
from plotting import plot_stats, load_stats
from constants import ACTIONS, TILE_SPAWN_DENSITY

import argparse
import random
import yaml
import numpy as np
import json
import os
import csv
# from multiprocessing import Pool

def next_generation(population, scores, elite_frac=0.2, mutation_std=0.02, reinject_frac=0.1):
    best_score = max(scores)
    limit = best_score - 1000

    ranked_pairs = sorted(
        zip(scores, population),
        key=lambda x: x[0],
        reverse=True
    )
    ranked = [g for _, g in ranked_pairs]
    ranked_scores = [s for s, _ in ranked_pairs]

    nb_best = sum(
        1 for score, _ in ranked_pairs
        if score >= limit
    )
    max_elites = int(len(population) * elite_frac)
    n_elite = min(max_elites, nb_best)

    elites = ranked[:n_elite]
    elite_scores = ranked_scores[:n_elite]

    score_range = max(elite_scores) - min(elite_scores)
    collapsed   = (n_elite == max_elites) and (score_range < 1.0)

    n_reinject = int(len(population) * reinject_frac) if collapsed else 0
    n_children = len(population) - n_elite - n_reinject

    children = [
        elites[np.random.randint(len(elites))].fork(mutation_std)
        for _ in range(n_children)
    ]
    randoms = [
        MLP.random(population[0].layer_dims, mutation_std)
        for _ in range(n_reinject)
    ]
    return elites + children + randoms, n_elite #, collapsed (maybe for log later)

def run_episode_two_teams(genomes_a, genomes_b, env, cfg, prev_scores_a, prev_scores_b):
    indices_a = random.sample(
        range(len(genomes_a)),
        min(cfg["TEAM_EPISODE_SIZE"], len(genomes_a))
    )
    indices_b = random.sample(
        range(len(genomes_b)),
        min(cfg["TEAM_EPISODE_SIZE"], len(genomes_b))
    )

    agents_a = [
        Agent(brain=genomes_a[i],
              map_size=env.map_size)
        for i in indices_a
    ]
    agents_b = [
        Agent(brain=genomes_b[i],
              map_size=env.map_size)
        for i in indices_b
    ]

    winner = env.run(agents_a, agents_b)

    highest_lvl_a = max(a.level for a in agents_a)
    highest_lvl_b = max(a.level for a in agents_b)

    full_scores_a = list(prev_scores_a)
    full_scores_b = list(prev_scores_b)
    for idx, agent in zip(indices_a, agents_a):
        full_scores_a[idx] = agent.metrics.fitness(agents_a,
                            env.map[agent.coordinates[0] + agent.coordinates[1] * env.map_size[0]],
                            agent)
    for idx, agent in zip(indices_b, agents_b):
        full_scores_b[idx] = agent.metrics.fitness(agents_b,
                            env.map[agent.coordinates[0] + agent.coordinates[1] * env.map_size[0]],
                            agent)

    return (full_scores_a, full_scores_b,
            highest_lvl_a, highest_lvl_b,
            winner, len(agents_a), len(agents_b))

def train(cfg, pop_a, pop_b, log_path):
    consecutive_wins_a = 0
    consecutive_wins_b = 0
    win_threshold = cfg.get("WIN_STREAK_STOP", 50)

    with open(log_path, "w", newline="") as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=[
            "gen", "best_a", "best_b", "mean_a", "mean_b",
            "max_level_a", "max_level_b",
            "team_a_size", "team_b_size", "n_elite_a", "n_elite_b",
            "nb_fork", "nb_incantation",
            "nb_broadcast", "nb_neg_broadcast", "nb_pos_broadcast",
            "useful_drops", "useless_drops",
            "objective_prog", "objective_failed",
            "teammates_reach", "winner"
        ])
        writer.writeheader()

        scores_a = [0.0] * (cfg["POP_SIZE"] // cfg["N_TEAMS"])
        scores_b = [0.0] * (cfg["POP_SIZE"] // cfg["N_TEAMS"])

        for gen in range(cfg["N_GENERATIONS"]):
            env = ZappyEnv(cfg["MAP_SIZE"], cfg["MAX_EPISODE_TICKS"], cfg["MAX_TEAM_SIZE"], cfg["FORK_MUTATION_STD"])
            (scores_a, scores_b,
            max_lvl_a, max_lvl_b,
            winner,
            team_a_size, team_b_size) = run_episode_two_teams(pop_a, pop_b, env, cfg, scores_a, scores_b)

            # early stopping
            if winner == 0:
                consecutive_wins_a += 1
                consecutive_wins_b  = 0
            elif winner == 1:
                consecutive_wins_b += 1
                consecutive_wins_a  = 0
            else:
                consecutive_wins_a = 0
                consecutive_wins_b = 0

            if consecutive_wins_a >= win_threshold:
                break
            if consecutive_wins_b >= win_threshold:
                break

            pop_a, n_elite_a = next_generation(pop_a, scores_a, cfg["ELITE_FRAC"], cfg["MUTATION_STD"], cfg["REINJECT_FRAC"])
            pop_b, n_elite_b = next_generation(pop_b, scores_b, cfg["ELITE_FRAC"], cfg["MUTATION_STD"], cfg["REINJECT_FRAC"])

            writer.writerow({
                "gen":              gen,
                "best_a":           max(scores_a),
                "best_b":           max(scores_b),
                "mean_a":           round(np.mean(scores_a), 2),
                "mean_b":           round(np.mean(scores_b), 2),
                "max_level_a":      max_lvl_a,
                "max_level_b":      max_lvl_b,
                "team_a_size":      team_a_size,
                "team_b_size":      team_b_size,
                "n_elite_a":        n_elite_a,
                "n_elite_b":        n_elite_b,
                "nb_fork":          env.metrics.nb_fork,
                "nb_incantation":   env.metrics.nb_incantation,
                "nb_broadcast":     env.metrics.nb_broadcast,
                "nb_neg_broadcast": env.metrics.nb_neg_broadcast,
                "nb_pos_broadcast": env.metrics.nb_pos_broadcast,
                "useful_drops":     env.metrics.useful_drops,
                "useless_drops":    env.metrics.useless_drops,
                "objective_prog":   env.metrics.objective_prog,
                "objective_failed": env.metrics.objective_failed,
                "teammates_reach":  env.metrics.teammates_reach,
                "winner":           winner,
            })
            csvfile.flush()

            if gen % cfg.get("CHECKPOINT_INTERVAL", 20) == 0:
                os.makedirs("checkpoints", exist_ok=True)
                np.save(f"checkpoints/gen{gen}_a.npy", pop_a[0])
                np.save(f"checkpoints/gen{gen}_b.npy", pop_b[0])

    return pop_a, pop_b, scores_a, scores_b

def compute_state_dim():
    n_resources = len(TILE_SPAWN_DENSITY)       # 7
    max_vision_tiles = (7 + 1) ** 2             # 64, level 7 max

    pos       = 2                               # x, y normalized
    direction = 4                               # one-hot
    level     = 1
    survival  = 1
    inventory = n_resources                     # 7
    broadcast = 20                              # 1 bool + 9 direction one-hot (0-8) + missing spec
    vision    = max_vision_tiles * (n_resources + 2)  # 64 * 7 = 448

    total = pos + direction + level + survival + inventory + broadcast + vision
    return total

def parse_yaml():
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=str, default="config.yaml")
    args = parser.parse_args()

    with open(args.config, "r") as f:
        return yaml.safe_load(f)

if __name__ == "__main__":
    cfg_params = parse_yaml()

    if cfg_params["POP_SIZE"] % cfg_params["N_TEAMS"] != 0:
        raise ValueError("POP_SIZE must be divisible by N_TEAMS")

    directory_name = f"seed_{cfg_params['SEED']}"
    os.makedirs(directory_name, exist_ok=True)
    
    cfg_params["STATE_DIM"]  = compute_state_dim()
    cfg_params["LAYER_DIMS"] = [cfg_params["STATE_DIM"]] + cfg_params["HIDDEN_LAYERS"] + [len(ACTIONS)]

    pop_size = cfg_params["POP_SIZE"] // cfg_params["N_TEAMS"]
    pop_a = [MLP.random(cfg_params["LAYER_DIMS"], cfg_params["MUTATION_STD"]) for _ in range(pop_size)]
    pop_b = [MLP.random(cfg_params["LAYER_DIMS"], cfg_params["MUTATION_STD"]) for _ in range(pop_size)]

    log_path = f"{directory_name}/training.csv"
    pop_a, pop_b, scores_a, scores_b = train(cfg_params, pop_a, pop_b, log_path)

    plot_stats(load_stats(log_path), directory_name)

    is_best_score_a = max(scores_a) >= max(scores_b)

    best_brain = pop_a[0] if is_best_score_a else pop_b[0]
    best_team = "a" if is_best_score_a else "b"
    np.save(f"{directory_name}/best_genome.npy", best_brain.genome)

    with open(f"{directory_name}/arch.json", "w") as f:
        json.dump({
            "layer_dims": cfg_params["LAYER_DIMS"],
            "state_dim":  cfg_params["STATE_DIM"],
            "n_actions":  len(ACTIONS),
            "team":       best_team,
        }, f, indent=2)
