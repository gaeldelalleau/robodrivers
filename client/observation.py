from enum import Enum
import numpy as np


class StateType(Enum):
    STOPPED = 1
    MOVING = 2

    def new(s):
        if isinstance(s, dict) and "MOVING" in s:
            return StateType.MOVING
        elif s == 'STOPPED':
            return StateType.STOPPED
        else:
            raise 'Unknown state'


class State():
    def __init__(self, state_type):
        self.state_type = state_type


class ItemType(Enum):
    PRODUCER = 1
    RESOURCE = 2
    BASE = 3


class CellType(Enum):
    OPEN = 1
    WALL = 2


class Cell():
    def __init__(self, cell_type, x, y):
        self.cell_type = cell_type
        self.x = x
        self.y = y
        self.items = []

    def set_items(self, items):
        self.items = items

    def is_wall(self):
        return self.cell_type == CellType.WALL

    def has_base(self):
        for i in self.items:
            if i.is_base():
                return True
        return False

    def has_producer(self):
        for i in self.items:
            if i.is_producer():
                return True
        return False

    def has_resource(self):
        for i in self.items:
            if i.is_resource():
                return True
        return False


class Item():
    def __init__(self, item_type):
        self.item_type = item_type

    def is_resource(self):
        return self.item_type == ItemType.RESOURCE

    def is_producer(self):
        return self.item_type == ItemType.PRODUCER

    def is_base(self):
        return self.item_type == ItemType.BASE


class Observation():
    def __init__(self, previous):
        self.previous_observation = previous

    def parse(self, game_state, team_id):
        self.cars = game_state['cars']
        self.map = game_state['map']
        self.team = game_state['teams'][str(team_id)]
        self.score = self.team['score']
        self.car = game_state['cars'][str(team_id)]
        self.resources = self.car['resources']
        self.health = self.car['health']
        self.killed = self.car['killed']
        self.collided = self.car['collided']
        self.state = State(StateType.new(self.car['state']))
        self.x = self.car['x']
        self.y = self.car['y']
        self.cells = []
        y = 0
        for r in self.map['cells']:
            x = 0
            # row = []
            # self.cells.append(row)
            for c in r:
                cell = Cell(CellType[c['block']], x, y)
                items = []
                for i in c['items']:
                    if isinstance(i, dict):
                        item_type = ItemType[list(i.keys())[0]]
                    else:
                        item_type = ItemType[i]
                    items.append(Item(item_type))
                # row.append(cell)
                self.cells.append(cell)
                x += 1
            y += 1

    def to_rl(self):
        o = []
        o.append(self.health)
        o.append(self.resources)
        for cell in self.cells:
            if cell.x == self.x and cell.y == self.y:
                o.append(1)
            else:
                o.append(0)
            o.append(int(cell.is_wall()))
            o.append(int(cell.has_base()))
            o.append(int(cell.has_producer()))
            o.append(int(cell.has_resource()))
        o = np.array(o, dtype=np.int8)
        return o

    # not used, I think
    def sample():
        o = []
        o.append(np.random.randint(0, 4))
        o.append(np.random.randint(0, 100))
        nb_cells = 20 * 24
        car_cell_index = np.random.randint(0, nb_cells)  # XXX car can be on a wall
        for i in nb_cells:
            if i == car_cell_index:
                o.append(1)
            else:
                o.append(0)
            has_wall = np.random.randint(0, 2)
            o.append(has_wall)
            if has_wall == 0:
                has_base = np.random.randint(0, 2)
                has_producer = np.random.randint(0, 2)
                has_resource = np.random.randint(0, 2)
            else:
                has_base = 0
                has_producer = 0
                has_resource = 0
            o.append(has_base)
            o.append(has_producer)
            o.append(has_resource)
        return o

    def to_reward(self):
        reward = 0

        if self.state.state_type != StateType.MOVING:
            reward -= 1

        if self.previous_observation is not None:
            reward += (self.score - self.previous_observation.score) * 10
            resources_bonus = self.resources - self.previous_observation.resources
            if resources_bonus > 0:
                reward += resources_bonus

        # if self.health decreased from previous time step : -1 (XXX once the other cars are added in the observation!)
        #    (or self.collided)
        #     TODO

        if self.killed:
            reward += -10 - self.resources*10

        return reward
