#!/usr/bin/env python3
"""websocket client, used to retrieve the game state from the remote server at each time step"""
import asyncio
from threading import Thread
# import sys

import aiohttp
from queue import Empty


def start_client(loop, url, queue):
    # send request
    ws = yield from aiohttp.ClientSession().ws_connect(url, autoclose=False, autoping=False, origin='Roboclient')

    @asyncio.coroutine
    def dispatch():
        while True:
            msg = yield from ws.receive()
            if msg.type == aiohttp.WSMsgType.TEXT:
                json_state = msg.data.strip()
                while not queue.empty():
                    try:
                        queue.get_nowait()
                    except Empty:
                        pass
                queue.put(json_state)
                # print('Text: ', msg.data.strip())
            elif msg.type == aiohttp.WSMsgType.BINARY:
                pass
                # print('Binary: ', msg.data)
            elif msg.type == aiohttp.WSMsgType.PING:
                ws.pong()
            elif msg.type == aiohttp.WSMsgType.PONG:
                pass
                # print('Pong received')
            else:
                if msg.type == aiohttp.WSMsgType.CLOSE:
                    yield from ws.close()
                elif msg.type == aiohttp.WSMsgType.ERROR:
                    print('Error during websocket receive %s' % ws.exception())
                elif msg.type == aiohttp.WSMsgType.CLOSED:
                    pass
                print('WebSocket connection terminated!')
                break

    yield from dispatch()


def start_loop(loop, url, queue):
    asyncio.set_event_loop(loop)
    # loop.add_signal_handler(signal.SIGINT, loop.stop)
    asyncio.Task(start_client(loop, url, queue))
    loop.run_forever()


def connect(url, queue):
    # run WebSocket event loop in a new thread, to avoid blocking the main thread
    new_loop = asyncio.new_event_loop()
    t = Thread(target=start_loop, args=(new_loop, url, queue))
    t.start()
