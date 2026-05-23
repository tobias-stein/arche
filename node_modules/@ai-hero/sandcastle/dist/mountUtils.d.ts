/**
 * Shared mount utilities for Docker and Podman sandbox providers.
 *
 * Handles host/sandbox path resolution, tilde expansion, user mount
 * validation, image naming, and Windows path normalization.
 */
import type { MountConfig } from "./MountConfig.js";
/**
 * SELinux volume label suffix applied to bind mounts.
 *
 * - `"z"` — shared label. No-op on non-SELinux systems.
 * - `"Z"` — private label; only this container can access the mount.
 * - `false` — disable labeling entirely.
 */
export type SelinuxLabel = "z" | "Z" | false;
/**
 * Deterministic mount point inside the sandbox for the parent repo's .git
 * directory when the workspace is a git worktree. See ADR-0006.
 */
export declare const PARENT_GIT_SANDBOX_DIR = "/.sandcastle-parent-git";
/**
 * Derive the default image name from the repo directory.
 * Returns `sandcastle:<dir-name>` where dir-name is the last path segment,
 * lowercased and sanitized for image tag rules.
 *
 * Handles both POSIX (`/`) and Windows (`\`) path separators.
 */
export declare const defaultImageName: (repoDir: string) => string;
/**
 * Expand tilde (`~`) to the given home directory (or `os.homedir()` if omitted).
 * Handles both `~/path` (POSIX) and `~\path` (Windows).
 */
export declare const expandTilde: (p: string, homeDirPath?: string | undefined) => string;
/**
 * Resolve a host path: expand tilde, then resolve relative paths from `process.cwd()`.
 */
export declare const resolveHostPath: (hostPath: string) => string;
/**
 * Resolve a sandbox path: expands tilde using `sandboxHomedir`, then resolves
 * relative paths from `SANDBOX_REPO_DIR`.
 *
 * Throws if `sandboxPath` starts with `~` but `sandboxHomedir` is `undefined`.
 */
export declare const resolveSandboxPath: (sandboxPath: string, sandboxHomedir?: string | undefined) => string;
/**
 * Resolve and validate user-provided mount configurations.
 * Throws if a hostPath does not exist on the filesystem.
 * Throws if a sandboxPath uses tilde but `sandboxHomedir` is `undefined`.
 */
export declare const resolveUserMounts: (mounts: readonly MountConfig[], sandboxHomedir?: string | undefined) => {
    hostPath: string;
    sandboxPath: string;
    readonly?: boolean | undefined;
}[];
/**
 * Normalize mount entries for cross-platform compatibility.
 *
 * On Windows (`platform === "win32"`):
 * - Replaces backslashes with forward slashes in all `hostPath` values
 * - Remaps `sandboxPath` values that look like Windows paths to valid POSIX
 *   paths relative to `sandboxRepoDir` when the host path is under the
 *   worktree host path
 *
 * On non-Windows platforms, returns mounts unchanged (preserving
 * `sandboxPath === hostPath` for git mounts so that `gitdir:` references
 * resolve correctly inside the container).
 *
 * This is a pure function — no filesystem access, accepts a `platform`
 * parameter so it can be unit-tested with fake Windows paths.
 */
export declare const normalizeMounts: <M extends {
    hostPath: string;
    sandboxPath: string;
}>(mounts: M[], worktreeHostPath: string, sandboxRepoDir: string, platform?: string) => M[];
/**
 * Parse a `gitdir:` path into its parent .git directory and worktree name.
 *
 * The gitdir path is like `/path/to/repo/.git/worktrees/<name>` or
 * `C:\Users\project\.git\worktrees\<name>`. We split on both `/` and `\`
 * so this works regardless of the host platform or which platform runs tests.
 */
export declare const parseGitdirPath: (gitdirPath: string) => {
    parentGitDir: string;
    worktreeName: string;
};
/**
 * On Windows, patch git mounts so that worktree `.git` files resolve inside
 * the Linux sandbox. See ADR-0006 for the full rationale.
 *
 * Two fixes are applied:
 * 1. The parent `.git` directory mount is remapped to `PARENT_GIT_SANDBOX_DIR`.
 * 2. A corrected `.git` file (with a POSIX `gitdir:` path) is created and
 *    mounted at `sandboxRepoDir/.git`, overlaying the original.
 *
 * On non-Windows platforms, or when the worktree's `.git` is a directory
 * (not a worktree pointer), returns the mounts unchanged.
 *
 * @param gitMounts - Raw mounts from `resolveGitMounts`.
 * @param worktreeHostPath - Host path to the directory mounted at `sandboxRepoDir`.
 * @param sandboxRepoDir - Where the worktree is mounted inside the sandbox.
 * @param readFile - Read a file's content (injectable for tests).
 * @param statFile - Stat a file to check if it's a directory (injectable for tests).
 * @param platform - Override for `process.platform` (injectable for tests).
 */
export declare const patchGitMountsForWindows: (gitMounts: {
    hostPath: string;
    sandboxPath: string;
}[], worktreeHostPath: string, sandboxRepoDir: string, readFile?: ((path: string) => Promise<string>) | undefined, statFile?: ((path: string) => Promise<"directory" | "file">) | undefined, platform?: string) => Promise<{
    hostPath: string;
    sandboxPath: string;
}[]>;
/**
 * Format a bind mount into a `-v` style string for container runtimes.
 *
 * Produces: `hostPath:sandboxPath[:ro][,z|Z]`
 *
 * Used by both Podman and Docker providers.
 */
export declare const formatVolumeMount: (mount: {
    hostPath: string;
    sandboxPath: string;
    readonly?: boolean | undefined;
}, selinuxLabel: SelinuxLabel | undefined) => string;
/**
 * Detect file-target mounts whose sandbox-side parent directory may not
 * exist in the container image, and return the parent dirs that need to
 * be created at container start.
 *
 * Validates at config time: if a file mount's sandbox parent is outside
 * `sandboxHomedir`, throws with a clear error and remediation guidance.
 *
 * For file mounts whose parent is under `sandboxHomedir` (but not equal
 * to it — `/home/agent` always exists), returns the unique set of parent
 * directories that must be `mkdir -p` + `chown`'d before the agent runs.
 *
 * @param mounts - Resolved mounts (hostPath already validated to exist).
 * @param sandboxHomedir - The agent's home directory in the sandbox (e.g. `/home/agent`).
 * @param statFn - Injectable stat function for testing (defaults to `statSync`).
 */
export declare const processFileMountParents: (mounts: readonly {
    hostPath: string;
    sandboxPath: string;
}[], sandboxHomedir: string, statFn?: (path: string) => {
    isFile(): boolean;
}) => string[];
//# sourceMappingURL=mountUtils.d.ts.map