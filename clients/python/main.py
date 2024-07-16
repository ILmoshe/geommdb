import socket


def send_command(command: str):
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
    # send_command("GEOADD location1 37.7749 -122.4194")
    # send_command("GEOADD location2 34.0522 -118.2437")
    # # send_command("GEOSEARCH 37.7749 -122.4194 500000\n")
    # send_command("GEOGET location1")
    # # send command geoadd polygon
    # send_command("GEOADD polygon1 37.7749 -122.4194 37.7749 -122.4194 37.7749 -122.4194 37.7749 -122.4194")
    send_command("GEOGET polygon1")
