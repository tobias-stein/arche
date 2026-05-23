import { Context, Effect, Layer } from "effect";
export class AgentStreamEmitter extends Context.Tag("AgentStreamEmitter")() {
}
export const noopAgentStreamEmitterLayer = Layer.succeed(AgentStreamEmitter, { emit: () => Effect.void });
/**
 * Build a layer that forwards each event to the provided callback.
 * The callback is invoked synchronously inside an `Effect.sync`; any error
 * thrown by the callback is caught and discarded so observability failures
 * cannot kill the run.
 */
export const callbackAgentStreamEmitterLayer = (onEvent) => Layer.succeed(AgentStreamEmitter, {
    emit: (event) => Effect.sync(() => {
        try {
            onEvent(event);
        }
        catch {
            // Swallow callback errors — a broken forwarder must not kill the run.
        }
    }),
});
//# sourceMappingURL=AgentStreamEmitter.js.map