import chalk from 'chalk';
import { wave_number, score, zombie_count, max_zombie_count } from '../assets/GameRules.ts';

export class Status {
    
    public getStatusBar(lineIndex: number, boxWidth: number): string {
        let content = '';

        switch (lineIndex) {
            case 0:
            content = chalk.bgRed(chalk.white('zombie game'));                break;
            case 2:
                content = `Wave ${wave_number}      Zombies: ${zombie_count}/${max_zombie_count}        Score: ${score}`;
                break;
            default:
                content = '';
                break;
        }

        const rawContentLength = content.replace(/\x1B\[[0-9;]*m/g, '').length; 
        const totalPaddingNeeded = Math.max(0, boxWidth - rawContentLength);

        const paddingLeft = Math.floor(totalPaddingNeeded / 2);
        const paddingRight = totalPaddingNeeded - paddingLeft;
        
        return chalk.blue('|') + ' '.repeat(paddingLeft) + content + ' '.repeat(paddingRight) + chalk.blue('|'); 
    }
}