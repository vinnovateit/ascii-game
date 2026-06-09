import { StartMenuScene } from "./scenes/StartMenuScene.ts";
import { GameplayScene } from "./scenes/GameplayScene.ts";
import { Engine } from "./core/engine.ts";
import { Input } from "./core/Input.ts";

const menu = new StartMenuScene();

// Start the game lifecycle by asking for menu options
menu.askForMenuInput((choice: string) => {
    if (choice === "1") {
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