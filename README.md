# robodrivers

Autonomous agents roaming in the city's underground tunnels to collect resources.

Each team controls one agent.

## Installation

### With docker-compose

```
docker-compose up
```

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
docker run --rm --network=robonet -p 3011:3011 -p 3012:3012 -it roboserver
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

### Running the client

Run the client, either locally with your own Python and Rust installation:
```
cd client
./roboclient.py --help
```
or through docker:
```
docker run --rm --network=robonet -it roboclient --help
```

```
To connect to the server and run actions with a default policy, do:
```
docker run --rm --network=robonet -it roboclient --host <server ip> run 
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

The "train" command is currently unimplemented in the client. That's your job too :)

### How to validate the flags

If you manage to reach certain score thresholds, flags will be unlocked.
You can see the flags you unlocked with:
```
docker run --rm --network=robonet -it roboclient --host <server ip> flags 
```

Make sure to check them periodically, as the score thresholds may be updated during the day!

In order to get the points for your team, validate the flags by manually entering them in the CTFd interface.
 
