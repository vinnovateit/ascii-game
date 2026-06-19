import chalk from 'chalk';
import { lightShadeBlock } from "../assets/components.ts";
import { MapGenerator } from "../world/MapGenerator.ts";
import { Input } from "../core/Input.ts";
import { PlayerStats } from "./playerStats.ts";
import { Status } from './StatusBar.ts';
import { Minimap_initiliser } from './Minimap.ts';
import { dialogue } from './dialogue.ts';
import { Chat } from './chat.ts';

export class GameplayScene {
    private viewWidth: number;
    private viewHeight: number;
    private worldWidth: number = 200;
    private worldHeight: number = 100;
    private worldMap: string[][];
    //player position
    private playerWorldX: number;
    private playerWorldY: number;

    //player stats
    public stats: PlayerStats;
    private sidebarWidth: number = 44;
    private sidebarHeight : number; 

    //top bar dimensions
    public statusBar :Status;
    private statusbarWidth : number ;
    private statusbarHeight: number = 3;

    //minimap dimensions
    public Minimap: Minimap_initiliser;
    private MinimapWidth : number;
    private MinimapHeight: number;

    //dialogue box dimensions
    public dialogueBox: dialogue;
    private dialogueWidth : number;
    private dialogueHeight: number;



    //chatbox dimensions
    public chatBox: Chat;
    private chatBoxWidth : number;
    private chatBoxHeight: number;




    constructor(viewWidth: number , viewHeight: number) {
        this.viewWidth = viewWidth;
        this.viewHeight = viewHeight;
        this.playerWorldX = Math.floor(this.worldWidth / 2);
        this.playerWorldY = Math.floor(this.worldHeight / 2);

        this.worldMap = MapGenerator.generate(this.worldWidth, this.worldHeight);
        this.worldMap[this.playerWorldY]![this.playerWorldX] = ' ';

        //ini stats
        this.stats = new PlayerStats(viewHeight);
        this.sidebarHeight = this.viewHeight/2;
        //ini statusBar
        this.statusBar = new Status();
        this.statusbarWidth = this.sidebarWidth*2 + this.viewWidth + 6;

        //ini minimap
        this.Minimap = new Minimap_initiliser();
        this.MinimapHeight = this.viewHeight/2;
        this.MinimapWidth =this.sidebarWidth;
        
        //ini dialogue box
        this.dialogueBox = new dialogue(viewHeight);
        this.dialogueWidth = this.sidebarWidth;
        this.dialogueHeight = this.viewHeight/2;


        //ini chat box
        this.chatBox = new Chat();
        this.chatBoxWidth = this.sidebarWidth;
        this.chatBoxHeight = this.viewHeight/2;

    }

    public update(): void {
        let nextX = this.playerWorldX;
        let nextY = this.playerWorldY;

        if (Input.isPressed('w')) nextY--;
        if (Input.isPressed('s')) nextY++;
        if (Input.isPressed('a')) nextX--;
        if (Input.isPressed('d')) nextX++;

        //please delete this line just used to test whether health bar colour changes
        if(Input.isPressed('l')&& this.stats.health >0) this.stats.health -=1;

        const destinationTile = this.worldMap[nextY]?.[nextX];
        //to allow player to pass through these objects
        const isWall = destinationTile && destinationTile !== ' ' && destinationTile !== '/' && destinationTile !== '-' && destinationTile !== '|' && destinationTile !== '.' && destinationTile !== `\\` && destinationTile !== '_' && destinationTile != lightShadeBlock;

        if (this.worldMap[nextY] && !isWall) {
            this.playerWorldX = nextX;
            this.playerWorldY = nextY;
        }

        Input.clearInputFlags();
    }


    //renders the entire game
    public render(): void {
        const halfWidth = Math.floor(this.viewWidth / 2);
        const halfHeight = Math.floor(this.viewHeight / 2);
        const cameraMinX = this.playerWorldX - halfWidth;
        const cameraMinY = this.playerWorldY - halfHeight;

        let frameOutput = '';
        frameOutput += '\x1B[H'; 

        const statusbarTopBorder = `+${'-'.repeat(this.statusbarWidth)}+`;
        frameOutput += chalk.blue(statusbarTopBorder) + "\n";
        for (let screenY = 0; screenY < this.statusbarHeight; screenY++){
            let statusBarRow = '';
            
            statusBarRow = this.statusBar.getStatusBar(screenY,this.statusbarWidth);
            
            frameOutput += statusBarRow + "\n";
        }
        const statusBarBottom = `+${'-'.repeat(this.statusbarWidth)}+`;
        frameOutput += chalk.blue(statusBarBottom) + "\n";
        
        




        const mapTopBorder = `+${'-'.repeat(this.viewWidth)}+`;
        const sidebarTopBorder = `+${'-'.repeat(this.sidebarWidth )}+`;
        const dialogueBarTopBorder = `+${'-'.repeat(this.dialogueWidth )}+`;
        frameOutput += sidebarTopBorder + " " + mapTopBorder +" "+ dialogueBarTopBorder +  "\n";

        for (let screenY = 0; screenY < this.viewHeight; screenY++) {
            let rowString = '|';

            let statBarRow = '';
            //statbar loading
            if(screenY < this.sidebarHeight){
            statBarRow = this.stats.getStatbarLine(screenY, this.sidebarWidth);
            }
            //minimap
            else{
                statBarRow = this.Minimap.getMinimap(
                    screenY - this.sidebarHeight, 
                    this.MinimapWidth, 
                    this.MinimapHeight, 
                    this.worldMap, 
                    this.playerWorldX, 
                    this.playerWorldY
                );
            }

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
            rowString += '|';

            let chatboxRow = '';
            //Dialogue box and chat box loading bottom of dialouge box and top of chat box is in getdialougeBox
            if(screenY <this.dialogueHeight)
                chatboxRow = this.dialogueBox.getdialogueBox(screenY, this.dialogueWidth);
            else{
                chatboxRow = this.chatBox.getchatbox(screenY-this.dialogueHeight,this.chatBoxWidth);

            }
           
            

            
            frameOutput += statBarRow + " " + rowString+" " + chatboxRow+ "\n";
        }

        //bottom of main map
        const mapBottomBorder = `+${'-'.repeat(this.viewWidth)}+`;
        const Minimapbottomborder = `+${'-'.repeat(this.MinimapWidth )}+`;
        const chatBoxbottomborder = `+${'-'.repeat(this.chatBoxWidth )}+`;
        frameOutput += Minimapbottomborder+ " "+ mapBottomBorder + " "+ chatBoxbottomborder+ "\n";

        process.stdout.write(frameOutput);
    }
}
