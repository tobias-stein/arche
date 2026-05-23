import { FileSystem } from "@effect/platform";
import { Effect } from "effect";
import { PromptError } from "./errors.js";
export interface ResolvePromptOptions {
    readonly prompt?: string;
    readonly promptFile?: string;
}
/**
 * The resolved prompt text plus the source it came from.
 *
 * - `inline`: came from `prompt: "..."` — delivered to the agent verbatim.
 *   No prompt argument substitution, no prompt expansion, no built-in args.
 * - `template`: came from `promptFile` — eligible for `{{KEY}}` substitution
 *   and `` !`command` `` expansion.
 */
export interface ResolvedPrompt {
    readonly text: string;
    readonly source: "inline" | "template";
}
export declare const resolvePrompt: (options: ResolvePromptOptions) => Effect.Effect<ResolvedPrompt, PromptError, FileSystem.FileSystem>;
//# sourceMappingURL=PromptResolver.d.ts.map