class Rpc():
    def __init__(self, rust_bindings, host, port, team_id, token):
        self.rust_bindings = rust_bindings
        self.host = host
        self.port = port
        self.host_and_port = self.host + ':' + str(self.port)
        self.team_id = team_id
        self.token = token

    def check(self, response):
        if response.startswith('ERROR'):
            print(response)
        return response

    def action(self, action, tick):
        action = action.to_json()
        return self.check(self.rust_bindings.rpc_action(self.host_and_port, self.team_id, self.token, action, tick))

    def flags(self):
        return self.check(self.rust_bindings.rpc_flags(self.host_and_port, self.team_id, self.token))

    def ping(self):
        return self.check(self.rust_bindings.ping(self.host_and_port))
