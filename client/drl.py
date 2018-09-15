#!/usr/bin/env python3
import numpy as np

from keras.models import Sequential
from keras.layers import Dense, Activation, Flatten
from keras.optimizers import Adam

from rl.agents.dqn import DQNAgent
from rl.policy import BoltzmannQPolicy
from rl.memory import SequentialMemory

from roborl import RoboEnv

# Setup Environment
env = RoboEnv()
np.random.seed(1)
env.seed(1)
nb_actions = 6

# Build Model
model = Sequential()
model.add(Flatten(input_shape=(1,) + env.observation_space.shape))
model.add(Dense(16))
model.add(Activation('relu'))
model.add(Dense(16))
model.add(Activation('relu'))
model.add(Dense(16))
model.add(Activation('relu'))
model.add(Dense(nb_actions))
model.add(Activation('linear'))
print(model.summary())

# Configure
memory = SequentialMemory(limit=50000, window_length=1)
policy = BoltzmannQPolicy()
dqn = DQNAgent(model=model, nb_actions=nb_actions, memory=memory, nb_steps_warmup=10,
               target_model_update=1e-2, policy=policy)

# Compile
dqn.compile(Adam(lr=1e-3), metrics=['mae'])

# Train
dqn.fit(env, nb_steps=10000, visualize=False, verbose=2, nb_max_episode_steps=1000)

# Persist
dqn.save_weights('dqn_{}_weights.h5f'.format("roborl"), overwrite=True)

# Test
dqn.test(env, nb_episodes=5, visualize=True, nb_max_episode_steps=500)

print("FINISHED!")
