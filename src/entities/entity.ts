export class Entity {
    id: string;
    xPos: number;
    yPos: number;
    hp: number;
    maxHp: number;
    alive: boolean;
    symbol: string;

    // id randomnly gen 
    constructor(x: number, y: number, hp: number, symbol: string){
        this.id = Math.random().toString(36).slice(2)
        this.xPos = x;
        this.yPos = y;
        this.hp = hp;
        this.maxHp = hp;
        this.alive = true;
        this.symbol = symbol;
    }

    takeDamage(amount: number): void {
        this.hp -= amount
        if (this.hp <= 0) {
            this.hp = 0;
            this.alive = false;
        }
    }

    heal(amount: number): void {
        this.hp = Math.min(this.hp + amount, this.maxHp)  
    }

    isAlive(): boolean {
        return this.alive;
    }

    // data 
    getData() {
    return {
        id: this.id,
        xPos: this.xPos,
        yPos: this.yPos,
        hp: this.hp,
        alive: this.alive,
        symbol: this.symbol
    }
}
}


