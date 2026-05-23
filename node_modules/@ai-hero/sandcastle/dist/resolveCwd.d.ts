import { FileSystem } from "@effect/platform";
import { Effect } from "effect";
declare const CwdError_base: new <A extends Record<string, any> = {}>(args: import("effect/Types").VoidIfEmpty<{ readonly [P in keyof A as P extends "_tag" ? never : P]: A[P]; }>) => import("effect/Cause").YieldableError & {
    readonly _tag: "CwdError";
} & Readonly<A>;
/** The provided `cwd` path does not exist or is not a directory. */
export declare class CwdError extends CwdError_base<{
    readonly message: string;
    readonly cwd: string;
}> {
}
/**
 * Resolve an optional `cwd` string to an absolute, validated host repo directory.
 *
 * - `undefined` → `process.cwd()` (resolved to absolute).
 * - Relative path → resolved against `process.cwd()`.
 * - Absolute path → passed through.
 *
 * Stats the result via Effect's `FileSystem`; fails with {@link CwdError}
 * when the path is missing or is not a directory.
 */
export declare const resolveCwd: (cwd: string | undefined) => Effect.Effect<string, CwdError, FileSystem.FileSystem>;
export {};
//# sourceMappingURL=resolveCwd.d.ts.map