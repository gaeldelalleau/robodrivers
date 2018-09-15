# robodrivers

Autonomous agents roaming in the city's underground tunnels to collect resources.

Each team controls one agent, thus this is a competitive multi-agents setting.

![](https://raw.githubusercontent.com/gaeldelalleau/robodrivers/github_assets/screenshot.jpg)

## Installation

The easiest way to get started is to use docker-compose, but you can also use docker directly or install everything on your local computer.

### With docker-compose

```
docker-compose up
```
That should launch the web and server containers, and a client container with a default agent using a random policy.

### With Docker

#### Build the web, server and client containers with:

```
docker build web/ -t roboweb
docker build server/ -t roboserver
docker build client/ -t roboclient
```
Make sure to build the roboserver docker image first, because the client image depends on it.

#### Create a Docker network

Create an internal Docker network to allow the client container to access the server container:
```
docker network create robonet
```
Make sure to always run your containers on this network.

#### Run the containers

Web interface:
```
docker run --rm -p 8000:8000 -it roboweb
```
Note: The "-it" options allow to see logs in real time in the console.
If you want to run the web server in the background instead, you can use the "-d" option.

Rust server:
```
docker run --rm --network=robonet -p 3011:3011 -p 3012:3012 -it roboserver -v
```

The local IP address of your server container can now be seen in the output of:
```
docker network inspect robonet
```

Python client:
```
docker run --rm --network=robonet -it roboclient <arguments>
```
Try first to run the client container without arguments to see the help message.

Important: use the --host option on the client to tell it to connect to the correct IP address of the server.
 

## Getting started

### Visualize the game 

Once you get the roboweb and roboserver running, you can visualize the agents, the map and the scores in your browser.

Point your browser to http://127.0.0.1:8000

### Running the client to control your autonomous agent

Run the client, either locally with your own Python and Rust installation:
```
cd client
./roboclient.py --help
```
or through docker:
```
docker run --rm -it roboclient --help
```

To connect to the server and run actions with a default policy for your agent, do:
```
docker run --rm [--network=robonet] -it roboclient --host <server ip> run
```

Do not forget to also pass these authentication parameters if you are connecting to the remote challenge server:
```
--team TEAM_ID       Your team identifier
--token TOKEN        Your team token
```

#### Building the Rust dependency manually for the client (NOT NEEDED if you use Docker) 

If you want to run the client locally without using docker, you need to have compiled a dependency first: a Rust shared library.
First install the Rust toolchain on your system, and switch to the nightly toolchain.
Depending on your system, details may vary. On Unix it should be:
```
curl https://sh.rustup.rs -sSf | sh
[...]
rustup update nightly
rustup default nightly
``` 
Then build the library:
```
cd server
cargo build
```

## What's next?

### Run your autonomous vehicule better than the other teams

Now try to update the client's policy scripts to get a better score!

Have an idea for better competitive heuristics or path finding? Reinforcement learning? Genetic algorithms maybe?
Just go for it!

The first step to get started could be to edit these lines in roboclient.py:
```
    # Change these to run your own policies:
    policy = RandomPolicy()
    agent = Agent(policy)
```
in order to use your own policy class, or directly modify the file implementing the HeuristicPolicy class, for instance.

### Training a policy

For more advanced machine learning algorithms requiring training, the "train" command exists but is currently unimplemented in the client. That's your job!

Some hooks have been added to facilitate your job, quite similar to OpenAI Gym:
You should run the server with the -z command line option (short for "--remote\_control")
This will allow you to control the game state through two additional APIs:
```
rpc.step()      advance the game engine state by 1 tick
rpc.reset()     reset the game state
```

To observe the game state after calling the step() API, the websocket API can be used in a similar way as in the run() command of the client (queue.get())

### How to validate the flags

If you manage to reach certain score thresholds, flags will be unlocked.
You can see the flags you unlocked with:
```
docker run --rm -it roboclient --team TEAM_ID --token TOKEN --host <server ip> flags
```

Make sure to check them periodically, as the score thresholds may be updated during the day!

In order to get the points for your team, validate the flags by manually entering them in the CTFd interface.
 
