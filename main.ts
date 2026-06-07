//starting screen

import chalk from 'chalk';
import * as readline from 'readline';
import { fullBlock } from "./components.ts";
import {LockedFrameGame} from "./game.ts";

class StartMenu {
    private viewHeight: number = 40;
    private viewWidth: number = 120;

    render(): number {
        
        const lines: string[] = [
            "", 
            "",
           "",
            "",
            "=================== TERMINAL ZOMBIE SURVIVAL ===================",
            "",
            "---------------------------- [ ☠ ] ----------------------------",
            "",
            "",
            "[1]  JOIN GAME",
            "",
            "[2]  CREATE LOBBY",
            "",
            "[3]  SELECT ROLE",
            "",
            "[4]  SETTINGS",
            "",
            "[5]  CREDITS",
            "",
            "[6]  HOW TO PLAY",
            "",
            "[7]  QUIT",
            ""
        ];

        let screenBuffer = "";

        
        for (let j = 0; j < this.viewHeight; j++) {
            let lineContent = "";

           
            if (j < lines.length) {
                let  rawLine: string  = lines[j] || "";
            
                const totallength = this.viewWidth - rawLine.length;
                const leftlength = Math.max(0, Math.floor(totallength / 2));
                const rightlength = Math.max(0, totallength - leftlength);

                lineContent = " ".repeat(leftlength) + rawLine + " ".repeat(rightlength);
            } else {
              
                lineContent = " ".repeat(this.viewWidth);
            }

            screenBuffer += lineContent + "\n";
        }

        
        process.stdout.write(chalk.red(screenBuffer));

        return 0;
    }
}

function handleMenuSelection(choice: string): void {
    switch (choice) {
        case "1":
            rl.close();
            console.clear();
            const game = new LockedFrameGame(92, 36);
            game.start();
            break;
        case "2":
            console.log("Setting up a new lobby");
            break;
        case "3":
            console.log("Opening role selection screen");
            break;
        case "4":
            console.log("Opening Settings menu");
            break;
        case "5":
            console.log("Game created by vinnovate boys");
            break;
        case "6":
            console.log("Controls.");
            break;
        case "7":
            console.log("bye");
            process.exit(0);
        default:
            console.log("Invalid option.");
            break;
    }
}

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});


//calls above classes and funcitons
const menu = new StartMenu();


//to stop from game rendering immediately
function askForMenuInput(): void {

    menu.render();


    rl.question('Your ans: ', (answer) => {
        const trimmed = answer.trim();
        if (/^[1-7]$/.test(trimmed)) {
            handleMenuSelection(trimmed);
            if (trimmed !== "1") {
                rl.close();
            }
        } else {
            
            askForMenuInput();
        }
    });
}

askForMenuInput();
