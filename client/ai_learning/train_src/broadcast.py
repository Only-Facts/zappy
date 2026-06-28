from constants import STONE_KEYS, MAX_LEVEL, MAX_PLAYERS, MAX_STONE_REQ

class Broadcast:
    def __init__(
        self,
        msg_type,
        level,
        players_needed,
        missing_stones
    ):
        self.msg_type = msg_type
        self.level = level
        self.players_needed = players_needed
        self.missing_stones = missing_stones
        self.direction = 0

    @staticmethod
    def create_msg(agent, players_needed, missing_resources, msg_type):
        if msg_type == "adv":
            return Broadcast(
                msg_type,
                agent.level,
                players_needed,
                missing_resources
            )
        if msg_type == "inv":
            return Broadcast(
                msg_type,
                agent.level,
                1,
                [agent.inventory[r] for r in STONE_KEYS]
            )
    
    def normalize(self, broadcast):
        broadcast[0] = 1.0
        broadcast[1 + self.direction] = 1.0

        broadcast[10] = self.level / MAX_LEVEL
        broadcast[11] = self.players_needed / MAX_PLAYERS
        for i, val in enumerate(self.missing_stones):
            broadcast[12 + i] = val / MAX_STONE_REQ
        broadcast[18] = 1.0 if self.msg_type == "adv" else 0.0
        broadcast[19] = 1.0 if self.msg_type == "inv" else 0.0
        return broadcast
