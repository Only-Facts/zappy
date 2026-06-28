TILE_SPAWN_DENSITY = {
    "food": 0.5,
    "linemate": 0.3,
    "deraumere": 0.15,
    "sibur": 0.1,
    "mendiane": 0.1,
    "phiras": 0.08,
    "thystame": 0.05
}

RESOURCE_KEYS = list(TILE_SPAWN_DENSITY.keys())

STONE_KEYS = [r for r in RESOURCE_KEYS if r != "food"]

EVOLUTION_TABLE = {
    # level: [nb_players linemate deraumere sibur mendiane phiras thystame]
    1: [1, 1, 0, 0, 0, 0, 0],
    2: [2, 1, 1, 1, 0, 0, 0],
    3: [2, 2, 0, 1, 0, 2, 0],
    4: [4, 1, 1, 2, 0, 1, 0],
    5: [4, 1, 2, 1, 3, 0, 0],
    6: [6, 1, 2, 3, 0, 1, 0],
    7: [6, 2, 2, 2, 2, 2, 1]
}

MAX_LEVEL = 8.0
MAX_PLAYERS = 6.0
MAX_STONE_REQ = 3.0

DIRECTION_VECTORS = {
    0: (0, -1),  # North
    1: (1,  0),  # East
    2: (0,  1),  # South
    3: (-1, 0),  # West
}

LEFT_ROT = {
    1: 7,
    2: 8,
    3: 1,
    4: 2,
    5: 3,
    6: 4,
    7: 5,
    8: 6,
}

RIGHT_ROT = {
    1: 3,
    2: 4,
    3: 5,
    4: 6,
    5: 7,
    6: 8,
    7: 1,
    8: 2,
}

BROADCAST_TABLE = {
    0: [7, 8, 1, 2, 3, 4, 5, 6],  # NORTH
    1: [1, 2, 3, 4, 5, 6, 7, 8],  # EAST
    2: [3, 4, 5, 6, 7, 8, 1, 2],  # SOUTH
    3: [5, 6, 7, 8, 1, 2, 3, 4],  # WEST
}

ACTIONS = {
    "INCANTATE":       0,
    "PUSH":            1,
    "MOVE_FORWARD":    2,
    "TURN_LEFT":       3,
    "TURN_RIGHT":      4,
    "TAKE_FOOD":       5,
    "TAKE_LINEMATE":   6,
    "TAKE_DERAUMERE":  7,
    "TAKE_SIBUR":      8,
    "TAKE_MENDIANE":   9,
    "TAKE_PHIRAS":    10,
    "TAKE_THYSTAME":  11,
    "DROP_FOOD":      12,
    "DROP_LINEMATE":  13,
    "DROP_DERAUMERE": 14,
    "DROP_SIBUR":     15,
    "DROP_MENDIANE":  16,
    "DROP_PHIRAS":    17,
    "DROP_THYSTAME":  18,
    "EAT":            19,
    "FORK":           20,
    "BROADCAST_ADV":  21,
    "BROADCAST_INV":  22,
}

MAX_VISION_TILES = 64 # lvl 7 so 7 rows (we stop once lvl 8 get so don't need to calculate it)

MAX_SURVIVAL = 126
