import chalk from 'chalk';


export class dialogue {
    private height: number;
    constructor(height: number){
        this.height = height/2;
    }


    public getdialogueBox(lineIndex: number, boxWidth: number): string {
        let content = '';

        switch (lineIndex) {
            case 0:
            content = chalk.bgYellow(chalk.white('dialogue Box'));break;
            case this.height-2: return content = `+${'-'.repeat(boxWidth)}+`;break;
            case this.height-1: return content = `+${'-'.repeat(boxWidth)}+`;break;
            default:
                content = '';
                break;
        }

        const rawContentLength = content.replace(/\x1B\[[0-9;]*m/g, '').length; 
        const totalPaddingNeeded = Math.max(0, boxWidth - rawContentLength);

        const paddingLeft = Math.floor(totalPaddingNeeded / 2);
        const paddingRight = totalPaddingNeeded - paddingLeft;
        
        return chalk.grey('|') + ' '.repeat(paddingLeft) + content + ' '.repeat(paddingRight) + chalk.grey('|'); 
    }
}