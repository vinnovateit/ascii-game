import chalk from 'chalk';


export class Chat {
    
    public getchatbox(lineIndex: number, boxWidth: number): string {
        let content = '';

        switch (lineIndex) {
            case 0:
            content = chalk.bgCyan(chalk.white('Chatbox'));                break;
            case 2:
                content = ``;
                break;
            default:
                content = '';
                break;
        }

        //find length and then get space needed on left and right to center the text in the box used similar code for almsot all rendering 
        const rawContentLength = content.replace(/\x1B\[[0-9;]*m/g, '').length; 
        const totalPaddingNeeded = Math.max(0, boxWidth - rawContentLength);

        const paddingLeft = Math.floor(totalPaddingNeeded / 2);
        const paddingRight = totalPaddingNeeded - paddingLeft;
        
        return chalk.grey('|') + ' '.repeat(paddingLeft) + content + ' '.repeat(paddingRight) + chalk.grey('|'); 
    }
}