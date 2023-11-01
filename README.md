![Build](https://github.com/jproyo/shared-secrets/actions/workflows/build.yml/badge.svg)

# Distributed Shared Secrets Example

In this section, I will explore a minimalistic implementation of a Distributed Shared Secret using [Shamir's secret sharing](https://en.wikipedia.org/wiki/Shamir%27s_secret_sharing) with [Proactive Refreshing](https://en.wikipedia.org/wiki/Proactive_secret_sharing).

## Table of Contents
- [Preliminaries](#preliminaries)
- [Building the Project](#building-the-project)
- [Running the Program](#running-the-program)
- [Design Documentation](#design-documentation)
    - [Solution](#solution)
    - [Assumptions](#assumptions)
    - [Error Handling](#error-handling)
    - [Testing](#testing)
- [Future Work](#future-work)
- [Conclusions](#conclusions)

---

## Preliminaries

### Rust requirements

- **Cargo**: `cargo 1.72+`.
- **Rustc**: `rustc 1.72+`.
- **Cargo Make**: `cargo make 0.37+`

### Docker requirements

- **Docker**: `24+`
- **Docker Compose**: `2.21+`

### System requirements

- **Protobuf Compiler**: `3.21+`
> This is needed because there is a dependency with `riteraft` crate which requires protobuf binaries

---

## Building the Project

To build the project, run the following commands:

### Building with Rust

```shell
> cargo build
```

---

## Running the Program

The solution contains 2 binaries *Client* and *Server*. Since this is a distributed system, it will require to run more than one server instance.

In order to simplify running the solution, there is a `docker-compose` file inside `server/operations/docker` in order to run several nodes.

Specific configurations can be changed under `server/operations/config`. There is 1 configuration file for node instance and 1 environment file for the **API_KEY**.

### Run Server

To run the server do the following:

1. In foreground mode

```bash
docker compose -f server/operations/docker/docker-compose.yaml up
```

2. In background mode

```bash
docker compose -f server/operations/docker/docker-compose.yaml up -d
```

> IMPORTANT: It is important to run this command from the root of the project.


### Run Client
Now that all the nodes are running you can run the client against it, any time you want:

1. Go to `client` directory
2. If you want to create a secret and distribute the shares run:

```bash
API_KEY=RANDOM_GENERATED_KEY cargo run -- --command create --secret "my secret long"
```

You should set as environment variable `API_KEY` the same value of `server/operations/config/.env.local`. This is the Key that uses the clients to communicate with the nodes. You can change in both places the key.

3. If you want to recover the secret run:

```bash
API_KEY=RANDOM_GENERATED_KEY cargo run -- --command get
```

### Run Tests

1. Run unit tests

```bash
cargo test --lib
```

2. Run integration test

```bash
cargo make --cwd server tests
```

---

## Design Documentation
In this section, I will describe all the assumptions, decisions, and pending improvements that led to the current state of this software.

### Solution
The most important part of the solution is the distributed nodes servers which are keeping 1 Share of the secret each time a trusted client creates a secret and distribute the shares among the available nodes. Here is a brief description of the most important aspects of the solution:

- **Sharing Secret Model**: As it is described in the introduction a [Shamir's secret sharing](https://en.wikipedia.org/wiki/Shamir%27s_secret_sharing) with [Proactive Refreshing](https://en.wikipedia.org/wiki/Proactive_secret_sharing) was implemented. Some part of the implementation was done using `sss-rs` crate, but there was no crate that has Proactive Sharing implemented. The refreshing code, which generates a new random polynomial, it was done in this project and it is inside `sss-wrap` module used by `server`.

- **Creation and Retrieval of Shares**: For this part wee have implemented a simple REST API in order each node can receive a **Share** and return a **Share** if it is requested by a trusted user.

- **Security**: The communication between **Clients** and **Servers** is done with an **Authorization** API Key Header. Although it is a weak security mechanism, it is a layer of security in which all the participants needs to be trusted entities in the interaction.

- **Proactive Shares Refreshing**: The refreshing mechanism happen in some random node at some moment in time without client interaction. Since 1 node will take the lead to create the new random polynomial and distribute the evaluation for each `x` among the other nodes, a [**Raft**](https://raft.github.io/) consensus algorithm was implement to coordinate this distributed update. This was done using [riteraft](https://github.com/ritelabs/riteraft) crate.

- **Security in Consensus**: Consensus protocol is closed to the participants of the nodes and at this moment there is no Security extra layer implemented in the protocol.

### Assumptions

Here are some of the assumptions that were made during the development of this software:

- AS_1: **Security between Client/Server**: An authorization API_KEY Bearer token was implement. No extra authorization or security mechanism was implemented because it is assumed that this is an example and the modular approach of the code, as well as `actix-web` crate allows implementing any other sophisticated mechanism if we want.

- AS_2: **Security in Consensus**: None security was implemented. It is assumed that ports of the consensus protocol, are neither going to be open to the host machine, if it is running on `docker compose`, nor going to be running in a public network if it is not running with `docker`.

- AS_3: **Client**: Client implementation is minimal and lack of strong design principles. `client` module was develop in order to test the nodes and algorithm properly.

### Error Handling

All error handling are based on `thiserror` crate using an enum and relying on `Result` type.
There are 2 kind of errors:

### Testing

The focus of the testing was put on Integration test. For running the integration test it is important to use `cargo make` as it is explained above, because it is going to start a `docker compose` with 3 nodes that are synchronizing and the test is acting as a client hitting the real servers inside `docker`.

There are some **unit tests** as well, but only for important parts like the refreshing in `ConsensusHandler`.

---

## Future Work
This exercise left many opportunities for improving the current solution that could be addressed in future implementations:

- Implement better security mechanism between client and server, such as `TLS` and some other token refresh based authentication / authorization like `JWT-Set`.
- Implement security in consensus protocol to allow be open to any trusted participant. Here we can implement something similar like Client/Server security but it will require forking `riteraft` to control the `join` function [here](https://github.com/ritelabs/riteraft/blob/2e02abb0cb5e5bb9e1e9d256f5672fb0449c84f8/src/raft.rs#L109).

---

## Conclusions

In conclusion, the implementation of a client-server program in Rust for distributing and sharing secrets using Shamir's secret sharing and the Raft consensus protocol represents a remarkable feat of combining cryptographic security and distributed systems engineering. This work demonstrates the power of Rust's reliability and performance, ensuring the confidentiality and integrity of shared secrets. By seamlessly integrating two robust technologies, it opens doors to new horizons in secure and resilient communication, showcasing the potential for innovation at the intersection of cryptography and distributed systems.

I thoroughly enjoyed working on this exercise, and I hope readers find it equally engaging. Your feedback and observations are welcome!
