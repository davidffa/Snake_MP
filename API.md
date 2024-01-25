# Snake Multiplayer

- The multiplayer feature works using an UDP-based connection

## UDP Packets

### Hello packet

The client sends this packet when it connects to the server.

| Field   | Description  | Size    | Value |
|---------|--------------|---------| ----- |
| Type    | Packet type  | 1 byte  | 0x1   |

### Info packet

The server sends this packet in response to the Hello packet, indicating your snake ID, the position of all snakes and the position of the food.

| Field    | Description                     | Size    |
|----------|---------------------------------|---------|
| Type     | Packet type value = 0x2         | 1 byte  |
| YourID   | Your snake identifier           | 1 byte  |
| ID       | Snake identifier or food (0xff) | 1 byte  |
| Size     | Coord sequence size (LSB)       | 2 bytes |
| PointX   | x coordinate                    | 1 byte  |
| PointY   | y coordinate                    | 1 byte  |
| PointX   | x coordinate                    | 1 byte  |
| PointY   | y coordinate                    | 1 byte  |
| ...      | ...                             | 1 byte  |

### Food update packet

The server sends this packet when a new food is spawned

| Field    | Description               | Size    |
|----------|---------------------------|---------|
| Type     | Packet type value = 0x3   | 1 byte  |
| PointX   | x coordinate              | 1 byte  |
| PointY   | y coordinate              | 1 byte  |

### Direction update packet

The client sends this packet to inform a direction update.

| Field     | Description               | Size    |
|-----------|---------------------------|---------|
| Type      | Packet type value = 0x4   | 1 byte  |
| Direction | The direction             | 1 byte  |

### Snakes head update

The server sends a packet with all snakes heads positions (is this a good idea??)

| Field    | Description              | Size    |
|----------|--------------------------|---------|
| Type     | Packet type value = 0x5  | 1 byte  |
| ID       | Snake identifier         | 1 byte  |
| PointX   | x coordinate             | 1 byte  |
| PointY   | y coordinate             | 1 byte  |

### Snake connect packet

The server sends this packet to inform all the clients that a new snake connected.

| Field    | Description                     | Size    |
|----------|---------------------------------|---------|
| Type     | Packet type value = 0x6         | 1 byte  |
| ID       | Snake identifier                | 1 byte  |
| Size     | Coord sequence size (LSB)       | 1 byte  |
| PointX   | x coordinate                    | 1 byte  |
| PointY   | y coordinate                    | 1 byte  |
| ...      | ...                             | 1 byte  |

### Disconnect

The client sends this packet for gracefully disconnect.

| Field    | Description               | Size    |
|----------|---------------------------|---------|
| Type     | Packet type value = 0x9   | 1 byte  |

### SnakeDisconnect

The server sends this packet to inform a client has disconnected.

| Field    | Description               | Size    |
|----------|---------------------------|---------|
| Type     | Packet type value = 0xA   | 1 byte  |
| SnakeID  | The snake id              | 1 byte  |