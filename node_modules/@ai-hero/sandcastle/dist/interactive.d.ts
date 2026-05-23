import type { AgentProvider } from "./AgentProvider.js";
import { type SandboxHooks } from "./SandboxLifecycle.js";
import type { AnySandboxProvider, BranchStrategy } from "./SandboxProvider.js";
import { type PromptArgs } from "./PromptArgumentSubstitution.js";
import type { Timeouts } from "./run.js";
export interface InteractiveOptions {
    /** Agent provider to use (e.g. claudeCode("claude-opus-4-7")) */
    readonly agent: AgentProvider;
    /** Sandbox provider (e.g. docker(), noSandbox()). */
    readonly sandbox?: AnySandboxProvider;
    /** Inline prompt string (mutually exclusive with promptFile). */
    readonly prompt?: string;
    /** Path to a prompt file (mutually exclusive with prompt). */
    readonly promptFile?: string;
    /** Optional name for the interactive session. */
    readonly name?: string;
    /** Branch strategy — controls how the agent's changes relate to branches.
     * Defaults to { type: "head" } for bind-mount providers and { type: "merge-to-head" } for isolated providers. */
    readonly branchStrategy?: BranchStrategy;
    /** Hooks to run during sandbox lifecycle */
    readonly hooks?: SandboxHooks;
    /** Paths relative to the host repo root to copy into the worktree before sandbox start. */
    readonly copyToWorktree?: string[];
    /** Key-value map for {{KEY}} placeholder substitution in prompts */
    readonly promptArgs?: PromptArgs;
    /** Environment variables to inject into the sandbox. */
    readonly env?: Record<string, string>;
    /**
     * Host repo directory to use instead of `process.cwd()`.
     *
     * Relative paths resolve against `process.cwd()`; absolute paths pass
     * through as-is. A {@link CwdError} is thrown if the path does not exist
     * or is not a directory.
     */
    readonly cwd?: string;
    /**
     * An `AbortSignal` that cancels the interactive session when aborted.
     *
     * - If `signal.aborted` is already `true` at entry, `interactive()` rejects
     *   immediately without doing any setup work.
     * - Aborting during an active session kills the agent subprocess.
     * - The rejected promise surfaces `signal.reason` via
     *   `signal.throwIfAborted()` — no Sandcastle-specific wrapping.
     * - The worktree is preserved on disk after abort (error-path behavior).
     */
    readonly signal?: AbortSignal;
    /** Override default timeouts for built-in lifecycle steps. Unset keys keep their defaults. */
    readonly timeouts?: Timeouts;
}
export interface InteractiveResult {
    /** List of commits made during the interactive session. */
    readonly commits: {
        sha: string;
    }[];
    /** The branch name the agent worked on. */
    readonly branch: string;
    /** Host path to the preserved worktree, if worktree had uncommitted changes. */
    readonly preservedWorktreePath?: string;
    /** Exit code of the interactive process. */
    readonly exitCode: number;
}
/**
 * Launch an interactive agent session inside a sandbox.
 *
 * The user sees the agent's TUI directly. When the session ends,
 * Sandcastle collects commits and handles branch merging, just like run().
 *
 * Full prompt preprocessing pipeline: PromptResolver -> PromptArgumentSubstitution
 * -> PromptPreprocessor (shell expressions inside sandbox).
 *
 * All three branch strategies are supported: head, merge-to-head, branch.
 */
export declare const interactive: (options: InteractiveOptions) => Promise<InteractiveResult>;
//# sourceMappingURL=interactive.d.ts.map