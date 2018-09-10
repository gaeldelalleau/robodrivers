#!/usr/bin/env python3
"""Python client for Robodrivers AI Challenge @ICON 2018"""
import argparse

import websocket
from rpc import Rpc


from agent import Agent
from random_policy import RandomPolicy


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
    parser.add_argument('--host', action="store", dest='host', default='127.0.0.1"',
                        help='Hostname of server hosting the Robodriver service')
    parser.add_argument('--rpc-port', action="store", dest='rpc_port', default=3011, type=int,
                        help='RPC port number (used to send the agent\'s actions to the server)')
    parser.add_argument('--ws-port', action="store", dest='ws_port', default=3012, type=int,
                        help='WebSocket port number (used to retrieve the game state at each time step)')
    parser.add_argument('action', choices=['train', 'run', 'show_flags'],
                        help="Select action to perform: **train** trains a policy locally | " +
                             "**run** run an agent using a policy on the selected host (if you want " +
                             "to score points, make sure to set --host to the ip address of " +
                             "the challenge server) | " +
                             "**show_flags** shows the flags you currently unlocked on the challenge (make " +
                             "sure to score them manually in the CTFd interface!)")
    return parser.parse_args()


def run(args):
    ws_url = 'http://{}:{}/ws/'.format(args.host, args.ws_port)
    websocket.connect(ws_url)
    rpc = Rpc(args.host, args.rpc_port)

    # Change these to run your own policies:
    policy = RandomPolicy()
    agent = Agent(policy)

    while True:
        # TODO: wait for next tick <=> new game_state received
        observation = None
        action = agent.forward(observation)
        rpc.action(action)


def show_flags(args):
    rpc = Rpc(args.host, args.rpc_port)
    text = rpc.flags()
    print(text)


def train(args):
    pass


def main():
    args = parse_args()
    if args.action == 'run':
        run(args)
    elif args.action == 'train':
        train(args)
    elif args.action == 'show_flags':
        show_flags(args)


if __name__ == '__main__':
    main()
