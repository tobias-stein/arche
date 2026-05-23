import { Effect } from "effect";
import { Display } from "./Display.js";
import { PromptError, PromptExpansionTimeoutError } from "./errors.js";
import type { ExecError } from "./errors.js";
import type { SandboxService } from "./SandboxFactory.js";
/**
 * @internal
 * Marker inserted between `!` and the opening backtick for shell blocks that
 * appear in the raw template. The preprocessor only executes marked blocks, so
 * `!`...`` patterns arriving via argument substitution are treated as data.
 * Used only by `substitutePromptArgs`; not part of the public API.
 */
export declare const SHELL_BLOCK_MARKER = "\u0001";
export declare const preprocessPrompt: (prompt: string, sandbox: SandboxService, cwd: string) => Effect.Effect<string, ExecError | PromptError | PromptExpansionTimeoutError, Display>;
//# sourceMappingURL=PromptPreprocessor.d.ts.map