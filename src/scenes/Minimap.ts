import chalk from 'chalk';
import { fullBlock, darkShadeBlock } from "../assets/components.ts";

export class Minimap_initiliser {
    
    public getMinimap(
        lineIndex: number, 
        boxWidth: number, 
        totalMinimapHeight: number, 
        worldMap: string[][], 
        playerX: number, 
        playerY: number ): string {
        const playableWidth = boxWidth ; 

        
        const zoomScaleY = 3; 
        const zoomScaleX = 4;

        const halfMiniWidth = Math.floor(playableWidth / 2);
        const halfMiniHeight = Math.floor(totalMinimapHeight / 2);

        let rowString = chalk.blue('|'); 

        
        const cameraCenterX = playerX;
        const cameraCenterY = playerY;

        for (let miniX = 0; miniX < playableWidth; miniX++) {
            
            //player at center of minimmap
            if (miniX === halfMiniWidth && lineIndex === halfMiniHeight) {
                rowString += chalk.green('@');
                continue;
            }

            const targetWorldX = Math.floor(cameraCenterX + (miniX - halfMiniWidth) * zoomScaleX);
            const targetWorldY = Math.floor(cameraCenterY + (lineIndex - halfMiniHeight) * zoomScaleY);

            const startY = targetWorldY;
            const endY = targetWorldY + zoomScaleY;
            const startX = targetWorldX;
            const endX = targetWorldX + zoomScaleX;

            let hasSolidWall = false;
            let hasHouseWall = false;
            let hasFountain = false;
            let isOutOfBounds = false;

            for (let y = startY; y < endY; y++) {
                if (y < 0 || y >= worldMap.length) {
                    isOutOfBounds = true;
                    continue;
                }
                
                for (let x = startX; x < endX; x++) {
                    if (x < 0 || x >= (worldMap[y]?.length ?? 0)) {
                        isOutOfBounds = true;
                        continue;
                    }

                    const tile = worldMap[y]?.[x];
                    if (tile === fullBlock) {
                        hasSolidWall = true;
                    } else if (tile === darkShadeBlock) {
                        hasHouseWall = true;
                    } else if (tile && tile !== ' ' && tile !== '/') {
                        hasFountain = true;
                    }
                }
            }

            // Render matching tiles based on scan
            if (isOutOfBounds) {
                rowString += ' '; 
            } else if (hasSolidWall) {
                rowString += chalk.blue('█'); 
            } else if (hasHouseWall) {
                rowString += chalk.gray('#'); 
            } else if (hasFountain) {
                rowString += chalk.cyan('·'); 
            } else {
                rowString += ' '; 
            }
        }

        rowString += chalk.blue('|'); 
        return rowString;
    }
}