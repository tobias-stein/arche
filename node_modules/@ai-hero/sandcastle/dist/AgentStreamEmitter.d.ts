import { Context, Effect, Layer } from "effect";
/**
 * A single event in the agent's output stream, surfaced to callers of `run()`
 * so they can forward it to their own observability system.
 *
 * Emitted only in log-to-file mode when an `onAgentStreamEvent` callback is
 * provided via `logging`. See `run()`.
 */
export type AgentStreamEvent = {
    readonly type: "text";
    readonly message: string;
    readonly iteration: number;
    readonly timestamp: Date;
} | {
    readonly type: "toolCall";
    readonly name: string;
    readonly formattedArgs: string;
    readonly iteration: number;
    readonly timestamp: Date;
};
export interface AgentStreamEmitterService {
    readonly emit: (event: AgentStreamEvent) => Effect.Effect<void>;
}
declare const AgentStreamEmitter_base: Context.TagClass<AgentStreamEmitter, "AgentStreamEmitter", AgentStreamEmitterService>;
export declare class AgentStreamEmitter extends AgentStreamEmitter_base {
}
export declare const noopAgentStreamEmitterLayer: Layer.Layer<AgentStreamEmitter>;
/**
 * Build a layer that forwards each event to the provided callback.
 * The callback is invoked synchronously inside an `Effect.sync`; any error
 * thrown by the callback is caught and discarded so observability failures
 * cannot kill the run.
 */
export declare const callbackAgentStreamEmitterLayer: (onEvent: (event: AgentStreamEvent) => void) => Layer.Layer<AgentStreamEmitter, never, never>;
export {};
//# sourceMappingURL=AgentStreamEmitter.d.ts.map