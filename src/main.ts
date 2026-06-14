import { StartMenuScene } from "./scenes/StartMenuScene.ts";
import { GameplayScene } from "./scenes/GameplayScene.ts";
import { Engine } from "./core/engine.ts";
import { Input } from "./core/Input.ts";

const menu = new StartMenuScene();


menu.askForMenuInput((choice: string) => {
    if (choice === "1") {
        console.clear();

        process.stdin.removeAllListeners('data');
        Input.initialize();
        const gameScene = new GameplayScene(92, 36);
        const engine = new Engine();
        engine.start(() => {
            gameScene.update();
            gameScene.render();
        });
    }
});