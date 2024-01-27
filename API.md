# Snake Multiplayer

The multiplayer feature works using an TCP-based connection, with a custom binary protocol described below.

## TCP Packets

### Info packet

The server sends this packet to a newly connected client, indicating its snake ID, the position of all snakes and the position of the food.

| Field    | Description                     | Size    |
|----------|---------------------------------|---------|
| Type     | Packet type value = 0x1         | 1 byte  |
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
| Type     | Packet type value = 0x2   | 1 byte  |
| PointX   | x coordinate              | 1 byte  |
| PointY   | y coordinate              | 1 byte  |

### Direction update packet

The client sends this packet to inform a direction update.

| Field     | Description               | Size    |
|-----------|---------------------------|---------|
| Type      | Packet type value = 0x3   | 1 byte  |
| Direction | The direction             | 1 byte  |

### Snakes head update

The server broadcasts this packet every 50ms, with the all snake heads

| Field    | Description              | Size    |
|----------|--------------------------|---------|
| Type     | Packet type value = 0x4  | 1 byte  |
| ID       | Snake identifier         | 1 byte  |
| PointX   | x coordinate             | 1 byte  |
| PointY   | y coordinate             | 1 byte  |

### Snake connect packet

The server sends this packet to inform all the clients that a new snake connected.

| Field    | Description               | Size    |
|----------|---------------------------|---------|
| Type     | Packet type value = 0x5   | 1 byte  |
| ID       | Snake identifier          | 1 byte  |
| Size     | Coord sequence size (LSB) | 2 bytes |
| PointX   | x coordinate              | 1 byte  |
| PointY   | y coordinate              | 1 byte  |
| ...      | ...                       | 1 byte  |


### SnakeDisconnect

The server sends this packet to inform a client has disconnected.

| Field    | Description               | Size    |
|----------|---------------------------|---------|
| Type     | Packet type value = 0x6   | 1 byte  |
| SnakeID  | The snake id              | 1 byte  |


### Connection rejected

The server sends this packet if the server is full.

| Field    | Description               | Size    |
|----------|---------------------------|---------|
| Type     | Packet type value = 0x7   | 1 byte  |
