/**
 Shared host-runtime detection for web storage quota heuristics.
 
 The Web Storage API exposes no way to read a store's quota, so the
 per-storage-area quota modules pair this detection with their own tables of
 measured defaults. Detection is by host global rather than user-agent
 string: Deno and Bun both shim `process` with a Node-compatible
 `process.versions.node`, so their own globals are tested before the Node
 check to avoid misclassifying them.
 
 @module
 */

/**
 Host runtimes distinguishable by global probes, plus `unknown` for
 everything else so callers fall back to uncapped reactive eviction.
 */
export type WebStorageRuntime = 'browser' | 'bun' | 'deno' | 'node' | 'unknown';

/**
 Detects the current host runtime for web storage quota lookups.
 
 A `node` result also covers Node-embedding hosts such as Electron, whose
 renderer exposes `process.versions.node` alongside a DOM; callers that need
 to tell those apart check `'document' in globalThis` themselves.
 
 @returns Detected runtime, or `unknown` when no marker global matches.
 
 @example
 ```ts
 const quota = RUNTIME_QUOTA_CHARS[detectWebStorageRuntime()];
 ```
 */
export function detectWebStorageRuntime(): WebStorageRuntime {
  if ('Deno' in globalThis)
    return 'deno';

  if ('Bun' in globalThis)
    return 'bun';

  if (((typeof process) !== 'undefined') && ((typeof process.versions
    ?.node) === 'string'))
    return 'node';

  if ('document' in globalThis)
    return 'browser';

  return 'unknown';
}
