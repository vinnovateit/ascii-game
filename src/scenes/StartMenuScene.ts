import figlet from "figlet";
import chalk from "chalk";
import * as readline from "readline";

interface MenuLine {
    text: string;
    font?: import("figlet").FontName;
}

export class StartMenuScene {
    private viewHeight: number = 40;
    private viewWidth: number = 120;
    private rl!: readline.Interface;

    public render(): void {
        const lines: MenuLine[] = [
            { text: "" },
            { text: "" },
            { text: "" },
            { text: "ZOMBIE GAME", font: "Graffiti" },
            
            { text: "" },
            { text: "----------------------------[ ☠ ]----------------------------" },
            { text: "" },
            
            { text: "[1]  JOIN GAME" },
            { text: "" },
            { text: "[2]  CREATE LOBBY" },
            { text: "" },
            { text: "[3]  START SINGLEPLAYER" },
            { text: "" },
            { text: "[4]  SETTINGS" },
            { text: "" },
            { text: "[5]  CREDITS" },
            { text: "" },
            { text: "[6]  HOW TO PLAY" },
            { text: "" },
            { text: "[7]  QUIT" },
            { text: "" }
        ];

        const renderedLines = this.renderMenuLines(lines);
        const screenBuffer = this.centerRenderedLines(renderedLines);

        process.stdout.write(chalk.red(screenBuffer));
    }

    private renderMenuLines(lines: MenuLine[]): string[] {
        const renderedLines: string[] = [];

        for (const line of lines) {
            renderedLines.push(...this.renderLine(line));
        }

        return renderedLines;
    }

    private renderLine(line: MenuLine): string[] {
        try {
            if (line.font) {
                const figletOptions = {
                    width: this.viewWidth,
                    whitespaceBreak: true,
                    font: line.font,
                };
                const figletOutput = figlet.textSync(line.text, figletOptions);
                return figletOutput.split("\n");
            } 
            
            return [line.text];
            
        } catch {
            return [line.text];
        }
    }

    private centerRenderedLines(lines: string[]): string {
        let screenBuffer = "";

        for (let row = 0; row < this.viewHeight; row++) {
            const rawText = lines[row] ?? "";
            const leftPadding = Math.max(0, Math.floor((this.viewWidth - rawText.length) / 2));
            const rightPadding = Math.max(0, this.viewWidth - rawText.length - leftPadding);
            screenBuffer += `${" ".repeat(leftPadding)}${rawText}${" ".repeat(rightPadding)}\n`;
        }

        return screenBuffer;
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
                
                process.stdout.write('\x1b[1A\x1b[2K');
                this.promptUser(onSelect);
            }
        });
    }
}