from ctypes import cdll, c_void_p, c_char_p, c_uint, cast
from sys import platform


class RustBindings():
    def __init__(self, lib_dir):
        if platform == 'darwin':
            prefix = 'lib'
            ext = 'dylib'
        elif platform == 'win32':
            prefix = ''
            ext = 'dll'
        else:
            prefix = 'lib'
            ext = 'so'

        path = '{}/{}robodrivers.{}'.format(lib_dir, prefix, ext)
        print('Loading Rust shared RPC library from path: {}'.format(path))
        self.lib = cdll.LoadLibrary(path)

    def __free_string(self, string):
        func = self.lib.free_string
        func.restype = None
        func.argtypes = [c_void_p]
        func(string)

    def rpc_action(self, host_and_port, team_id, token, action, tick):
        func = self.lib.rpc_action
        func.restype = c_void_p
        func.argtypes = [c_char_p, c_uint, c_char_p, c_char_p, c_uint]

        response_string = func(host_and_port.encode('utf-8'), team_id, token.encode('utf-8'),
                               action.encode('utf-8'), tick)
        response = cast(response_string, c_char_p).value.decode('utf-8')
        self.__free_string(response_string)

        return response

    def rpc_flags(self, host_and_port, team_id, token):
        func = self.lib.rpc_flags
        func.restype = c_void_p
        func.argtypes = [c_char_p, c_uint, c_char_p]

        response_string = func(host_and_port.encode('utf-8'), team_id, token.encode('utf-8'))
        response = cast(response_string, c_char_p).value.decode('utf-8')
        self.__free_string(response_string)
        return response

    def rpc_step(self, host_and_port, team_id, token):
        func = self.lib.rpc_step
        func.restype = c_void_p
        func.argtypes = [c_char_p, c_uint, c_char_p]

        response_string = func(host_and_port.encode('utf-8'), team_id, token.encode('utf-8'))
        response = cast(response_string, c_char_p).value.decode('utf-8')
        self.__free_string(response_string)
        return response

    def rpc_reset(self, host_and_port, team_id, token):
        func = self.lib.rpc_reset
        func.restype = c_void_p
        func.argtypes = [c_char_p, c_uint, c_char_p]

        response_string = func(host_and_port.encode('utf-8'), team_id, token.encode('utf-8'))
        response = cast(response_string, c_char_p).value.decode('utf-8')
        self.__free_string(response_string)
        return response

    def ping(self, host_and_port):
        func = self.lib.rpc_ping
        func.restype = c_void_p
        func.argtypes = [c_char_p]

        response_string = func(host_and_port.encode('utf-8'))
        response = cast(response_string, c_char_p).value.decode('utf-8')
        self.__free_string(response_string)

        return response
