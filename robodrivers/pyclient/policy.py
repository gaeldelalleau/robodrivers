class Policy():
    """
    Extend this class to implement your own policy
    """

    def __init__(self):
        pass

    def action(self, observation):
        """
        Should return an Action, selected on the basis of the current observation of the environment
        """
        raise NotImplementedError()
