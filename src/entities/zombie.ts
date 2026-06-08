import { Entity } from "./entity";

const TYPE_STATS = {
    NORMAL: { hp: 100, symbol: "Z", damage: 10, speed: 1 },
    FAST : { hp: 60,  symbol: "F", damage: 8,  speed: 2 },
    TANK : { hp: 300, symbol: "T", damage: 20, speed: 1 },
    SPITTER : { hp: 70,  symbol: "S", damage: 15, speed: 1 },
    EXPLODER : { hp: 50,  symbol: "X", damage: 60, speed: 1 }
}

type ZombieType = "NORMAL" | "FAST" | "SPITTER" | "TANK" | "EXPLODER"

export class Zombie extends Entity {
    zombieType: ZombieType
    damage: number
    speed: number

    constructor(x: number, y: number, zombieType: ZombieType) {
        const stats = TYPE_STATS[zombieType]
        super(x, y, stats.hp, stats.symbol)
        this.zombieType = zombieType
        this.damage = stats.damage
        this.speed = stats.speed
    }
}


const z = new Zombie(10, 10, "FAST")
console.log("type:", z.zombieType)
console.log("hp:", z.hp)
console.log("damage:", z.damage)
console.log("speed:", z.speed)

z.takeDamage(30)
console.log("hp after damage (expect 30):", z.hp)