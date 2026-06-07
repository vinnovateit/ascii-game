export class Engine {
    // locked 30 FPS target
    private readonly TARGET_FPS = 30;
    private readonly FRAME_TIME_NS = BigInt(Math.floor(1_000_000_000 / this.TARGET_FPS)); 
    private lastFrameTime: bigint = process.hrtime.bigint();
    private isRunning: boolean = false;
    private tickCallback: () => void = () => {};

    public start(tick: () => void): void {
        this.isRunning = true;
        this.tickCallback = tick;
        this.lastFrameTime = process.hrtime.bigint();
        setImmediate(this.gameLoop);
    }

    public stop(): void {
        this.isRunning = false;
    }

    private gameLoop = (): void => {
        if (!this.isRunning) return;

        const now = process.hrtime.bigint();
        const elapsedTime = now - this.lastFrameTime;

        if (elapsedTime >= this.FRAME_TIME_NS) {
            this.tickCallback();
            this.lastFrameTime = now - (elapsedTime % this.FRAME_TIME_NS);
        }

        setImmediate(this.gameLoop);
    };
}