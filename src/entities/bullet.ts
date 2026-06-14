let bulletCounter = 0;

export class Bullet {
    id: number
    x: number
    y: number
    dx: number      //direction
    dy: number      
    damage: number
    speed: number
    range: number
    travelled: number  // range traveled 
    firedBy: string
    active: boolean  

    constructor(x: number, y: number, dx: number, dy: number, damage: number, speed: number, range: number, ownerId: string) {
        this.id = bulletCounter++   
        this.x = x
        this.y = y
        this.dx = dx
        this.dy = dy
        this.damage = damage
        this.speed = speed
        this.range = range
        this.travelled = 0         
        this.firedBy = ownerId
        this.active = true
    }


}