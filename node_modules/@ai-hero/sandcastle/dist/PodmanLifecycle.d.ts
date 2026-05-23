import { Effect } from "effect";
import { PodmanError } from "./errors.js";
/**
 * Build the sandcastle Podman image.
 *
 * When `containerfile` is provided, uses `podman build -f <containerfile> <cwd>`
 * so COPY instructions resolve relative to the current working directory.
 * Otherwise, uses `podman build <containerfileDir>` (the default .sandcastle/ directory).
 */
export declare const buildImage: (imageName: string, containerfileDir: string, options?: {
    readonly containerfile?: string | undefined;
} | undefined) => Effect.Effect<void, PodmanError, never>;
/**
 * Remove a Podman image.
 */
export declare const removeImage: (imageName: string) => Effect.Effect<void, PodmanError, never>;
//# sourceMappingURL=PodmanLifecycle.d.ts.map