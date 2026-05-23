import { Context, Effect, Layer } from "effect";
import { FileSystem } from "@effect/platform";
import type { PlatformError } from "@effect/platform/Error";
import { CopyError, ExecError, SyncError, type SandboxError } from "./errors.js";
import type { Timeouts } from "./run.js";
import { Display } from "./Display.js";
import type { SandboxProvider, BranchStrategy, BindMountSandboxHandle, IsolatedSandboxHandle, NoSandboxHandle } from "./SandboxProvider.js";
import { type SandboxHooks } from "./SandboxLifecycle.js";
export interface ExecResult {
    readonly stdout: string;
    readonly stderr: string;
    readonly exitCode: number;
}
export interface SandboxService {
    readonly exec: (command: string, options?: {
        onLine?: (line: string) => void;
        cwd?: string;
        sudo?: boolean;
        stdin?: string;
    }) => Effect.Effect<ExecResult, ExecError>;
    /** Copy a file or directory from the host into the sandbox. */
    readonly copyIn: (hostPath: string, sandboxPath: string) => Effect.Effect<void, CopyError>;
    /** Copy a single file from the sandbox to the host. */
    readonly copyFileOut: (sandboxPath: string, hostPath: string) => Effect.Effect<void, CopyError>;
}
declare const Sandbox_base: Context.TagClass<Sandbox, "Sandbox", SandboxService>;
export declare class Sandbox extends Sandbox_base {
}
/**
 * Wrap a Promise-based sandbox handle into an Effect-based SandboxService layer.
 * Delegates copyIn/copyFileOut to the handle when available.
 */
export declare const makeSandboxLayerFromHandle: (handle: BindMountSandboxHandle | IsolatedSandboxHandle | NoSandboxHandle) => Layer.Layer<Sandbox, never, never>;
/** The mount point inside the sandbox where the project worktree is bound. */
export declare const SANDBOX_REPO_DIR = "/home/agent/workspace";
export interface SandboxInfo {
    /** Host-side path to the worktree directory (worktree/branch mode only). */
    readonly hostWorktreePath?: string;
    /** Absolute path to the worktree inside the sandbox, as reported by the provider. */
    readonly sandboxRepoPath: string;
    /** Sync changes from the sandbox to the host worktree (isolated providers only). */
    readonly applyToHost?: () => Effect.Effect<void, SyncError>;
    /** The bind-mount sandbox handle, available when the provider is a bind-mount provider. Used for session capture. */
    readonly bindMountHandle?: BindMountSandboxHandle;
}
export interface WithSandboxResult<A> {
    readonly value: A;
    /** Host path to the preserved worktree, set when the worktree was left behind due to uncommitted changes. */
    readonly preservedWorktreePath?: string;
}
declare const SandboxFactory_base: Context.TagClass<SandboxFactory, "SandboxFactory", {
    readonly withSandbox: <A, E, R>(makeEffect: (info: SandboxInfo) => Effect.Effect<A, E, R | Sandbox>) => Effect.Effect<WithSandboxResult<A>, E | SandboxError, Exclude<R, Sandbox>>;
}>;
export declare class SandboxFactory extends SandboxFactory_base {
}
declare const SandboxConfig_base: Context.TagClass<SandboxConfig, "SandboxConfig", {
    readonly env: Record<string, string>;
    readonly hostRepoDir: string;
    /** Paths relative to the host repo root to copy into the worktree before sandbox start. */
    readonly copyToWorktree?: string[] | undefined;
    /** When specified, the run name is included in the auto-generated branch and worktree names. */
    readonly name?: string | undefined;
    /** Sandbox provider — delegates sandbox lifecycle to the provider. */
    readonly sandboxProvider: SandboxProvider;
    /** Branch strategy — controls how the agent's changes relate to branches. */
    readonly branchStrategy: BranchStrategy;
    /** Lifecycle hooks grouped by execution location (host or sandbox). */
    readonly hooks?: SandboxHooks | undefined;
    /** AbortSignal threaded to lifecycle hooks so they can cooperatively cancel. */
    readonly signal?: AbortSignal | undefined;
    /** Override default timeouts for built-in lifecycle steps. */
    readonly timeouts?: Timeouts | undefined;
}>;
export declare class SandboxConfig extends SandboxConfig_base {
}
export interface MountEntry {
    readonly hostPath: string;
    readonly sandboxPath: string;
}
/**
 * Resolves the git-related mounts needed for the sandbox.
 * Handles both normal repos (where .git is a directory) and worktrees
 * (where .git is a file pointing to the parent repo's .git/worktrees/<name>).
 */
export declare const resolveGitMounts: (gitPath: string) => Effect.Effect<MountEntry[], PlatformError, FileSystem.FileSystem>;
export declare const WorktreeDockerSandboxFactory: {
    layer: Layer.Layer<SandboxFactory, never, Display | FileSystem.FileSystem | SandboxConfig>;
};
export {};
//# sourceMappingURL=SandboxFactory.d.ts.map