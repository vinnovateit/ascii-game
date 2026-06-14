import { createInterface } from "node:readline";
import { io as clientIo, type Socket } from "socket.io-client";
import { CreateGame } from "./CreateGame.ts";
import { LOBBY_SERVER_URL } from "./LobbyServer.ts";

type Player = {
    socketId: string;
    username: string;
};

type RoomEvent = {
    roomCode: string;
    players: Player[];
    maxPlayers: number;
    hostId: string;
    inGame: boolean;
};

export class JoinGame {
    private client: Socket | null = null;
    private username: string;

    constructor(username: string) {
        this.username = username;
    }

    public async start(): Promise<void> {
        const connected = await this.tryConnectToServer();
        if (!connected) {
            console.clear();
            console.log("==================================================");
            console.log("Could not connect to lobby server");
            console.log("==================================================");

            const createInstead = await this.askYesNo("Create a room instead? (y/n): ");
            if (createInstead) {
                const createGame = new CreateGame(this.username);
                await createGame.start();
            }
            return;
        }

        // Check if any rooms exist
        const roomsExist = await this.checkRoomsExist();
        if (!roomsExist) {
            console.clear();
            console.log("==================================================");
            console.log("No rooms available");
            console.log("==================================================");

            const createInstead = await this.askYesNo("Create a room instead? (y/n): ");
            if (createInstead) {
                this.client?.disconnect();
                const createGame = new CreateGame(this.username);
                await createGame.start();
            } else {
                this.client?.disconnect();
            }
            return;
        }

        // Rooms exist, ask for room code
        const roomCode = (await this.askQuestion("Enter room code: ")).toUpperCase();

        return new Promise((resolve) => {
            let joinErrorHandled = false;

            this.client?.on("room_updated", (room: RoomEvent) => {
                console.clear();
                console.log("==================================================");
                console.log("You joined the room!");
                console.log("==================================================");
                this.printRoomState(room);
                console.log("==================================================");
            });

            this.client?.on("join_error", async (message: string) => {
                if (joinErrorHandled) return;
                joinErrorHandled = true;

                console.clear();
                console.log("==================================================");
                console.log(`Join error: ${message}`);
                console.log("==================================================");

                await this.promptReturnToMenu();
                this.client?.disconnect();
                resolve();
            });

            this.client?.on("disconnect", () => {
                if (!joinErrorHandled) {
                    console.clear();
                    console.log("==================================================");
                    console.log("Disconnected from lobby server.");
                    console.log("==================================================");
                }
                resolve();
            });

            this.client?.emit("join_game", {
                roomCode,
                username: this.username
            });
        });
    }

    private checkRoomsExist(): Promise<boolean> {
        return new Promise((resolve) => {
            let responded = false;

            const timeout = setTimeout(() => {
                if (!responded) {
                    responded = true;
                    resolve(false);
                }
            }, 2000);

            this.client?.once("rooms_list", (data: { count: number }) => {
                if (!responded) {
                    responded = true;
                    clearTimeout(timeout);
                    resolve(data.count > 0);
                }
            });

            this.client?.emit("list_rooms");
        });
    }

    private async tryConnectToServer(): Promise<boolean> {
        return new Promise((resolve) => {
            this.client = clientIo(LOBBY_SERVER_URL, {
                transports: ["websocket"],
                reconnectionAttempts: 1,
                timeout: 2000
            });

            const cleanup = () => {
                this.client?.off("connect");
                this.client?.off("connect_error");
                this.client?.off("connect_timeout");
            };

            this.client.on("connect", () => {
                cleanup();
                resolve(true);
            });

            this.client.on("connect_error", () => {
                cleanup();
                resolve(false);
            });

            this.client.on("connect_timeout", () => {
                cleanup();
                resolve(false);
            });
        });
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

    private async promptReturnToMenu(): Promise<void> {
        const returnToMenu = await this.askYesNo("Return to main menu? (y/n): ");
        if (!returnToMenu) {
            console.log("Exiting...");
            process.exit(0);
        }
    }

    private printRoomState(room: RoomEvent): void {
        const host = room.players.find((player) => player.socketId === room.hostId);
        const teammates = room.players.filter((player) => player.socketId !== room.hostId);

        console.log(`Room code: ${room.roomCode}`);
        console.log("Share this code with friends to join your game.");
        console.log("==================================================");
        console.log(`Host: ${host?.username ?? "unknown"}`);
        console.log(`Team: ${[host?.username ?? "unknown", ...teammates.map((player) => player.username)].join(", ")}`);
        console.log(`Players: ${room.players.length}/${room.maxPlayers}`);
        console.log(`Game started: ${room.inGame ? "yes" : "no"}`);
    }
}
