import { Data, Duration, Effect } from "effect";
/** Command execution failed in the sandbox */
export class ExecError extends Data.TaggedError("ExecError") {
}
/** Command execution failed on the host */
export class ExecHostError extends Data.TaggedError("ExecHostError") {
}
/** File copy between host and sandbox failed */
export class CopyError extends Data.TaggedError("CopyError") {
}
/** Docker infrastructure operation failed */
export class DockerError extends Data.TaggedError("DockerError") {
}
/** Podman infrastructure operation failed */
export class PodmanError extends Data.TaggedError("PodmanError") {
}
/** Git sync-in or sync-out operation failed */
export class SyncError extends Data.TaggedError("SyncError") {
}
/** Git worktree operation failed */
export class WorktreeError extends Data.TaggedError("WorktreeError") {
}
/** Prompt resolution or preprocessing failed */
export class PromptError extends Data.TaggedError("PromptError") {
}
/** Agent invocation failed */
export class AgentError extends Data.TaggedError("AgentError") {
}
/** .sandcastle/ config directory missing */
export class ConfigDirError extends Data.TaggedError("ConfigDirError") {
}
/** Initialization or setup operation failed */
export class InitError extends Data.TaggedError("InitError") {
}
/** Run exceeded the configured agent idle timeout */
export class AgentIdleTimeoutError extends Data.TaggedError("AgentIdleTimeoutError") {
}
/** Git worktree create or prune timed out */
export class WorktreeTimeoutError extends Data.TaggedError("WorktreeTimeoutError") {
}
/** Sandbox container start timed out */
export class ContainerStartTimeoutError extends Data.TaggedError("ContainerStartTimeoutError") {
}
/** Copying files to worktree timed out */
export class CopyToWorktreeTimeoutError extends Data.TaggedError("CopyToWorktreeTimeoutError") {
}
/** Fallback cp -R to worktree failed */
export class CopyToWorktreeError extends Data.TaggedError("CopyToWorktreeError") {
}
/** Git sync-in for isolated providers timed out */
export class SyncInTimeoutError extends Data.TaggedError("SyncInTimeoutError") {
}
/** onSandboxReady hook command timed out */
export class HookTimeoutError extends Data.TaggedError("HookTimeoutError") {
}
/** Git config setup command timed out */
export class GitSetupTimeoutError extends Data.TaggedError("GitSetupTimeoutError") {
}
/** Prompt shell expression expansion timed out */
export class PromptExpansionTimeoutError extends Data.TaggedError("PromptExpansionTimeoutError") {
}
/** Commit collection timed out */
export class CommitCollectionTimeoutError extends Data.TaggedError("CommitCollectionTimeoutError") {
}
/** Merge-to-host branch timed out */
export class MergeToHostTimeoutError extends Data.TaggedError("MergeToHostTimeoutError") {
}
/**
 * Wrap an effect with a timeout that fails with a specific error on expiry.
 * Uses `Effect.timeoutFail` under the hood.
 */
export const withTimeout = (timeoutMs, onTimeout) => (effect) => effect.pipe(Effect.timeoutFail({
    duration: Duration.millis(timeoutMs),
    onTimeout,
}));
/** Session capture (read, rewrite, or write) failed */
export class SessionCaptureError extends Data.TaggedError("SessionCaptureError") {
}
/** The provided `cwd` path does not exist or is not a directory. */
export { CwdError } from "./resolveCwd.js";
//# sourceMappingURL=errors.js.map