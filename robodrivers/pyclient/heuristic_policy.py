from policy import Policy
from actions import import Action

class HeuristicPolicy(Policy):
    """
    Example of a basic policy using simple heuristics
    """

    def __init__(self):
        super().__init__()

    def action(self, observation):
        raise "XXX TODO"
        # return Action()
