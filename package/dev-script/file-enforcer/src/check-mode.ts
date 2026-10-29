/**
 Check mode: run every generator without writing, and report the files that would change.

 The CLI enables it with `--check`; the write core consults {@link isCheckMode}
 and records drift through {@link recordDrift} instead of touching disk. This is
 the generated-file counterpart of `pnpm install --frozen-lockfile`.

 @module
 */

//region State

/**
 Holder for whether check mode is on; a map avoids a module-root `let`.
 */
const checkModeByKey: Map<'enabled', boolean> = new Map<'enabled', boolean>();

/**
 Destinations whose generated content differs from disk, in first-seen order.
 */
const driftedDestinations: Set<string> = new Set<string>();

//endregion State

//region API

/**
 Turns check mode on for the rest of the process.

 @example
 ```ts
 enableCheckMode();
 ```
 */
export function enableCheckMode(): void {
  checkModeByKey.set(
    'enabled',
    true,
  );
}

/**
 Whether check mode is on.

 @returns true after {@link enableCheckMode}

 @example
 ```ts
 if (isCheckMode()) recordDrift({ dest: './CLAUDE.md', });
 ```
 */
export function isCheckMode(): boolean {
  return checkModeByKey.get('enabled',) === true;
}

/**
 Records a destination whose generated content differs from disk.

 @param dest - destination path as the generator names it

 @example
 ```ts
 recordDrift({ dest: './package/forbidden-strings/cli/Cargo.toml', });
 ```
 */
export function recordDrift({ dest, }: { readonly dest: string; },): void {
  driftedDestinations.add(dest,);
}

/**
 Destinations recorded as drifted so far.

 @returns paths in first-seen order

 @example
 ```ts
 const drifted = driftedPaths();
 ```
 */
export function driftedPaths(): readonly string[] {
  return [...driftedDestinations,];
}

//endregion API
