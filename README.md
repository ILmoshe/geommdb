# Geommdb

Geommdb is a geospatial in-memory database built with Rust. It supports basic geospatial operations such as adding and searching geospatial points, and it ensures persistence using a write-ahead log (WAL) and periodic snapshots.

### Contribution

We welcome contributions to the **geommdb** project! We are looking for contribution which wil design, develop, discuss the project, If you're interested in improving the codebase, adding new features, or fixing bugs, please follow these steps:

1. **Fork the Repository**: Click the "Fork" button at the top right of this repository to create your own fork.
2. **Clone Your Fork**: Clone your forked repository to your local machine using `git clone <your-forked-repo-url>`.
3. **Create a Branch**: Create a new branch for your feature or bugfix with a descriptive name using `git checkout -b feature-or-bugfix-name`.
4. **Make Changes**: Implement your changes in the codebase. Ensure you follow the existing code style and conventions.
5. **Commit Your Changes**: Commit your changes with a descriptive commit message using `git commit -m "Description of your changes"`.
6. **Push to Your Fork**: Push your branch to your forked repository using `git push origin feature-or-bugfix-name`.
7. **Create a Pull Request**: Open a pull request to the main repository. Provide a detailed description of your changes and any relevant information.

### Contact

If you have any questions or need assistance, feel free to open an issue or reach out to the maintainers.

We appreciate your contributions and look forward to collaborating with you!


## Features

- **In-memory storage**: Fast access to geospatial data.
- **Geospatial operations**: Add and search for points within a given radius.
- **Persistence**: Data is persisted using WAL and snapshots to ensure durability.
- **Concurrency**: Handles multiple clients concurrently using Tokio.

## Getting Started

### Prerequisites

- **Rust**: Install Rust from [rust-lang.org](https://www.rust-lang.org/). (for running localy)
- **Docker**: Install Docker from [docker.com](https://www.docker.com/). (for running in a container)

### Installation

#### run local:

1. **Clone the repository**:

   ```sh
   git clone https://github.com/ILmoshe/geommdb.git
   cd geommdb
   ```

2. **Build the project**:

   ```sh
   cargo build --release
   ```

3. **Run the server**:
   ```sh
   cargo run --release
   ```

#### Docker

1. **Pull the Docker container**

   ```sh
   docker pull moshemiz/geomemdb
   ```

2. **Run the Docker container**:
   ```sh
   docker run -p 6379:6379 geommdb
   ```

### Usage

Once the server is running, you can interact with it using TCP clients. Below are the supported commands:

- **GEOADD**: Add a geospatial point.

  ```
  GEOADD key latitude longitude
  ```

  Example:

  ```
  GEOADD point1 40.7128 -74.0060
  ```

- **GEOSEARCH**: Search for points within a radius.
  ```
  GEOSEARCH latitude longitude radius
  ```
  Example:
  ```
  GEOSEARCH 40.7128 -74.0060 10
  ```

### Running localy with Docker

1. **Build the Docker image**:

   ```sh
   docker build -t geommdb .
   ```

2. **Run the Docker container**:
   ```sh
   docker run -p 6379:6379 geommdb
   ```

### Project Structure

- `src/main.rs`: Entry point of the application.
- `src/network.rs`: Handles TCP connections and command parsing.
- `src/storage.rs`: Contains the `GeoDatabase` struct and its geospatial operations.
- `src/persistence.rs`: Manages WAL and snapshot operations for data persistence.

### Example Usage

Note: We don't have official libraries for Python yet, but here's a snapshot of how you might interact with the server using Python's built-in libraries:

#### Python

```python
import socket


def send_command(command):
    server_address = ("127.0.0.1", 6379)
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    try:
        sock.connect(server_address)

        print(f"Sending: {command}")
        sock.sendall(command.encode("utf-8"))

        response = sock.recv(1024)
        print(f'Received: {response.decode("utf-8")}')

    finally:
        sock.close()


if __name__ == "__main__":
    send_command("GEOADD location1 37.7749 -122.4194")
    send_command("GEOADD location2 34.0522 -118.2437")
    send_command("GEOSEARCH 37.7749 -122.4194 500000\n")
```

#### Node.js

```javascript
const net = require("net");

function sendCommand(command) {
  const client = new net.Socket();

  client.connect(6379, "127.0.0.1", () => {
    console.log(`Sending: ${command}`);
    client.write(command);
  });

  client.on("data", (data) => {
    console.log(`Received: ${data}`);
    client.destroy();
  });

  client.on("close", () => {
    console.log("Connection closed");
  });

  client.on("error", (err) => {
    console.error(`Error: ${err.message}`);
  });
}

const commands = [
  "GEOADD location1 37.7749 -122.4194\n",
  "GEOADD location2 34.0522 -118.2437\n",
  "GEOSEARCH 37.7749 -122.4194 500000\n",
];

commands.forEach((command) => sendCommand(command));
```
