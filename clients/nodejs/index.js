const net = require('net');

function sendCommand(command) {
    const client = new net.Socket();

    client.connect(6379, '127.0.0.1', () => {
        console.log(`Sending: ${command}`);
        client.write(command);
    });

    client.on('data', (data) => {
        console.log(`Received: ${data}`);
        client.destroy();
    });

    client.on('close', () => {
        console.log('Connection closed');
    });

    client.on('error', (err) => {
        console.error(`Error: ${err.message}`);
    });
}

const commands = [
    "GEOADD location1 37.7749 -122.4194\n",
    "GEOADD location2 34.0522 -118.2437\n",
    "GEOSEARCH 37.7749 -122.4194 500000\n"
];

commands.forEach(command => sendCommand(command));
