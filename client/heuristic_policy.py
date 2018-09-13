from policy import Policy
from random_policy import RandomPolicy
from actions import Action
from actions import ActionType
from actions import Direction


class HeuristicPolicy(Policy):
    """
    Example of a basic policy using simple heuristics
    """

    def __init__(self):
        self.random_steps = 0
        super().__init__()

    def parse(self, observation, team_id):
        """
        quick-and-dirty, non-exhaustive, parsing of the observed game state
        """
        self.observation = observation
        self.team_id = team_id
        self.cars = observation['cars']
        self.map = observation['map']
        self.team = observation['teams'][str(team_id)]
        self.score = ['score']
        self.car = observation['cars'][str(team_id)]
        self.resources = self.car['resources']
        self.x = self.car['x']
        self.y = self.car['y']
        self.health = self.car['health']
        self.cells = []
        for r in self.map['cells']:
            row = []
            self.cells.append(row)
            for cell in r:
                row.append(cell)

    def blocked(self, x, y):
        return self.cells[y][x]['block'] == 'WALL'

    def path_find(self, x, y):
        """
        "A-star" is probably a cloth brand from some US rap singer...
        Let's do our own stupid greedy, short-sighted algo instead, right?
        """
        if self.x == x and self.y == y:
            return Action(ActionType.STOP)

        direction = None
        if self.y > y:
            if not self.blocked(self.x, self.y - 1):
                direction = Direction.NORTH
        elif self.y < y:
            if not self.blocked(self.x, self.y + 1):
                direction = Direction.SOUTH
        if direction is None and self.x > x:
            if not self.blocked(self.x - 1, self.y):
                direction = Direction.WEST
        elif direction is None and self.x < x:
            if not self.blocked(self.x + 1, self.y):
                direction = Direction.EAST

        if direction is not None:
            return Action(ActionType.MOVE, direction)
        else:
            self.random_steps = 10
            self.random_action = RandomPolicy().action(self.team_id, self.observation)
            return self.random_action

    def find_cell(self, cell_type):
        """
        cell_type can be either "WALL" or "OPEN"
        """
        y = 0
        for row in self.cells:
            x = 0
            for cell in row:
                if cell['block'] == cell_type:
                    return cell, x, y
                x += 1
            y += 1
        return None, 0, 0

    def find_item(self, item_type):
        """
        item_type can be "BASE", "RESOURCE" or "PRODUCER"
        """
        y = 0
        for row in self.cells:
            x = 0
            for cell in row:
                items = cell['items']
                for item in items:
                    if item == item_type or (isinstance(item, dict) and item_type in item):
                        return cell, x, y
                x += 1
            y += 1
        return None, 0, 0

    def action(self, team_id, observation):
        if self.random_steps > 0: # super smart way of avoiding to get stuck ;)
            self.random_steps -= 1
            return self.random_action

        self.parse(observation, team_id)

        if self.resources > 0 or self.health <= 2:
            # return to base
            _, target_x, target_y = self.find_item("BASE")
        else:
            # find some resource
            resource, target_x, target_y = self.find_item("RESOURCE")
            if resource is None:
                return Action(ActionType.STOP)
        return self.path_find(target_x, target_y)
