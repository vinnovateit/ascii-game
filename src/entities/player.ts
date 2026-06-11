import { Entity } from "./entity"

//not decided stats
const ROLE_STATS = {
    MEDIC:    { hp: 100, symbol: "M", ammo: 30 },
    HEAVY:    { hp: 100, symbol: "H", ammo: 30 },
    SCOUT:    { hp: 100, symbol: "S", ammo: 30 },
    DEFENDER: { hp: 100, symbol: "D", ammo: 30 },
}

type Role = "MEDIC" | "HEAVY" | "SCOUT" | "DEFENDER"

let playerCounter = 1
export class Player extends Entity {

    role: Role;
    isDown: boolean;
    ammo: number;
    armour: number;

    constructor(x: number, y: number, role: Role) {
        const stats = ROLE_STATS[role];

        super(x, y, stats.hp, stats.symbol);
        this.id = `P${playerCounter++}`  // P1 P2 P3 id for players
        this.isDown = false;
        this.ammo = stats.ammo;
        this.armour = 0;
        this.role = role;
    }

    // damage will be reduced by armuour points


    takeDamage(amount: number): void {
        const reduced = amount * (1 - this.armour)
        super.takeDamage(reduced)  
    }

    goDown(): void {
        this.isDown = true
        this.hp = 1
        this.alive = true  
    }

}


