import chalk from 'chalk';

export class PlayerStats {
    public health: number = 100;
    public maxHealth: number = 100;
    public score: number = 0;
    public wave: number = 1;

    //changes health bar colour
    private getHealthColor(text: string): string {
        const percentage = (this.health / this.maxHealth) * 100;
        if (percentage > 75) return chalk.green(text);
        if (percentage > 25) return chalk.yellow(text);
        return chalk.red(text);
    }

    //displays the health bar
    private getHealthBar(width: number): string {
        const filledLength = Math.max(0, Math.floor((this.health / this.maxHealth) * width));
        const emptyLength = Math.max(0, width - filledLength);
        
        const bar = '█'.repeat(filledLength) + '░'.repeat(emptyLength);
        return this.getHealthColor(bar);
    }

    //genrates the stats and then returns them in the form |        =survivor =       | or just |            |
    public getStatbarLine(lineIndex: number, boxWidth: number): string {
        let content = '';

        switch (lineIndex) {
            case 1: content = `  === SURVIVOR ===  `; break;
            case 3: content = `  HEALTH: ${this.health}/${this.maxHealth} `; break;
            case 4: content = `  [${this.getHealthBar(12)}] `; break;
            case 6: content = `  SCORE:  ${this.score} `; break;
            case 7: content = `  WAVE:   ${this.wave} `; break;
            default: content = ''; break;
        }

        
        const rawContentLength = content.replace(/\x1B\[[0-9;]*m/g, '').length; 
        const paddingNeeded = Math.max(0, boxWidth - rawContentLength - 2);
        
        return chalk.gray('|') + content + ' '.repeat(paddingNeeded) + chalk.gray('|');
    }
}