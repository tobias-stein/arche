/**
 * Filesystem-based test bind-mount sandbox provider.
 *
 * Uses a temp directory on the local filesystem as the "sandbox".
 * Intended for testing the bind-mount provider abstraction without
 * requiring Docker or Podman.
 */
import { type BindMountSandboxProvider } from "../SandboxProvider.js";
/**
 * Create a filesystem-based test bind-mount sandbox provider.
 *
 * The "sandbox" is a temp directory. `exec` runs shell commands in it,
 * `copyFileIn`/`copyFileOut` copy single files between host and the temp dir,
 * and `close` removes the temp dir.
 */
export declare const testBindMount: () => BindMountSandboxProvider;
//# sourceMappingURL=test-bind-mount.d.ts.map