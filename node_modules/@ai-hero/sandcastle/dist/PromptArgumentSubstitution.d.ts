import { Effect } from "effect";
import { Display } from "./Display.js";
import { PromptError } from "./errors.js";
/**
 * A map of named values used for prompt argument substitution.
 * Each key corresponds to a `{{KEY}}` placeholder in the prompt; the value
 * replaces it before the prompt is passed to the agent.
 */
export type PromptArgs = Record<string, string | number | boolean>;
/**
 * Prompt argument keys that Sandcastle injects automatically.
 * These cannot be overridden via `promptArgs`.
 */
export declare const BUILT_IN_PROMPT_ARG_KEYS: readonly ["SOURCE_BRANCH", "TARGET_BRANCH"];
/**
 * Validates that the user has not provided any built-in prompt argument keys.
 * Fails with a PromptError if any built-in key is found in `args`.
 */
/**
 * Fails if the user passed any `promptArgs` alongside an inline prompt.
 * Inline prompts are delivered to the agent verbatim, so `promptArgs` would
 * silently do nothing. An empty object is treated as absent.
 */
export declare const validateNoArgsWithInlinePrompt: (args: PromptArgs) => Effect.Effect<void, PromptError, never>;
export declare const validateNoBuiltInArgOverride: (args: PromptArgs) => Effect.Effect<void, PromptError, never>;
/**
 * Scans a prompt for `{{KEY}}` placeholders and returns the keys that are
 * missing from `providedArgs`, excluding built-in keys.
 */
export declare const findMissingPromptArgKeys: (prompt: string, providedArgs: PromptArgs) => string[];
export declare const substitutePromptArgs: (prompt: string, args: PromptArgs, silentKeys?: ReadonlySet<string> | undefined) => Effect.Effect<string, PromptError, Display>;
//# sourceMappingURL=PromptArgumentSubstitution.d.ts.map