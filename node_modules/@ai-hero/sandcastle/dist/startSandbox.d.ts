import { Effect, Layer } from "effect";
import { ContainerStartTimeoutError, CopyToWorktreeTimeoutError, SyncError, SyncInTimeoutError, WorktreeError, type DockerError } from "./errors.js";
import type { BindMountSandboxProvider, BindMountSandboxHandle, IsolatedSandboxProvider, IsolatedSandboxHandle, NoSandboxProvider, NoSandboxHandle } from "./SandboxProvider.js";
import { type Sandbox, type MountEntry } from "./SandboxFactory.js";
export interface StartSandboxBindMountOptions {
    provider: BindMountSandboxProvider;
    hostRepoDir: string;
    env: Record<string, string>;
    worktreeOrRepoPath: string;
    gitMounts: MountEntry[];
    repoDir: string;
    copyPaths?: undefined;
}
export interface StartSandboxIsolatedOptions {
    provider: IsolatedSandboxProvider;
    hostRepoDir: string;
    env: Record<string, string>;
    worktreeOrRepoPath?: undefined;
    gitMounts?: undefined;
    repoDir?: undefined;
    copyPaths?: string[];
}
export interface StartSandboxNoSandboxOptions {
    provider: NoSandboxProvider;
    hostRepoDir: string;
    env: Record<string, string>;
    /** Host-side worktree path the agent will run in. Equal to hostRepoDir in head mode. */
    worktreeOrRepoPath: string;
    gitMounts?: undefined;
    repoDir?: undefined;
    copyPaths?: undefined;
}
export type StartSandboxOptions = StartSandboxBindMountOptions | StartSandboxIsolatedOptions | StartSandboxNoSandboxOptions;
export interface StartSandboxResult {
    handle: BindMountSandboxHandle | IsolatedSandboxHandle | NoSandboxHandle;
    sandboxLayer: Layer.Layer<Sandbox>;
    worktreePath: string;
}
export declare const COPY_PATHS_TIMEOUT_MS = 120000;
/**
 * Start a sandbox by dispatching on `provider.tag`.
 *
 * - `"bind-mount"`: creates mounts and delegates to the provider's `create()`.
 * - `"isolated"`: creates handle, syncs host repo via git bundle, then copies
 *   optional `copyPaths` via `handle.copyIn()`.
 *
 * Returns the handle, a `SandboxService` layer, and the worktree path.
 */
export declare const startSandbox: (options: StartSandboxOptions) => Effect.Effect<StartSandboxResult, ContainerStartTimeoutError | CopyToWorktreeTimeoutError | DockerError | SyncError | SyncInTimeoutError | WorktreeError, never>;
//# sourceMappingURL=startSandbox.d.ts.map