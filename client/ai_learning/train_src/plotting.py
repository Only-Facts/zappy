import csv
import matplotlib.pyplot as plt
import numpy as np

def load_stats(log_path):
    int_fields = {"gen", "max_level_a", "max_level_b", "winner"}
    stats = {k: [] for k in [
        "gen", "best_a", "best_b", "mean_a", "mean_b",
        "max_level_a", "max_level_b", "winner"
    ]}
    with open(log_path, newline="") as f:
        for row in csv.DictReader(f):
            for k in stats:
                stats[k].append(int(float(row[k])) if k in int_fields else float(row[k]))
    return stats

def plot_stats(stats, directory_name):
    gens     = range(len(stats["best_a"]))
    window   = 20

    fig, axes = plt.subplots(3, 2, figsize=(14, 15))

    # --- row 0: fitness ---
    ax = axes[0, 0]
    ax.plot(gens, stats["best_a"], label="Team A best",  color="royalblue")
    ax.plot(gens, stats["best_b"], label="Team B best",  color="tomato")
    ax.plot(gens, stats["mean_a"], label="Team A mean",  color="royalblue", linestyle="--", alpha=0.5)
    ax.plot(gens, stats["mean_b"], label="Team B mean",  color="tomato",    linestyle="--", alpha=0.5)
    ax.set_title("Fitness over generations")
    ax.set_xlabel("Generation")
    ax.set_ylabel("Fitness")
    ax.legend()
    ax.grid(True, alpha=0.3)

    # --- row 0: win rate ---
    ax = axes[0, 1]
    winners = np.array(stats["winner"])
    win_a = np.convolve(winners == 0,  np.ones(window) / window, mode="valid")
    win_b = np.convolve(winners == 1,  np.ones(window) / window, mode="valid")
    draw  = np.convolve(winners == -1, np.ones(window) / window, mode="valid")
    x     = range(len(win_a))
    ax.plot(x, win_a, label="Team A", color="royalblue")
    ax.plot(x, win_b, label="Team B", color="tomato")
    ax.plot(x, draw,  label="Draw",   color="grey", linestyle=":")
    ax.set_title(f"Win rate (rolling {window}-gen window)")
    ax.set_xlabel("Generation")
    ax.set_ylabel("Rate")
    ax.set_ylim(0, 1)
    ax.legend()
    ax.grid(True, alpha=0.3)

    # --- row 1: max level reached ---
    ax = axes[1, 0]
    ax.plot(gens, stats["max_level_a"], label="Team A", color="royalblue")
    ax.plot(gens, stats["max_level_b"], label="Team B", color="tomato")
    # rolling max to show progression trend
    roll_a = np.convolve(stats["max_level_a"], np.ones(window) / window, mode="valid")
    roll_b = np.convolve(stats["max_level_b"], np.ones(window) / window, mode="valid")
    ax.plot(range(len(roll_a)), roll_a, color="royalblue", linestyle="--",
            alpha=0.5, label=f"A rolling avg ({window})")
    ax.plot(range(len(roll_b)), roll_b, color="tomato",    linestyle="--",
            alpha=0.5, label=f"B rolling avg ({window})")
    ax.set_title("Max level reached per generation")
    ax.set_xlabel("Generation")
    ax.set_ylabel("Level (1–8)")
    ax.set_yticks(range(1, 9))
    ax.legend()
    ax.grid(True, alpha=0.3)

    # --- row 1: level distribution heatmap ---
    ax = axes[1, 1]
    # count how often each level was the max, per team, across all gens
    level_counts_a = np.zeros(8)
    level_counts_b = np.zeros(8)
    for lvl in stats["max_level_a"]:
        level_counts_a[lvl - 1] += 1
    for lvl in stats["max_level_b"]:
        level_counts_b[lvl - 1] += 1
    x      = np.arange(1, 9)
    width  = 0.35
    ax.bar(x - width/2, level_counts_a, width, label="Team A", color="royalblue", alpha=0.7)
    ax.bar(x + width/2, level_counts_b, width, label="Team B", color="tomato",    alpha=0.7)
    ax.set_title("Distribution of max level reached")
    ax.set_xlabel("Max level")
    ax.set_ylabel("Count across generations")
    ax.set_xticks(x)
    ax.legend()
    ax.grid(True, alpha=0.3, axis="y")

    # --- row 2: draw/timeout rate + cumulative wins ---
    ax = axes[2, 0]
    ax.plot(range(len(draw)), draw, color="grey")
    ax.set_title(f"Draw/timeout rate (rolling {window}-gen window)")
    ax.set_xlabel("Generation")
    ax.set_ylabel("Rate")
    ax.set_ylim(0, 1)
    ax.grid(True, alpha=0.3)

    ax = axes[2, 1]
    cum_a = np.cumsum(winners == 0)
    cum_b = np.cumsum(winners == 1)
    ax.plot(gens, cum_a, label="Team A", color="royalblue")
    ax.plot(gens, cum_b, label="Team B", color="tomato")
    ax.set_title("Cumulative wins")
    ax.set_xlabel("Generation")
    ax.set_ylabel("Total wins")
    ax.legend()
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    plt.savefig(f"{directory_name}/training_stats.png", dpi=150)
    plt.show()
