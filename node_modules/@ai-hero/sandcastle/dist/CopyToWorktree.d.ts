import { Effect } from "effect";
import { CopyToWorktreeError, CopyToWorktreeTimeoutError } from "./errors.js";
/**
 * Returns cp flags for copy-on-write support:
 * - macOS (darwin): `-cR` uses APFS clonefile
 * - Other (Linux, etc.): `-R --reflink=auto` uses GNU coreutils reflink
 */
export declare const getCopyOnWriteFlags: (platform: string) => string[];
/**
 * Copy files and directories from the host repo root to the worktree root,
 * using copy-on-write when the filesystem supports it.
 * Missing paths are silently skipped.
 */
export declare const copyToWorktree: (paths: string[], hostRepoDir: string, worktreePath: string, timeoutMs?: number | undefined) => Effect.Effect<void, CopyToWorktreeError | CopyToWorktreeTimeoutError, never>;
//# sourceMappingURL=CopyToWorktree.d.ts.map