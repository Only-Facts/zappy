# Zappy Server Architecture

## 1. Goal of the server

The `zappy_server` is the central authority of the Zappy project.

It is responsible for:

- accepting client connections
- reading data sent by clients
- sending the initial `WELCOME\n` message
- storing the server configuration
- running with a single process and a single thread
- using `poll`-style socket multiplexing through `mio`

The server must not block while waiting for one client.

## 2. Current state

Current implemented features:

- Rust server crate created
- command-line argument parsing
- TCP listener creation
- `mio` poll loop
- client accept handling
- `WELCOME\n` message sent to new clients
- basic client read handling
- named constants used instead of magic numbers

## 3. Project constraints

The server binary must be named:

```txt
zappy_server
````

Expected usage:

```txt
./zappy_server -p port -x width -y height -n name1 name2 ... -c clientsNb -f freq
```

Options:

```txt
-p port       server port
-x width      world width
-y height     world height
-n names      team names
-c clientsNb  authorized clients per team
-f freq       reciprocal of time unit
```

The reserved team name for the graphical client is:

```txt
GRAPHIC
```

When a client connects, the server first sends:

```txt
WELCOME\n
```

## 4. Current server flow

```txt
start program
parse arguments
validate configuration
create TCP listener
create poll instance
register listener in poll
enter main loop
wait for socket activity
accept new clients
send WELCOME message
read client data
remove disconnected clients
```

## 5. Server mental model

The current server has three main parts.

```txt
Argument layer
    reads command-line options
    creates the Config structure
    validates values

Network layer
    creates the TCP listener
    accepts new clients
    reads data from sockets

Poll layer
    waits for activity
    wakes up when a socket is ready
    allows several clients without blocking
```

The current rule is:

```txt
network code should only manage sockets
argument code should only manage configuration
poll code should only dispatch socket events
```

## 6. Current data structures

### `Config`

Stores the command-line configuration.

```rust
struct Config {
    port: u16,
    width: usize,
    height: usize,
    teams: Vec<String>,
    clients_nb: usize,
    freq: usize,
}
```

Fields:

* `port`: port used by the server
* `width`: map width
* `height`: map height
* `teams`: list of team names
* `clients_nb`: number of authorized clients per team
* `freq`: frequency used for Zappy time units

### `Client`

Stores a connected client.

```rust
struct Client {
    socket: TcpStream,
}
```

Fields:

* `socket`: TCP connection between the server and the client

## 7. Current main functions

### `main`

Entry point of the server.

Responsibilities:

* parse command-line arguments
* create the poll instance
* create the TCP listener
* register the listener in poll
* store connected clients
* run the main event loop

Current flow:

```txt
parse args
create listener
register listener into poll
loop:
    wait for socket activity
    if server socket is ready:
        accept new clients
    if client socket is ready:
        read client data
```

### `parse_args`

Reads the command-line arguments and builds a `Config`.

Example accepted command:

```bash
./zappy_server -p 4242 -x 10 -y 10 -n team1 team2 -c 3 -f 100
```

Returns:

```rust
Config {
    port,
    width,
    height,
    teams,
    clients_nb,
    freq,
}
```

### `parse_next_value`

Reads the value after a flag.

Example:

```txt
-p 4242
```

The flag is:

```txt
-p
```

The value is:

```txt
4242
```

This function converts the value from text into the expected type.

### `parse_team_names`

Reads every team name after `-n`.

Example:

```bash
-n team1 team2 team3
```

Result:

```rust
teams = ["team1", "team2", "team3"]
```

The function stops reading team names when it reaches another flag.

Example:

```bash
-n team1 team2 -c 3
```

It reads:

```txt
team1
team2
```

Then stops at:

```txt
-c
```

### `validate_config`

Checks if the parsed configuration is valid.

Current checks:

* port must be greater than 0
* width must be greater than 0
* height must be greater than 0
* at least one team must exist
* clients number must be greater than 0
* frequency must be greater than 0

### `create_listener`

Creates the TCP listener.

The listener is the socket that waits for new clients.

### `create_socket_address`

Builds the address used by the listener.

Example:

```txt
0.0.0.0:4242
```

This means the server listens on port `4242`.

### `accept_new_clients`

Accepts every pending client connection.

Current behavior:

* accepts the client
* gives it a unique poll token
* registers the client socket into poll
* sends `WELCOME\n`
* stores the client in the clients map

### `read_from_client`

Reads data from a connected client.

Current behavior:

* reads bytes from the socket
* prints the received message
* removes the client if it disconnects


## 8. How to test the current server

Run the server:

```bash
cargo run -- -p 4242 -x 10 -y 10 -n team1 team2 -c 3 -f 100
```

In another terminal, connect with `nc`:

```bash
nc localhost 4242
```

Expected client output:

```txt
WELCOME
```

Then type:

```txt
team1
```

Expected server output:

```txt
Received: team1
```
