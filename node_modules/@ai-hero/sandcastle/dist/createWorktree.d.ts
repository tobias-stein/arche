import type { AgentProvider } from "./AgentProvider.js";
import { type SandboxHooks } from "./SandboxLifecycle.js";
import type { AnySandboxProvider, SandboxProvider, MergeToHeadBranchStrategy, NamedBranchStrategy } from "./SandboxProvider.js";
import type { CloseResult, Sandbox } from "./createSandbox.js";
import type { InteractiveResult } from "./interactive.js";
import type { LoggingOption } from "./run.js";
import { type IterationResult } from "./Orchestrator.js";
import { type PromptArgs } from "./PromptArgumentSubstitution.js";
import type { Timeouts } from "./run.js";
/** Branch strategies valid for createWorktree — head is excluded. */
export type WorktreeBranchStrategy = MergeToHeadBranchStrategy | NamedBranchStrategy;
export interface CreateWorktreeOptions {
    /** Branch strategy — only 'branch' and 'merge-to-head' are allowed. */
    readonly branchStrategy: WorktreeBranchStrategy;
    /**
     * Host repo directory. Replaces `process.cwd()` as the anchor for
     * `.sandcastle/worktrees/`, `.sandcastle/.env`, and git operations.
     *
     * - Relative paths are resolved against `process.cwd()`.
     * - Absolute paths are used as-is.
     * - Defaults to `process.cwd()` when omitted.
     */
    readonly cwd?: string;
    /** Paths relative to the host repo root to copy into the worktree at creation time. */
    readonly copyToWorktree?: string[];
    /** Lifecycle hooks grouped by execution location (host or sandbox).
     *  Only `host.onWorktreeReady` is executed here — other hooks are passed through
     *  to `run()`, `interactive()`, or `createSandbox()`. */
    readonly hooks?: SandboxHooks;
    /** Override default timeouts for built-in lifecycle steps. Unset keys keep their defaults. */
    readonly timeouts?: Timeouts;
}
export interface WorktreeInteractiveOptions {
    /** Agent provider to use (e.g. claudeCode("claude-opus-4-7")) */
    readonly agent: AgentProvider;
    /** Sandbox provider (e.g. docker(), noSandbox()). Defaults to noSandbox(). */
    readonly sandbox?: AnySandboxProvider;
    /** Inline prompt string (mutually exclusive with promptFile). */
    readonly prompt?: string;
    /** Path to a prompt file (mutually exclusive with prompt). */
    readonly promptFile?: string;
    /** Optional name for the interactive session. */
    readonly name?: string;
    /** Hooks to run during sandbox lifecycle */
    readonly hooks?: SandboxHooks;
    /** Key-value map for {{KEY}} placeholder substitution in prompts */
    readonly promptArgs?: PromptArgs;
    /** Environment variables to inject into the sandbox. */
    readonly env?: Record<string, string>;
    /**
     * An `AbortSignal` that cancels the interactive session when aborted.
     *
     * - If `signal.aborted` is already `true` at entry, rejects immediately.
     * - Aborting during an active session kills the agent subprocess.
     * - The worktree is preserved on disk after abort.
     * - The `Worktree` handle remains usable for subsequent operations.
     * - The rejected promise surfaces `signal.reason` via
     *   `signal.throwIfAborted()` — no Sandcastle-specific wrapping.
     */
    readonly signal?: AbortSignal;
}
export interface WorktreeRunOptions {
    /** Agent provider to use (e.g. claudeCode("claude-opus-4-7")) */
    readonly agent: AgentProvider;
    /** Sandbox provider (e.g. docker()). Required — AFK agents should always be sandboxed. */
    readonly sandbox: SandboxProvider;
    /** Inline prompt string (mutually exclusive with promptFile). */
    readonly prompt?: string;
    /** Path to a prompt file (mutually exclusive with prompt). */
    readonly promptFile?: string;
    /** Key-value map for {{KEY}} placeholder substitution in prompts */
    readonly promptArgs?: PromptArgs;
    /** Maximum iterations to run (default: 1). */
    readonly maxIterations?: number;
    /** Substring(s) the agent emits to stop the iteration loop early. */
    readonly completionSignal?: string | string[];
    /** Idle timeout in seconds. Default: 600. */
    readonly idleTimeoutSeconds?: number;
    /** Optional name for the run. */
    readonly name?: string;
    /** Logging mode. */
    readonly logging?: LoggingOption;
    /** Hooks to run during sandbox lifecycle */
    readonly hooks?: SandboxHooks;
    /** Environment variables to inject into the sandbox. */
    readonly env?: Record<string, string>;
    /** Resume a prior Claude Code session by ID. The session JSONL must exist on the host. Incompatible with maxIterations > 1. */
    readonly resumeSession?: string;
    /**
     * An `AbortSignal` that cancels the run when aborted.
     *
     * - If `signal.aborted` is already `true` at entry, rejects immediately
     *   without doing any setup work.
     * - Aborting mid-iteration kills the in-flight agent subprocess.
     * - The worktree is preserved on disk after abort.
     * - The `Worktree` handle remains usable for subsequent operations.
     */
    readonly signal?: AbortSignal;
}
export interface WorktreeRunResult {
    /** Per-iteration results (use `iterations.length` for the count). */
    readonly iterations: IterationResult[];
    /** The matched completion signal string, or undefined if none fired. */
    readonly completionSignal?: string;
    /** Combined stdout output from all agent iterations. */
    readonly stdout: string;
    /** List of commits made by the agent during the run. */
    readonly commits: {
        sha: string;
    }[];
    /** The branch name the agent worked on. */
    readonly branch: string;
    /** Path to the log file, if logging was drained to a file. */
    readonly logFilePath?: string;
}
export interface WorktreeCreateSandboxOptions {
    /** Sandbox provider (e.g. docker({ imageName: "sandcastle:myrepo" })). */
    readonly sandbox: SandboxProvider;
    /** Lifecycle hooks grouped by execution location (host or sandbox). */
    readonly hooks?: SandboxHooks;
    /** Paths relative to the host repo root to copy into the worktree at creation time. */
    readonly copyToWorktree?: string[];
    /** Override default timeouts for built-in lifecycle steps. Unset keys keep their defaults. */
    readonly timeouts?: Timeouts;
    /** @internal Test-only overrides to bypass the sandbox provider. */
    readonly _test?: {
        readonly buildSandboxLayer?: (sandboxDir: string) => import("effect").Layer.Layer<import("./SandboxFactory.js").Sandbox>;
    };
}
export interface Worktree {
    /** The branch the worktree is on. */
    readonly branch: string;
    /** Host path to the worktree (worktree). */
    readonly worktreePath: string;
    /** Run an AFK agent in this worktree with a required sandbox. */
    run(options: WorktreeRunOptions): Promise<WorktreeRunResult>;
    /** Run an interactive agent session in this worktree. */
    interactive(options: WorktreeInteractiveOptions): Promise<InteractiveResult>;
    /** Create a long-lived sandbox backed by this worktree's worktree. */
    createSandbox(options: WorktreeCreateSandboxOptions): Promise<Sandbox>;
    /** Clean up the worktree. Preserves worktree if dirty. */
    close(): Promise<CloseResult>;
    /** Auto cleanup via `await using`. */
    [Symbol.asyncDispose](): Promise<void>;
}
/**
 * Creates a git worktree as an independent, first-class worktree.
 * Returns a Worktree handle with close() and [Symbol.asyncDispose]().
 *
 * Only accepts 'branch' and 'merge-to-head' strategies — 'head' is a
 * compile-time type error since head means no worktree.
 */
export declare const createWorktree: (options: CreateWorktreeOptions) => Promise<Worktree>;
//# sourceMappingURL=createWorktree.d.ts.map