/**
 * Filesystem-based test isolated sandbox provider.
 *
 * Uses a temp directory on the local filesystem as the "sandbox".
 * Intended for testing the isolated provider abstraction without
 * requiring a real remote environment.
 */
import { type IsolatedSandboxProvider } from "../SandboxProvider.js";
/**
 * Create a filesystem-based test isolated sandbox provider.
 *
 * The "sandbox" is a temp directory. `exec` runs shell commands in it,
 * `copyIn`/`copyFileOut` copy files between host and the temp dir,
 * and `close` removes the temp dir.
 */
export declare const testIsolated: () => IsolatedSandboxProvider;
//# sourceMappingURL=test-isolated.d.ts.map