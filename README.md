# Redis Server Clone

## Overview

This project is a learning endeavor to create a clone of a Redis server. It aims to replicate the basic functionalities of Redis, focusing on key-value storage, replication, and basic command processing. This project is purely educational and leverages only the Rust standard library, making it an excellent resource for understanding the internals of a Redis server and the Rust programming language.

**Note:** This project is intended for learning purposes and should not be used in production environments.

## Features

- Basic key-value storage
- Replication support to mimic master-replica dynamics
- Command processing for a subset of Redis commands
- Custom TCP server implementation
- Command-line interface for server configuration

## Getting Started

### Prerequisites

- Rust (latest stable version recommended)

### Installation

1. Clone the repository:

```sh
git clone https://github.com/florian-trehaut/redis-server-clone.git
```

2. Navigate to the project directory:

```sh
cd redis-server
```

3. Build the project:

```sh
cargo build --release
```

### Running the Server

To start the server with default configurations:

```sh
cargo run --release
```

To configure the server as a master or replica, use the command-line arguments:

```sh
cargo run --release -- [--port <port_of_master>]
```

Or:

```sh
cargo run --release -- --replicaof <hostname> <port_of_master> [--port <port_of_replica>]
```

## Configuration

The server can be configured either as a master or a replica through command-line arguments. The configuration is managed in the `src/server_config/server.rs` file, which parses the arguments and sets up the server accordingly.
