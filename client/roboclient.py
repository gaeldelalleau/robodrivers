#!/usr/bin/env python3
"""Python client for Robodrivers AI Challenge @ICON 2018"""
import argparse
import json
from queue import Queue

import websocket
from rpc import Rpc
from agent import Agent
from random_policy import RandomPolicy
from rust_bindings import RustBindings


rust_bindings = None
rpc = None


def parse_args():
    ARGS = argparse.ArgumentParser(
        description="websocket console client for wssrv.py example.")
    ARGS.add_argument(
        '--host', action="store", dest='host',
        default='127.0.0.1', help='Host name')
    ARGS.add_argument(
        '--port', action="store", dest='port',
        default=3012, type=int, help='Port number')

    parser = argparse.ArgumentParser(description="Python client for Robodrivers AI Challenge @ICON 2018")
    parser.add_argument('--host', action="store", dest='host', default='127.0.0.1',
                        help='Hostname of server hosting the Robodriver service')
    parser.add_argument('--rpc-port', action="store", dest='rpc_port', default=3011, type=int,
                        help='RPC port number (used to send the agent\'s actions to the server)')
    parser.add_argument('--ws-port', action="store", dest='ws_port', default=3012, type=int,
                        help='WebSocket port number (used to retrieve the game state at each time step)')
    parser.add_argument('--lib_dir', action="store", dest='lib_dir', default='../target/debug',
                        help='Directory holding the Rust RPC shared library used to connect with the server')
    parser.add_argument('--team', action="store", dest='team_id', default=1, type=int,
                        help='Your team identifier')
    parser.add_argument('--token', action="store", dest='token', default='XXX1',
                        help='Your team token')
    parser.add_argument('action', choices=['train', 'run', 'flags'],
                        help="Select action to perform: **train** trains a policy locally | " +
                             "**run** run an agent using a policy on the selected host (if you want " +
                             "to score points, make sure to set --host to the ip address of " +
                             "the challenge server) | " +
                             "**flags** shows the flags you currently unlocked on the challenge (make " +
                             "sure to score them manually in the CTFd interface!)")
    return parser.parse_args()


def observe(game_state):
    return game_state


def get_tick(game_state):
    return int(game_state['tick'])


def run(args):
    ws_url = 'ws://{}:{}'.format(args.host, args.ws_port)
    queue = Queue()
    websocket.connect(ws_url, queue)

    # Change these to run your own policies:
    policy = RandomPolicy()
    agent = Agent(policy)

    tick = None

    while True:
        json_state = queue.get()
        game_state = json.loads(json_state)
        new_tick = get_tick(game_state)
        if tick is not None:
            if new_tick == tick:
                print('Warning: we received game state with the same tick twice...')
                continue
            elif new_tick < tick:
                print('Warning: we received an earlier tick, maybe the server state was restarted?')
            elif new_tick != tick + 1:
                print('Warning: it seems that we skipped some game steps')
        tick = new_tick

        print('tick: ' + str(tick))
        print('game state["teams"]: ' + repr(game_state['teams']))

        observation = observe(game_state)
        action = agent.forward(observation)
        rpc.action(action, tick)


def show_flags(args):
    response = rpc.flags()
    print(response)


def train(args):
    pass


def main():
    global rust_bindings
    global rpc

    args = parse_args()

    rust_bindings = RustBindings(args.lib_dir)
    rpc = Rpc(rust_bindings, args.host, args.rpc_port, args.team_id, args.token)

    if args.action == 'run':
        run(args)
    elif args.action == 'train':
        train(args)
    elif args.action == 'flags':
        show_flags(args)


if __name__ == '__main__':
    main()
