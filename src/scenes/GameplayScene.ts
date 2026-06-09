import chalk from 'chalk';
import { lightShadeBlock } from "../assets/components.ts";
import { MapGenerator } from "../world/MapGenerator.ts";
import { Input } from "../core/Input.ts";

export class GameplayScene {
    private viewWidth: number;
    private viewHeight: number;
    private worldWidth: number = 200;
    private worldHeight: number = 100;
    private worldMap: string[][];
    private playerWorldX: number;
    private playerWorldY: number;

    constructor(viewWidth: number = 92, viewHeight: number = 36) {
        this.viewWidth = viewWidth;
        this.viewHeight = viewHeight;
        this.playerWorldX = Math.floor(this.worldWidth / 2);
        this.playerWorldY = Math.floor(this.worldHeight / 2);
        this.worldMap = MapGenerator.generate(this.worldWidth, this.worldHeight);
        this.worldMap[this.playerWorldY]![this.playerWorldX] = ' ';
    }

    public update(): void {
        let nextX = this.playerWorldX;
        let nextY = this.playerWorldY;

        if (Input.isPressed('w')) nextY--;
        if (Input.isPressed('s')) nextY++;
        if (Input.isPressed('a')) nextX--;
        if (Input.isPressed('d')) nextX++;

        const destinationTile = this.worldMap[nextY]?.[nextX];
        const isWall = destinationTile && destinationTile !== ' ' && destinationTile !== '/' && destinationTile !== '-' && destinationTile !== '|' && destinationTile !== '.' && destinationTile !== `\\` && destinationTile !== '_' && destinationTile != lightShadeBlock;

        if (this.worldMap[nextY] && !isWall) {
            this.playerWorldX = nextX;
            this.playerWorldY = nextY;
        }

        Input.clearInputFlags();
    }

    public render(): void {
        const halfWidth = Math.floor(this.viewWidth / 2);
        const halfHeight = Math.floor(this.viewHeight / 2);
        const cameraMinX = this.playerWorldX - halfWidth;
        const cameraMinY = this.playerWorldY - halfHeight;

        let frameOutput = '';
        frameOutput += '\x1B[H'; 
        frameOutput += `+${'-'.repeat(this.viewWidth)}+\n`;

        for (let screenY = 0; screenY < this.viewHeight; screenY++) {
            let rowString = '|';
            for (let screenX = 0; screenX < this.viewWidth; screenX++) {
                if (screenY === halfHeight && screenX === halfWidth) {
                    rowString += chalk.green('@');
                    continue;
                }

                const targetWorldX = cameraMinX + screenX;
                const targetWorldY = cameraMinY + screenY;

                if (targetWorldY >= 0 && targetWorldY < this.worldHeight &&
                    targetWorldX >= 0 && targetWorldX < this.worldWidth) {
                    const tile = this.worldMap[targetWorldY]?.[targetWorldX] || ' ';
                    if (tile !== ' ') {
                        rowString += chalk.gray(tile);
                    } else {
                        rowString += ' ';
                    }
                } else {
                    rowString += ' ';
                }
            }
            rowString += '|\n';
            frameOutput += rowString;
        }

        frameOutput += `+${'-'.repeat(this.viewWidth)}+\n`;
        process.stdout.write(frameOutput);
    }
}