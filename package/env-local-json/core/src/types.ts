/**
 Public option and result types for the encrypted environment store reader.

 @module
 */

/**
 Caller-supplied environment mapping. A read-only mirror of `process.env`,
 injected so callers and tests never mutate the real process environment.
 */
export type EnvLocalJsonEnvironment = Readonly<NodeJS.ProcessEnv>;

/**
 Options for {@link readEnvLocalJson}.

 @example
 ```ts
 const result = await readEnvLocalJson({
   keys: ['EXA_API_KEY'],
   env: process.env,
 });
 ```
 */
export type ReadEnvLocalJsonOptions = {
  /**
   Environment variable names the caller needs. Only these names can appear
   in the result, and the store is not touched when every one is already
   non-blank in `env`.
   */
  readonly keys: readonly string[];

  /**
   Authoritative store path. When supplied, discovery never runs, and a
   missing or unreadable file raises `EnvLocalJsonPathError`.
   */
  readonly path?: string;

  /**
   Directory the bounded discovery walk starts from. Defaults to
   `process.cwd()`.
   */
  readonly cwd?: string;

  /**
   Environment consulted for the all-keys-present shortcut, identity
   candidates, and the curated child environment. Defaults to
   `process.env` and is never mutated.
   */
  readonly env?: EnvLocalJsonEnvironment;

  /**
   Home directory used for discovery bounds and the mise identity default.
   Defaults to the runtime home directory.
   */
  readonly home?: string;

  /**
   Aborting rejects with the signal's reason and stops discovery or the
   sops child process at the next step boundary.
   */
  readonly signal?: AbortSignal;
};

/**
 Successful reader result.

 `loaded` is `true` only when a store was discovered, encrypted, decrypted,
 and shape-checked; `path` is its canonical realpath. `values` contains only
 requested keys that were absent or blank in the supplied environment and
 present in the store, so `{ ...env, ...values }` preserves environment
 precedence.
 */
export type ReadEnvLocalJsonResult = {
  /**
   Fill-in values keyed by requested environment variable name.
   */
  readonly values: Readonly<Record<string, string>>;

  /**
   Canonical store path; present exactly when `loaded` is `true`.
   */
  readonly path?: string;

  /**
   Whether the store was read. `false` means no store was found or every
   requested key was already supplied by the environment.
   */
  readonly loaded: boolean;
};
