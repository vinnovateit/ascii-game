import { createServer, type Server as HttpServer } from "node:http";
import { Server, type Socket } from "socket.io";

export type Player = {
    socketId: string;
    username: string;
};

export type Room = {
    code: string;
    players: Player[];
    maxPlayers: number;
    hostId: string;
    inGame: boolean;
};

type CreateGamePayload = {
    username?: string;
    roomCode?: string;
};

type JoinGamePayload = {
    roomCode: string;
    username?: string;
};

type SetInGamePayload = {
    roomCode: string;
    inGame: boolean;
};

type RoomEvent = {
    roomCode: string;
    players: Player[];
    maxPlayers: number;
    hostId: string;
    inGame: boolean;
};

const PORT = 3000;
const ROOM_CODE_LENGTH = 4;
const MAX_PLAYERS = 5;
const ROOM_CODE_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

export const LOBBY_SERVER_URL = `http://localhost:${PORT}`;

export class LobbyServer {
    private static httpServer: HttpServer | null = null;
    private static io: Server | null = null;
    private static rooms: Map<string, Room> = new Map();

    public static start(): void {
        if (LobbyServer.io) {
            return;
        }

        LobbyServer.httpServer = createServer();
        LobbyServer.io = new Server(LobbyServer.httpServer, {
            cors: {
                origin: "*"
            }
        });

        LobbyServer.io.on("connection", (socket: Socket) => {
            LobbyServer.registerSocketHandlers(socket);
        });

        LobbyServer.httpServer.listen(PORT, () => {
            console.log(`Lobby server running on ${LOBBY_SERVER_URL}`);
        });
    }

    private static registerSocketHandlers(socket: Socket): void {
        socket.on("list_rooms", () => {
            const roomCount = LobbyServer.rooms.size;
            socket.emit("rooms_list", {
                count: roomCount,
                rooms: Array.from(LobbyServer.rooms.values()).map((room) => ({
                    code: room.code,
                    players: room.players.length,
                    maxPlayers: room.maxPlayers,
                    inGame: room.inGame
                }))
            });
        });

        socket.on("create_game", (payload: CreateGamePayload = {}) => {
            LobbyServer.removePlayerFromExistingRooms(socket.id);

            const roomCode = LobbyServer.generateRoomCode();
            const room: Room = {
                code: roomCode,
                players: [
                    {
                        socketId: socket.id,
                        username: payload.username || "Player"
                    }
                ],
                maxPlayers: MAX_PLAYERS,
                hostId: socket.id,
                inGame: false
            };

            LobbyServer.rooms.set(roomCode, room);
            socket.join(roomCode);

            socket.emit("game_created", {
                roomCode,
                players: room.players,
                maxPlayers: room.maxPlayers,
                hostId: room.hostId,
                inGame: room.inGame
            });
        });

        socket.on("join_game", (payload: JoinGamePayload) => {
            const room = LobbyServer.rooms.get(payload.roomCode);
            if (!room) {
                socket.emit("join_error", "Room not found.");
                return;
            }

            if (room.inGame) {
                socket.emit("join_error", "Game already started.");
                return;
            }

            if (room.players.length >= room.maxPlayers) {
                socket.emit("join_error", "Room is full.");
                return;
            }

            if (room.players.some((player) => player.socketId === socket.id)) {
                socket.emit("join_error", "Already joined.");
                return;
            }

            const player: Player = {
                socketId: socket.id,
                username: payload.username || "Player"
            };

            room.players.push(player);
            socket.join(room.code);

            LobbyServer.io?.to(room.code).emit("room_updated", {
                roomCode: room.code,
                players: room.players,
                maxPlayers: room.maxPlayers,
                hostId: room.hostId,
                inGame: room.inGame
            });
        });

        socket.on("set_in_game", (payload: SetInGamePayload) => {
            const room = LobbyServer.rooms.get(payload.roomCode);
            if (!room) {
                socket.emit("join_error", "Room not found.");
                return;
            }

            if (socket.id !== room.hostId) {
                socket.emit("join_error", "Only host can change game state.");
                return;
            }

            room.inGame = payload.inGame;
            LobbyServer.io?.to(room.code).emit("room_updated", {
                roomCode: room.code,
                players: room.players,
                maxPlayers: room.maxPlayers,
                hostId: room.hostId,
                inGame: room.inGame
            });
        });

        socket.on("disconnect", () => {
            LobbyServer.removePlayerFromExistingRooms(socket.id);
        });
    }

    private static generateRoomCode(): string {
        let code = "";

        do {
            code = "";
            for (let i = 0; i < ROOM_CODE_LENGTH; i++) {
                const randomIndex = Math.floor(Math.random() * ROOM_CODE_CHARS.length);
                code += ROOM_CODE_CHARS[randomIndex];
            }
        } while (LobbyServer.rooms.has(code));

        return code;
    }

    private static removePlayerFromExistingRooms(socketId: string): void {
        for (const [roomCode, room] of LobbyServer.rooms) {
            const playerIndex = room.players.findIndex((player) => player.socketId === socketId);
            if (playerIndex === -1) {
                continue;
            }

            const removedPlayer = room.players.splice(playerIndex, 1)[0];
            const isHost = room.hostId === socketId;

            if (room.players.length === 0) {
                LobbyServer.rooms.delete(roomCode);
                continue;
            }

            if (isHost) {
                const newHost = room.players[0];
                if (newHost) {
                    room.hostId = newHost.socketId;
                    LobbyServer.io?.to(room.code).emit("host_changed", {
                        roomCode: room.code,
                        players: room.players,
                        maxPlayers: room.maxPlayers,
                        hostId: room.hostId,
                        inGame: room.inGame
                    });
                }
            }

            LobbyServer.io?.to(room.code).emit("room_updated", {
                roomCode: room.code,
                players: room.players,
                maxPlayers: room.maxPlayers,
                hostId: room.hostId,
                inGame: room.inGame
            });
        }
    }
}

import { fileURLToPath } from "node:url";

const isMainModule = process.argv[1] === fileURLToPath(import.meta.url);
if (isMainModule) {
    LobbyServer.start();
}
