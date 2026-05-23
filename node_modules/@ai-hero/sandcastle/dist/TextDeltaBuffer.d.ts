/**
 * Buffers streaming text deltas and flushes them as readable multi-word chunks.
 *
 * Flush triggers:
 * - Newline character in the accumulated buffer
 * - Sentence boundary (`. `, `! `, `? ` at end of buffer)
 * - Buffer exceeds ~80 characters
 * - Debounce timer (~50ms of no new writes)
 */
export declare class TextDeltaBuffer {
    private buffer;
    private timer;
    private readonly onFlush;
    constructor(onFlush: (text: string) => void);
    write(text: string): void;
    /** Force-flush any buffered text. */
    flush(): void;
    /** Flush remaining buffer and clean up. */
    dispose(): void;
    private shouldFlush;
    private doFlush;
    private clearTimer;
}
//# sourceMappingURL=TextDeltaBuffer.d.ts.map