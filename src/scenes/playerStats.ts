import chalk from 'chalk';

export class PlayerStats {
    public health: number = 100;
    public maxHealth: number = 100;
    private height: number ;

    constructor(height:number)
    {
        this.height = height/2;

    }



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
            
            case 0: content = `  === SURVIVORS ===  `; break;
            
            case 3: content =  chalk.red('   [H]     ') + `[${this.getHealthBar(12)}] `; break;
            case 5: content =   chalk.green('   [M]     ') + `[${this.getHealthBar(12)}] `; break;
            case 7: content =   chalk.blue('   [D]     ') + `[${this.getHealthBar(12)}] `; break;
            case 9: content =   chalk.yellow('   [S]     ') + `[${this.getHealthBar(12)}] `; break;
            //to seperate the player stat and minimap
            case this.height-2: return content = `+${'-'.repeat(boxWidth)}+`;break;
            case this.height-1: return content = `+${'-'.repeat(boxWidth)}+`;break;
            default: content = ''; break;
        }

        
                const rawContentLength = content.replace(/\x1B\[[0-9;]*m/g, '').length; 
                const totalPaddingNeeded = Math.max(0, boxWidth - rawContentLength);
        
                const paddingLeft = Math.floor(totalPaddingNeeded / 2);
                const paddingRight = totalPaddingNeeded - paddingLeft;
                
                return chalk.gray('|') + ' '.repeat(paddingLeft) + content + ' '.repeat(paddingRight) + chalk.gray('|'); 
    }
}