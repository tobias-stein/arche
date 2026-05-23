/**
 * Merge env vars from the env resolver, agent provider, and sandbox provider.
 *
 * Provider env (agent + sandbox) overrides env resolver output for shared keys.
 * Agent and sandbox provider env must NOT have overlapping keys — if they do,
 * this function throws.
 */
export declare const mergeProviderEnv: (options: {
    readonly resolvedEnv: Record<string, string>;
    readonly agentProviderEnv: Record<string, string>;
    readonly sandboxProviderEnv: Record<string, string>;
}) => Record<string, string>;
//# sourceMappingURL=mergeProviderEnv.d.ts.map