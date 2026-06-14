import { StartMenuScene } from "./scenes/StartMenuScene.ts";
import { GameplayScene } from "./scenes/GameplayScene.ts";
import { CreateGame } from "./lobby/CreateGame.ts";
import { JoinGame } from "./lobby/JoinGame.ts";
import { Engine } from "./core/engine.ts";
import { Input } from "./core/Input.ts";
import { createInterface } from "node:readline";

const askQuestion = (prompt: string): Promise<string> => {
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
};

let globalUsername: string = "";

const showMainMenu = async () => {
    const menu = new StartMenuScene();

    menu.askForMenuInput(async (choice: string) => {
        if (choice === "1") {
            const joinGame = new JoinGame(globalUsername);
            await joinGame.start();
            // Return to main menu after join
            await showMainMenu();
        } else if (choice === "2") {
            const createGame = new CreateGame(globalUsername);
            await createGame.start();
            // Return to main menu after create
            await showMainMenu();
        } else if (choice === "3") {
            console.clear();

            // 1. Completely strip out any lingering data event listeners from Readline
            process.stdin.removeAllListeners('data');

            // 2. Initialize the global raw real-time keypress capture hooks
            Input.initialize();

            // 3. Instantiate the isolated gameplay logic module
            const gameScene = new GameplayScene(92, 36);

            // 4. Spin up the precision engine loop to call updates and renders directly
            const engine = new Engine();
            engine.start(() => {
                gameScene.update();
                gameScene.render();
            });
        }
    });
};

globalUsername = await askQuestion("Enter your name: ");
showMainMenu();
