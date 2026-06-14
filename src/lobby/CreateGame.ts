import { createInterface } from "node:readline";
import { io as clientIo, type Socket as ClientSocket } from "socket.io-client";
import { LOBBY_SERVER_URL } from "./LobbyServer.ts";

type Player = {
    socketId: string;
    username: string;
};

type Room = {
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

export class CreateGame {
    private hostSocket: ClientSocket | null = null;
    private username: string;

    constructor(username: string) {
        this.username = username;
    }

    public async start(): Promise<void> {
        return new Promise((resolve) => {
            this.startInternal(resolve);
        });
    }

    private async startInternal(resolve: () => void): Promise<void> {
        const startGame = await this.askYesNo("Start game now when ready? (y/n): ");

        this.hostSocket = clientIo(LOBBY_SERVER_URL, {
            transports: ["websocket"]
        });

        this.hostSocket.on("connect", () => {
            this.hostSocket?.emit("create_game", { username: this.username });
        });

        this.hostSocket.on("game_created", (room: RoomEvent) => {
            console.clear();
            console.log("==================================================");
            console.log("Lobby server is live.");
            console.log("==================================================");
            console.log(`Room code: ${room.roomCode}`);
            console.log("Share this code with friends to join your game.");
            console.log("==================================================");
            this.printRoomState(room);
            console.log("==================================================");

            if (startGame) {
                this.setRoomInGame(room.roomCode, true);
            }
        });

        this.hostSocket.on("room_updated", (room: RoomEvent) => {
            console.clear();
            console.log("==================================================");
            console.log("Room Updated - New player joined!");
            console.log("==================================================");
            this.printRoomState(room);
            console.log("==================================================");

            if (room.inGame) {
                return;
            }

            this.askYesNo("Start game now? (y/n): ").then((shouldStart) => {
                if (shouldStart) {
                    this.setRoomInGame(room.roomCode, true);
                }
            });
        });

        this.hostSocket.on("host_changed", (room) => {
            console.log(`New host is now ${room.players.find((p: Player) => p.socketId === room.hostId)?.username || "unknown"}`);
        });

        this.hostSocket.on("join_error", (message: string) => {
            console.log(`Join error: ${message}`);
        });

        this.hostSocket.on("disconnect", async () => {
            console.clear();
            console.log("==================================================");
            console.log("Host socket disconnected.");
            console.log("==================================================");
            await this.promptReturnToMenu();
            resolve();
        });
    }

    private async promptReturnToMenu(): Promise<void> {
        const returnToMenu = await this.askYesNo("Return to main menu? (y/n): ");
        if (!returnToMenu) {
            console.log("Exiting...");
            process.exit(0);
        }
    }

    private askQuestion(prompt: string): Promise<string> {
        const rl = createInterface({
            input: process.stdin,
            output: process.stdout
        });

        return new Promise((resolve) => {
            rl.question(prompt, (answer) => {
                rl.close();
                resolve(answer.trim());
            });
        });
    }

    private async askYesNo(prompt: string): Promise<boolean> {
        const answer = (await this.askQuestion(prompt)).toLowerCase();
        return answer === "y" || answer === "yes";
    }

    private setRoomInGame(roomCode: string, inGame: boolean): void {
        if (!this.hostSocket) {
            return;
        }

        this.hostSocket.emit("set_in_game", {
            roomCode,
            inGame
        });
    }

    private printRoomState(room: RoomEvent): void {
        const host = room.players.find((player) => player.socketId === room.hostId);
        const teammates = room.players.filter((player) => player.socketId !== room.hostId);

        console.log(`Room code: ${room.roomCode}`);
        console.log(`Host: ${host?.username ?? "unknown"}`);
        console.log(`Team: ${[host?.username ?? "unknown", ...teammates.map((player) => player.username)].join(", ")}`);
        console.log(`Players: ${room.players.length}/${room.maxPlayers}`);
        console.log(`Game started: ${room.inGame ? "yes" : "no"}`);
    }
}
