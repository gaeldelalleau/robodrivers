class Agent():
    def __init__(self, team_id, policy):
        self.policy = policy
        self.team_id = team_id

    def forward(self, observation):
        return self.policy.action(self.team_id, observation)
