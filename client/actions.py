import json
from json import JSONEncoder
from enum import Enum


class Direction(Enum):
    NORTH = 1
    SOUTH = 2
    EAST = 3
    WEST = 4


class ActionType(Enum):
    STOP = 1
    MOVE = 2
    SUICIDE = 3


class Action():
    def __init__(self, action_type, direction=None):
        if not isinstance(action_type, ActionType):
            raise 'Invalid action type'
        if direction is not None and not isinstance(direction, Direction):
            raise 'Invalid directione'
        self.action_type = action_type
        self.direction = direction

    def to_json(self):
        return json.dumps(self, cls=ActionEncoder)


class ActionEncoder(JSONEncoder):
        def default(self, obj):
            if isinstance(obj, Action):
                action_type = str(obj.action_type).split('.')[1]
                if obj.direction is not None:
                    action = {}
                    action[action_type] = str(obj.direction).split('.')[1]
                else:
                    action = action_type
                return action
            return json.JSONEncoder.default(self, obj)
