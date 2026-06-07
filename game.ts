import chalk from 'chalk';
import * as readline from 'readline';
import { fullBlock, darkShadeBlock, mediumShadeBlock, lightShadeBlock } from "./components.ts";

export class LockedFrameGame {
    private viewWidth: number;
    private viewHeight: number;

    private worldWidth: number = 200;
    private worldHeight: number = 100;
    private worldMap: string[][] = [];

    //player positons
    private playerWorldX: number;
    private playerWorldY: number;

    private keysPressed: { [key: string]: boolean } = { w: false, a: false, s: false, d: false };

    // locked 30 FPS target
    private readonly TARGET_FPS = 30;
    private readonly FRAME_TIME_NS = BigInt(Math.floor(1_000_000_000 / this.TARGET_FPS)); 
    private lastFrameTime: bigint = process.hrtime.bigint();

    constructor(viewWidth: number = 92, viewHeight: number = 36) {
        this.viewWidth = viewWidth;
        this.viewHeight = viewHeight;

        this.playerWorldX = Math.floor(this.worldWidth / 2);
        this.playerWorldY = Math.floor(this.worldHeight / 2);

        this.generateWorld();
    }

    private generateWorld(): void {
    const mapWidth = this.worldWidth;
    const mapHeight = this.worldHeight;
    
    const centerX = Math.floor(mapWidth / 2);
    const centerY = Math.floor(mapHeight / 2);
    
    const blockWidth = 30;  
    const blockHeight = 20; 
    
    const houseWidth = 14;
    const houseHeight = 8;

    for (let y = 0; y < mapHeight; y++) {
        this.worldMap[y] = [];
        for (let x = 0; x < mapWidth; x++) {
            
            //world border
            if (y === 0 || y === mapHeight - 1 || x === 0 || x === mapWidth - 1) {
                this.worldMap[y][x] = fullBlock;
                continue;
            }

            
            const dx = x - centerX;
const dy = y - centerY;

const distance = Math.sqrt((dx * dx) + (dy * dy * 4.5));

if (distance < 15) {
    
    this.worldMap[y][x] = ' '; 

    // dotted ring
    if (distance >= 12.5 && distance < 13.5) {
        this.worldMap[y][x] = '.'; 
    }
    
    // center spawn
   
    const absX = Math.abs(dx);
    const absY = Math.abs(dy);

    // top and bottom of sapwn 
    if (absY === 2 && absX <= 2) {
        this.worldMap[y][x] = '-';
    }
    // legt and right parts of spawn
    else if (absX === 5 && absY <= 0) {
        this.worldMap[y][x] = '|';
    }
    // making it a circular thing
    else if (absX === 4 && absY === 1) {
        // tf n br diagonal
        if ((dx < 0 && dy < 0) || (dx > 0 && dy > 0)) this.worldMap[y][x] = '/';
        // tr n bl diagonal
        if ((dx > 0 && dy < 0) || (dx < 0 && dy > 0)) this.worldMap[y][x] = '\\';
    }
    else if (absX === 3 && absY === 2) {
       
        if ((dx < 0 && dy < 0) || (dx > 0 && dy > 0)) this.worldMap[y][x] = '/';
        if ((dx > 0 && dy < 0) || (dx < 0 && dy > 0)) this.worldMap[y][x] = '\\';
    }

    // center of spawn
    if (dx === 0 && dy === 0) {
        this.worldMap[y][x] = 'o';
    }

    continue; 
}

            // house generation 
            const localX = x % blockWidth;
            const localY = y % blockHeight;

            const houseStartX = Math.floor((blockWidth - houseWidth) / 2);
            const houseStartY = Math.floor((blockHeight - houseHeight) / 2);
            const houseEndX = houseStartX + houseWidth - 1;
            const houseEndY = houseStartY + houseHeight - 1;

            if (localX >= houseStartX && localX <= houseEndX && localY >= houseStartY && localY <= houseEndY) {
                const isHorizontalWall = localY === houseStartY || localY === houseEndY;
                const isVerticalWall = localX === houseStartX || localX === houseEndX;

                if (isHorizontalWall || isVerticalWall) {
                    const midX = houseStartX + Math.floor(houseWidth / 2);
                    const midY = houseStartY + Math.floor(houseHeight / 2);

                    const isNorthDoor = localY === houseStartY && (localX === midX || localX === midX - 1);
                    const isSouthDoor = localY === houseEndY && (localX === midX || localX === midX - 1);
                    const isWestDoor = localX === houseStartX && localY === midY;
                    const isEastDoor = localX === houseEndX && localY === midY;

                    if (isNorthDoor || isSouthDoor || isWestDoor || isEastDoor) {
                        this.worldMap[y][x] = '/'; 
                    } else {
                        this.worldMap[y][x] = darkShadeBlock; 
                    }
                } else {
                    //floor
                    this.worldMap[y][x] = ' '; 
                }
            } else {
                //space around house
                this.worldMap[y][x] = ' '; 
            }
        }
    }

    //makes player spawn on empty tile
    this.worldMap[this.playerWorldY][this.playerWorldX] = ' ';
}

    private updatePhysics(): void {
        let nextX = this.playerWorldX;
        let nextY = this.playerWorldY;

        if (this.keysPressed.w) nextY--;
        if (this.keysPressed.s) nextY++;
        if (this.keysPressed.a) nextX--;
        if (this.keysPressed.d) nextX++;

        // Stop player if they walk into any wall 
        const destinationTile = this.worldMap[nextY]?.[nextX];
        //this lets the player walk through decorative stuff like \|. etc
        const isWall = destinationTile && destinationTile !== ' '&& destinationTile !== '/'&& destinationTile !== '-'&& destinationTile !== '|'&& destinationTile !== '.'&& destinationTile !== `\\` && destinationTile !== '_' && destinationTile != lightShadeBlock;

        if (this.worldMap[nextY] && !isWall) {
            this.playerWorldX = nextX;
            this.playerWorldY = nextY;
        }
    }

    //basic movement to see if rendierng works
    private handleKeyboardInput = (data: Buffer): void => {
        const keyString = data.toString();
        if (keyString === '\u0003') process.exit(); 

        const key = keyString.toLowerCase();
        if (key.includes('w')) this.keysPressed.w = true;
        if (key.includes('s')) this.keysPressed.s = true;
        if (key.includes('a')) this.keysPressed.a = true;
        if (key.includes('d')) this.keysPressed.d = true;
    };
    
    //renders map
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
                    const tile = this.worldMap[targetWorldY][targetWorldX];
                    
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

    private gameLoop = (): void => {
        const now = process.hrtime.bigint();
        const elapsedTime = now - this.lastFrameTime;

        if (elapsedTime >= this.FRAME_TIME_NS) {
            this.updatePhysics();
            this.render();

            this.keysPressed.w = false;
            this.keysPressed.s = false;
            this.keysPressed.a = false;
            this.keysPressed.d = false;

            this.lastFrameTime = now - (elapsedTime % this.FRAME_TIME_NS);
        }

        setImmediate(this.gameLoop);
    };

    public start(): void {
        console.clear();

        readline.emitKeypressEvents(process.stdin);
        if (process.stdin.isTTY) {
            process.stdin.setRawMode(true);
        }

        process.stdin.resume();
        process.stdin.on('data', this.handleKeyboardInput);

        this.lastFrameTime = process.hrtime.bigint();
        setImmediate(this.gameLoop);
    }
}