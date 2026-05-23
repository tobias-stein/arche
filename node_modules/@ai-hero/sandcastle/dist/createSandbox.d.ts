import { Layer } from "effect";
import type { AgentProvider } from "./AgentProvider.js";
import { type IterationResult } from "./Orchestrator.js";
import { type PromptArgs } from "./PromptArgumentSubstitution.js";
import type { LoggingOption, Timeouts } from "./run.js";
import { type SandboxHooks } from "./SandboxLifecycle.js";
import { Sandbox as SandboxTag } from "./SandboxFactory.js";
import type { SandboxProvider } from "./SandboxProvider.js";
export interface CreateSandboxOptions {
    /** Explicit branch for the worktree (required). */
    readonly branch: string;
    /**
     * Ref to fork from when `branch` does not yet exist. Ignored when the branch
     * already exists. Defaults to `HEAD`.
     */
    readonly baseBranch?: string;
    /** Sandbox provider (e.g. docker({ imageName: "sandcastle:myrepo" })). */
    readonly sandbox: SandboxProvider;
    /**
     * Host repo directory. Replaces `process.cwd()` as the anchor for
     * `.sandcastle/worktrees/`, `.sandcastle/.env`, and git operations.
     *
     * - Relative paths are resolved against `process.cwd()`.
     * - Absolute paths are used as-is.
     * - Defaults to `process.cwd()` when omitted.
     */
    readonly cwd?: string;
    /** Lifecycle hooks grouped by execution location (host or sandbox). */
    readonly hooks?: SandboxHooks;
    /** Paths relative to the host repo root to copy into the worktree at creation time. */
    readonly copyToWorktree?: string[];
    /** Override default timeouts for built-in lifecycle steps. Unset keys keep their defaults. */
    readonly timeouts?: Timeouts;
    /** @internal Test-only overrides to bypass the sandbox provider. */
    readonly _test?: {
        readonly buildSandboxLayer?: (sandboxDir: string) => Layer.Layer<SandboxTag>;
    };
}
export interface SandboxRunOptions {
    /** Agent provider to use (e.g. claudeCode("claude-opus-4-7")). */
    readonly agent: AgentProvider;
    /** Inline prompt string (mutually exclusive with promptFile). */
    readonly prompt?: string;
    /** Path to a prompt file (mutually exclusive with prompt). */
    readonly promptFile?: string;
    /** Key-value map for {{KEY}} placeholder substitution in prompts. */
    readonly promptArgs?: PromptArgs;
    /** Maximum iterations to run (default: 1). */
    readonly maxIterations?: number;
    /** Substring(s) the agent emits to stop the iteration loop early. */
    readonly completionSignal?: string | string[];
    /** Idle timeout in seconds. Default: 600. */
    readonly idleTimeoutSeconds?: number;
    /** Display name for this run. */
    readonly name?: string;
    /** Logging mode. */
    readonly logging?: LoggingOption;
    /**
     * An `AbortSignal` that cancels the run when aborted.
     *
     * - Pre-aborted signal rejects immediately without setup.
     * - Mid-iteration abort kills the in-flight agent subprocess.
     * - The rejected promise surfaces `signal.reason` verbatim.
     * - The `Sandbox` handle remains usable after abort — call `.run()` again
     *   with a fresh signal, or `.close()` to tear down.
     */
    readonly signal?: AbortSignal;
}
export interface SandboxRunResult {
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
    /** Path to the log file, if logging was drained to a file. */
    readonly logFilePath?: string;
}
export interface SandboxInteractiveOptions {
    /** Agent provider to use (e.g. claudeCode("claude-opus-4-7")). */
    readonly agent: AgentProvider;
    /** Inline prompt string (mutually exclusive with promptFile). */
    readonly prompt?: string;
    /** Path to a prompt file (mutually exclusive with prompt). */
    readonly promptFile?: string;
    /** Key-value map for {{KEY}} placeholder substitution in prompts. */
    readonly promptArgs?: PromptArgs;
    /** Display name for this interactive session. */
    readonly name?: string;
    /**
     * An `AbortSignal` that cancels the interactive session when aborted.
     *
     * - Pre-aborted signal rejects immediately without setup.
     * - The rejected promise surfaces `signal.reason` verbatim.
     * - The `Sandbox` handle remains usable after abort.
     */
    readonly signal?: AbortSignal;
}
export interface SandboxInteractiveResult {
    /** List of commits made during the interactive session. */
    readonly commits: {
        sha: string;
    }[];
    /** Exit code of the interactive process. */
    readonly exitCode: number;
}
export interface CloseResult {
    /** Host path to the preserved worktree, set when the worktree had uncommitted changes. */
    readonly preservedWorktreePath?: string;
}
export interface Sandbox {
    /** The branch the worktree is on. */
    readonly branch: string;
    /** Host path to the worktree. */
    readonly worktreePath: string;
    /** Invoke an agent inside the existing sandbox. */
    run(options: SandboxRunOptions): Promise<SandboxRunResult>;
    /** Launch an interactive agent session inside the existing sandbox. */
    interactive(options: SandboxInteractiveOptions): Promise<SandboxInteractiveResult>;
    /** Tear down the sandbox and worktree. */
    close(): Promise<CloseResult>;
    /** Auto teardown via `await using`. */
    [Symbol.asyncDispose](): Promise<void>;
}
/** @internal Options for createSandboxFromWorktree — used by worktree.createSandbox(). */
export interface CreateSandboxFromWorktreeOptions {
    readonly branch: string;
    readonly worktreePath: string;
    readonly hostRepoDir: string;
    readonly sandbox: SandboxProvider;
    readonly hooks?: SandboxHooks;
    readonly copyToWorktree?: string[];
    readonly timeouts?: Timeouts;
    readonly _test?: {
        readonly buildSandboxLayer?: (sandboxDir: string) => Layer.Layer<SandboxTag>;
    };
}
/**
 * @internal Creates a sandbox backed by an existing worktree.
 * Split ownership: close() tears down the container only, leaving the worktree intact.
 * Used by Worktree.createSandbox().
 */
export declare const createSandboxFromWorktree: (options: CreateSandboxFromWorktreeOptions) => Promise<Sandbox>;
/**
 * Eagerly creates a git worktree on the provided explicit branch and starts
 * a sandbox with the worktree bind-mounted. Returns a Sandbox handle that
 * can be reused across multiple `run()` calls.
 */
export declare const createSandbox: (options: CreateSandboxOptions) => Promise<Sandbox>;
//# sourceMappingURL=createSandbox.d.ts.map