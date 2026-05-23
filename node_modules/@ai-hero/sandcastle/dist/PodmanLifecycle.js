import { Effect } from "effect";
import { execFile } from "node:child_process";
import { resolve } from "node:path";
import { PodmanError } from "./errors.js";
const podmanExec = (args) => Effect.async((resume) => {
    execFile("podman", args, { maxBuffer: 10 * 1024 * 1024 }, (error, stdout, stderr) => {
        if (error) {
            resume(Effect.fail(new PodmanError({
                message: `podman ${args[0]} failed: ${stderr?.toString() || error.message}`,
            })));
        }
        else {
            resume(Effect.succeed(stdout.toString()));
        }
    });
});
/**
 * Build the sandcastle Podman image.
 *
 * When `containerfile` is provided, uses `podman build -f <containerfile> <cwd>`
 * so COPY instructions resolve relative to the current working directory.
 * Otherwise, uses `podman build <containerfileDir>` (the default .sandcastle/ directory).
 */
export const buildImage = (imageName, containerfileDir, options) => Effect.gen(function* () {
    if (options?.containerfile) {
        yield* podmanExec([
            "build",
            "-t",
            imageName,
            "-f",
            resolve(options.containerfile),
            process.cwd(),
        ]);
    }
    else {
        yield* podmanExec(["build", "-t", imageName, resolve(containerfileDir)]);
    }
});
/**
 * Remove a Podman image.
 */
export const removeImage = (imageName) => Effect.gen(function* () {
    yield* podmanExec(["rmi", imageName]);
});
//# sourceMappingURL=PodmanLifecycle.js.map