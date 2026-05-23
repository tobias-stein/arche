import { FileSystem } from "@effect/platform";
import { Effect } from "effect";
import { PromptError } from "./errors.js";
export const resolvePrompt = (options) => {
    const { prompt, promptFile } = options;
    if (prompt !== undefined && promptFile !== undefined) {
        return Effect.fail(new PromptError({
            message: "Cannot provide both --prompt and --prompt-file",
        }));
    }
    if (prompt !== undefined) {
        return Effect.succeed({ text: prompt, source: "inline" });
    }
    if (promptFile === undefined) {
        return Effect.fail(new PromptError({
            message: "Must provide either prompt or promptFile. Pass prompt: '...' or promptFile: './.sandcastle/prompt.md' to run().",
        }));
    }
    return Effect.gen(function* () {
        const fs = yield* FileSystem.FileSystem;
        const text = yield* fs.readFileString(promptFile).pipe(Effect.catchAll((e) => Effect.fail(new PromptError({
            message: `Failed to read prompt from ${promptFile}: ${e}`,
        }))));
        return { text, source: "template" };
    });
};
//# sourceMappingURL=PromptResolver.js.map