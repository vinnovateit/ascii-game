import * as readline from 'readline';

//added l to test healthbar changes
export class Input {
    private static keysPressed: { [key: string]: boolean } = { w: false, a: false, s: false, d: false, l: false };

    public static initialize(): void {
        readline.emitKeypressEvents(process.stdin);
        if (process.stdin.isTTY) {
            process.stdin.setRawMode(true);
        }
        process.stdin.resume();
        
        
        process.stdin.on('data', this.handleKeyboardInput);
    }
    //basic movement to see if rendering works

    private static handleKeyboardInput = (data: Buffer): void => {
        const keyString = data.toString();
        if (keyString === '\u0003') process.exit(); 

        const key = keyString.toLowerCase();
        if (key.includes('w')) this.keysPressed.w = true;
        if (key.includes('s')) this.keysPressed.s = true;
        if (key.includes('a')) this.keysPressed.a = true;
        if (key.includes('d')) this.keysPressed.d = true;
        if(key.includes('l')) this.keysPressed.l = true;
    };

    public static isPressed(key: string): boolean {
        return !!this.keysPressed[key.toLowerCase()];
    }

    public static clearInputFlags(): void {
        this.keysPressed.w = false;
        this.keysPressed.s = false;
        this.keysPressed.a = false;
        this.keysPressed.d = false;
        this.keysPressed.l =false;
    }
}