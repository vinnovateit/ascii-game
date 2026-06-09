import chalk from 'chalk';
import * as readline from 'readline';

export class StartMenuScene {
    private viewHeight: number = 40;
    private viewWidth: number = 120;
    private rl!: readline.Interface;

    public render(): void {
        const lines: string[] = [
            "", "", "", "",
            "=================== TERMINAL ZOMBIE SURVIVAL ===================",
            "",
            "---------------------------- [ ☠ ] ----------------------------",
            "", "",
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
                let rawLine: string = lines[j] || "";
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
    }

    public askForMenuInput(onSelect: (choice: string) => void): void {
        this.render();

        this.rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout
        });

        this.promptUser(onSelect);
    }

    private promptUser(onSelect: (choice: string) => void): void {
        this.rl.question('Your ans: ', (answer) => {
            const trimmed = answer.trim();
            if (/^[1-7]$/.test(trimmed)) {
                this.rl.close();
                onSelect(trimmed);
            } else {
                this.promptUser(onSelect);
            }
        });
    }
}