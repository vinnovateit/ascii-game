import { fullBlock, darkShadeBlock } from "../assets/components.ts";

export class MapGenerator {
    public static generate(worldWidth: number, worldHeight: number): string[][] {
        const worldMap: string[][] = [];
        const centerX = Math.floor(worldWidth / 2);
        const centerY = Math.floor(worldHeight / 2);
        
        const blockWidth = 30;  
        const blockHeight = 20; 
        
        const houseWidth = 14;
        const houseHeight = 8;

        for (let y = 0; y < worldHeight; y++) {
    const row: string[] = []; // Create a temporary local array row
    
    for (let x = 0; x < worldWidth; x++) {
        let tile = ' '; // Default empty tile space

        //world border
        if (y === 0 || y === worldHeight - 1 || x === 0 || x === worldWidth - 1) {
            tile = fullBlock;
        } else {
            const dx = x - centerX;
            const dy = y - centerY;
            const distance = Math.sqrt((dx * dx) + (dy * dy * 4.5));

            if (distance < 15) {
                tile = ' ';
                if (distance >= 12.5 && distance < 13.5) tile = '.';
                
                const absX = Math.abs(dx);
                const absY = Math.abs(dy);

                if (absY === 2 && absX <= 2) tile = '-';
                else if (absX === 5 && absY <= 0) tile = '|';
                else if (absX === 4 && absY === 1) {
                    if ((dx < 0 && dy < 0) || (dx > 0 && dy > 0)) tile = '/';
                    if ((dx > 0 && dy < 0) || (dx < 0 && dy > 0)) tile = '\\';
                }
                else if (absX === 3 && absY === 2) {
                    if ((dx < 0 && dy < 0) || (dx > 0 && dy > 0)) tile = '/';
                    if ((dx > 0 && dy < 0) || (dx < 0 && dy > 0)) tile = '\\';
                }

                if (dx === 0 && dy === 0) tile = 'o';
            } else {
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
                            tile = '/';
                        } else {
                            tile = darkShadeBlock;
                        }
                    }
                }
            }
        }
        
        row.push(tile); // Cleanly add the string character onto the row array
    }

    worldMap.push(row); // Save the complete row safely into the map grid
}

return worldMap;
    }
}