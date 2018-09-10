#!/usr/bin/env python3
"""websocket client, used to retrieve the game state from the remote server at each time step"""
import asyncio
import signal
from threading import Thread
# import sys

import aiohttp


def start_client(loop, url):
    # send request
    ws = yield from aiohttp.ClientSession().ws_connect(url, autoclose=False, autoping=False)

    """
    # input reader
    def stdin_callback():
        line = sys.stdin.buffer.readline().decode('utf-8')
        if not line:
            loop.stop()
        else:
            ws.send_str(line)
    loop.add_reader(sys.stdin.fileno(), stdin_callback)
    """

    @asyncio.coroutine
    def dispatch():
        while True:
            msg = yield from ws.receive()
            if msg.type == aiohttp.WSMsgType.TEXT:
                json = msg.data.strip()

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


def start_loop(loop, url):
    asyncio.set_event_loop(loop)
    loop.add_signal_handler(signal.SIGINT, loop.stop)
    asyncio.Task(start_client(loop, url))
    loop.run_forever()


def connect(url):
    # run WebSocket event loop in a new thread, to avoid blocking the main thread
    new_loop = asyncio.new_event_loop()
    t = Thread(target=start_loop, args=(new_loop, url))
    t.start()
