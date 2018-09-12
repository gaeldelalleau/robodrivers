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


Run the client, either locally with your own Python and Rust installation, or through Docker:
```
docker run roboclient ./roboclient.py --help
```


## Game rules

