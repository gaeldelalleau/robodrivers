class Agent():
    def __init__(self, policy):
        self.policy = policy

    def forward(self, observation):
        return self.policy.action(observation)
