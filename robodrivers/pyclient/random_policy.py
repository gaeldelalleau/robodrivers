from policy import Policy
from actions import import Action

class RandomPolicy(Policy):
    """
    Example of a basic policy chosing a random action
    """

    def __init__(self):
        super().__init__()

    def action(self, observation):
        """
        Chose an action at random, with a bias towards moving
        """
        raise "XXX TODO"
        # return Action()
