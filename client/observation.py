from enum import Enum
import numpy as np
from actions import Direction


class Car():
    def __init__(self, car):
        self.x = int(car['x'])
        self.y = int(car['y'])
        self.health = int(car['health'])
        self.resources = int(car['resources'])
        self.collided = bool(car['collided'])
        self.killed = bool(car['killed'])
        self.state = State(car['state'])


class StateType(Enum):
    STOPPED = 1
    MOVING = 2


class State():
    def __init__(self, state):
        if isinstance(state, dict):
            self.state_type = StateType[list(state.keys())[0]]
            self.direction = Direction[state[list(state.keys())[0]]]
        else:
            self.state_type = StateType[state]
            self.direction = None


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
        self.map = game_state['map']
        self.cars = {str(k): Car(v) for k, v in game_state['cars'].items()}

        self.team = game_state['teams'][str(team_id)]
        self.score = self.team['score']
        self.car = self.cars[str(team_id)]

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
        self.size_x = x
        self.size_y = y

    def get_car_at(self, cell):
        for car in self.cars.values():
            if car.x == cell.x and car.y == cell.y:
                return car
        return None

    def to_rl(self):
        o = []
        o.append(self.car.health)
        o.append(self.car.resources)
        o.append(self.car.collided)
        o.append(self.car.killed)
        c = []
        for cell in self.cells:
            c.append(int(cell.is_wall()))
            c.append(int(cell.has_base()))
            c.append(int(cell.has_producer()))
            c.append(int(cell.has_resource()))
            car = self.get_car_at(cell)
            if car is not None:
                c.append(int(car.state.state_type == StateType.MOVING and car.state.direction == Direction.NORTH))
                c.append(int(car.state.state_type == StateType.MOVING and car.state.direction == Direction.SOUTH))
                c.append(int(car.state.state_type == StateType.MOVING and car.state.direction == Direction.EAST))
                c.append(int(car.state.state_type == StateType.MOVING and car.state.direction == Direction.WEST))
                c.append(car.resources)
                c.append(car.health)
                c.append(car.collided)
            else:
                for _ in range(7):
                    c.append(0)

        cell_index = self.car.x + self.car.y*self.size_y
        # Center observations on the car
        o += c[cell_index:] + c[:cell_index]
        o = np.array(o, dtype=np.int8)
        return o

    # not used
    def sample():
        raise 'Observation.sample called! Not really tested/implemented...'
        o = []
        o.append(np.random.randint(0, 4))
        o.append(np.random.randint(0, 100))
        nb_cells = 20 * 24  # XXX
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

        if self.car.state.state_type != StateType.MOVING:
            reward -= 1

        if self.previous_observation is not None:
            reward += (self.score - self.previous_observation.score) * 100
            # resources_bonus = self.car.resources - self.previous_observation.car.resources
            # if resources_bonus > 0:
            #     reward += resources_bonus * 10

        if self.car.collided:  # XXX tune penalty depending on health?
            reward -= 2

        if self.car.killed:
            reward += -10 - self.car.resources*100

        return reward
