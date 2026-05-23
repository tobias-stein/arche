import { Effect } from "effect";
/**
 * Race an Effect against an optional `AbortSignal`.
 *
 * - If no signal is provided, the effect runs unmodified.
 * - If the signal is already aborted at call time, dies immediately with
 *   `signal.reason`.
 * - Otherwise, races the effect against a Deferred that fires when the
 *   signal aborts.
 *
 * Uses `Effect.die` so the abort reason propagates as a defect — callers
 * should use `signal.throwIfAborted()` in their catch handler to surface
 * the original reason without Sandcastle-specific wrapping.
 *
 * The abort listener is always cleaned up, even when the effect wins the race.
 */
export declare const raceAbortSignal: <A, E, R>(effect: Effect.Effect<A, E, R>, signal?: AbortSignal | undefined) => Effect.Effect<A, E, R>;
//# sourceMappingURL=raceAbortSignal.d.ts.map