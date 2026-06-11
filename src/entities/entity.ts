export class Entity {
    id: string;
    xPos: number;
    yPos: number;
    hp: number;
    maxHp: number;
    alive: boolean;
    symbol: string;

   
    constructor(x: number, y: number, hp: number, symbol: string){

        this.id = '.' // overide in player and zombie class
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

    // simple distance method 
    
    distanceTo(other: Entity): number {
    return Math.sqrt(
        Math.pow(other.xPos - this.xPos, 2) + Math.pow(other.yPos - this.yPos, 2)
    )
    }


}


