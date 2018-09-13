from policy import Policy
from actions import Action
from actions import ActionType
from actions import Direction

import random


class RandomPolicy(Policy):
    """
    Example of a basic policy chosing a random action
    """

    def __init__(self):
        super().__init__()

    def action(self, team_id, observation):
        """
        Chose an action at random, with a bias towards moving
        """
        action_types = [item for item in ActionType]
        directions = [item for item in Direction]
        action_type = random.choices(action_types, weights=[10, 100, 1])[0]

        if action_type == ActionType.MOVE:
            direction = random.choices(directions)[0]
        else:
            direction = None

        action = Action(action_type, direction)
        return action
