from rl.policy import Policy
import numpy as np


class RlPolicy(Policy):
    def __init__(self, eps1=.8, eps2=0.1):
        super().__init__()
        self.eps1 = eps1
        self.eps2 = eps2
        self.previous_action = None

    def select_action(self, q_values):
        """Return the selected action
        # Arguments
            q_values (np.ndarray): List of the estimations of Q for each action
        # Returns
            Selection action
        """
        assert q_values.ndim == 1
        nb_actions = q_values.shape[0]

        draw = np.random.uniform()
        if draw < self.eps2:
            action = np.random.random_integers(2, nb_actions-1)  # avoid action 1 (stop) and 2 (suicide)
        elif draw < self.eps1 and self.previous_action is not None:
            action = self.previous_action
        else:
            action = np.argmax(q_values)
        self.previous_action = action  # XXX it should be reset to None when a new episode starts. How?!
        return action

    def get_config(self):
        config = super().get_config()
        config['eps1'] = self.eps1
        config['eps2'] = self.eps2
        return config
