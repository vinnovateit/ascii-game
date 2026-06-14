import { Zombie } from "../entities/zombie"
import { Player } from "../entities/player"


export function findNearestPlayer(zombie: Zombie, players: Player[]): Player | null {

    let nearest: Player | null = null    
    let shortestDistance = Infinity      

    for (const player of players) {     
        
        if (!player.alive) continue      

        const distance = zombie.distanceTo(player)   

        if (distance < shortestDistance) {
            shortestDistance = distance  
            nearest = player             
        }
    }
    return nearest
}

export function moveToward(zombie: Zombie, target: Player): void {

    let dx = target.xPos -zombie.xPos 
    let dy = target.yPos - zombie.yPos

    // movie zombie in whicver is short

    dx > dy ? zombie.xPos++ : zombie.yPos++

    if (Math.abs(dx) > Math.abs(dy)){
        zombie.xPos += dx > 0 ? 1 : -1 
    }
    else {
        zombie.yPos += dy > 0 ? 1 : -1 
    }

}



