from rl.core import Env
from rl.core import Space
import websocket
from queue import Queue
import json

import numpy as np

from rust_bindings import RustBindings
from rpc import Rpc
from actions import Action
from observation import Observation


class RoboEnv(Env):
    def __init__(self):
        self.action_space = ActionSpace()
        self.observation_space = ObservationSpace()
        self.previous_observation = None
        self.init()

    def init(self, lib_dir="../server/target/debug", ip='127.0.0.1', rpc_port=3011, ws_port=3012,
             team_id=1, token="XXX1"):
        self.team_id = team_id
        self.tick = 0
        self.rust_bindings = RustBindings(lib_dir)
        self.rpc = Rpc(self.rust_bindings, ip, rpc_port, team_id, token)
        ws_url = 'ws://{}:{}'.format(ip, ws_port)
        self.queue = Queue()
        websocket.connect(ws_url, self.queue)

    def observe(self):
        json_state = self.queue.get()
        game_state = json.loads(json_state)

        observation = None
        done = False

        observation = Observation(self.previous_observation)
        observation.parse(game_state, self.team_id)
        self.previous_observation = observation
        return observation.to_rl(), observation.to_reward(), done

    def step(self, action):
        """
        Accepts an action and returns a tuple (observation, reward, done, info).
        """
        self.rpc.action(Action.from_rl(action), self.tick)
        self.rpc.step()
        observation, reward, done = self.observe()

        return observation, reward, done, {}

    def reset(self):
        """
        returns observation (object): The initial observation of the space. Initial reward is assumed to be 0.
        """
        self.previous_observation = None
        self.rpc.reset()
        self.rpc.step()
        observation, reward, done = self.observe()
        return observation

    def render(self, mode='human', close=False):
        pass

    def close(self):
        pass

    def seed(self, seed=None):
        """
        Returns the list of seeds used in this env's random number generators
        """
        return [seed]

    def configure(self, *args, **kwargs):
        pass


class ObservationSpace(Space):
    def __init__(self):
        self.shape = (20*24*5 + 2,)  # XXX
        super().__init__()

    def sample(self, seed=None):
        """Uniformly randomly sample a random element of this space.
        """
        return Observation.sample()

    def contains(self, x):
        """Return boolean specifying if x is a valid member of this space
        """
        raise NotImplementedError()


class ActionSpace(Space):
    def __init__(self):
        super().__init__()

    def sample(self, seed=None):
        """Uniformly randomly sample a random element of this space.
        """
        return Action.sample()

    def contains(self, x):
        """Return boolean specifying if x is a valid member of this space
        """
        raise NotImplementedError()
