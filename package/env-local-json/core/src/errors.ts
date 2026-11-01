/**
 Named failures raised by the encrypted environment store reader and writer.

 Every message names the store path, key names, or candidates only. Decrypted
 and encrypted payloads, sops stdout, and sops stderr are never attached, so
 no error can carry a secret value.

 @module
 */

import dedent from 'string-dedent';

//region Path failures

/**
 Why an explicit store path could not be used.
 */
export type EnvLocalJsonPathFailureReason =
  | 'missing'
    | 'not-a-file'
    | 'not-a-directory'
    | 'unreadable'
    | 'unresolvable';

/**
 Plain-language text for each path failure reason.
 */
const PATH_FAILURE_TEXT: Readonly<Record<EnvLocalJsonPathFailureReason, string>> = {
  'missing': 'does not exist',
  'not-a-file': 'is not a regular file',
  'not-a-directory': 'is not a directory',
  'unreadable': 'cannot be read',
  'unresolvable': 'cannot be resolved to a canonical path',
};

/**
 Raised when an explicitly supplied store path is missing or unreadable.

 @example
 ```ts
 try {
   await readEnvLocalJson({ keys: ['A'], path: '/no/store.json' });
 }
 catch (error) {
   if (error instanceof EnvLocalJsonPathError) console.log(error.path);
 }
 ```
 */
export class EnvLocalJsonPathError extends Error {
  /**
   Store path that failed.
   */
  public readonly path: string;

  /**
   Why the path could not be used.
   */
  public readonly reason: EnvLocalJsonPathFailureReason;

  /**
   Creates one explicit-path failure.

   @param path - explicitly supplied store path

   @param reason - why the path could not be used

   @example
   ```ts
   throw new EnvLocalJsonPathError({ path: '/no/store.json', reason: 'missing' });
   ```
   */
  public constructor({
    path,
    reason,
  }: {
    readonly path: string;
    readonly reason: EnvLocalJsonPathFailureReason;
  },) {
    super(`env-local-json path ${path} ${PATH_FAILURE_TEXT[reason]}`,);
    this.name = 'EnvLocalJsonPathError';
    this.path = path;
    this.reason = reason;
  }
}

//endregion Path failures

//region Discovery and read failures

/**
 Raised when a discovered store exists but cannot be read.

 @example
 ```ts
 try {
   await readEnvLocalJson({ keys: ['A'] });
 }
 catch (error) {
   if (error instanceof EnvLocalJsonReadError) console.log(error.path);
 }
 ```
 */
export class EnvLocalJsonReadError extends Error {
  /**
   Canonical store path that could not be read.
   */
  public readonly path: string;

  /**
   Creates one discovered-store read failure.

   @param path - canonical store path

   @example
   ```ts
   throw new EnvLocalJsonReadError({ path: '/repo/.env.local.json' });
   ```
   */
  public constructor({ path, }: { readonly path: string; },) {
    super(`env store ${path} could not be read`,);
    this.name = 'EnvLocalJsonReadError';
    this.path = path;
  }
}

/**
 Raised when the store is not a sops-encrypted envelope. Plaintext JSON and
 malformed ciphertext both fail here, before any identity lookup or sops call.

 @example
 ```ts
 try {
   await readEnvLocalJson({ keys: ['A'], path: '/repo/.env.local.json' });
 }
 catch (error) {
   if (error instanceof EnvLocalJsonPlaintextError) console.log(error.path);
 }
 ```
 */
export class EnvLocalJsonPlaintextError extends Error {
  /**
   Store path that failed the encrypted-envelope check.
   */
  public readonly path: string;

  /**
   Creates one plaintext-store rejection.

   @param path - store path that lacked sops metadata

   @example
   ```ts
   throw new EnvLocalJsonPlaintextError({ path: '/repo/.env.local.json' });
   ```
   */
  public constructor({ path, }: { readonly path: string; },) {
    super(
      `env store ${path} is not sops-encrypted; plaintext stores are rejected`,
    );
    this.name = 'EnvLocalJsonPlaintextError';
    this.path = path;
  }
}

//endregion Discovery and read failures

//region Identity and tooling failures

/**
 Raised when no encryption identity candidate can be used.

 @example
 ```ts
 try {
   await readEnvLocalJson({ keys: ['A'] });
 }
 catch (error) {
   if (error instanceof EnvLocalJsonMissingIdentityError) console.log(error.candidates);
 }
 ```
 */
export class EnvLocalJsonMissingIdentityError extends Error {
  /**
   Every candidate consulted, in order, with its rejection reason.
   */
  public readonly candidates: readonly string[];

  /**
   Creates one missing-identity failure.

   @param candidates - ordered candidate descriptions

   @example
   ```ts
   throw new EnvLocalJsonMissingIdentityError({ candidates: ['SOPS_AGE_KEY (unset)'] });
   ```
   */
  public constructor({
    candidates,
  }: {
    readonly candidates: readonly string[];
  },) {
    super(
      dedent`
        no usable sops/age identity found. Candidates, in precedence order:
        ${candidates.map(function renderCandidate(candidate,): string {
          return `- ${candidate}`;
        },)
          .join('\n',)}
        Set one of these and retry.
      `,
    );
    this.name = 'EnvLocalJsonMissingIdentityError';
    this.candidates = candidates;
  }
}

/**
 Raised when the `sops` executable cannot be found on the curated `PATH`.

 @example
 ```ts
 try {
   await readEnvLocalJson({ keys: ['A'], env: { PATH: '/empty' } });
 }
 catch (error) {
   if (error instanceof EnvLocalJsonSopsMissingError) console.log(error.message);
 }
 ```
 */
export class EnvLocalJsonSopsMissingError extends Error {
  /**
   Creates one missing-sops failure.
   */
  public constructor() {
    super(
      dedent`
        sops executable not found on PATH, so the encrypted env store cannot be decrypted.
        Install sops and age (for example, 'mise install sops age'), then retry.
      `,
    );
    this.name = 'EnvLocalJsonSopsMissingError';
  }
}

//endregion Identity and tooling failures

//region Decryption and shape failures

/**
 How the sops child process failed, without exposing its output.
 */
export type EnvLocalJsonDecryptFailureKind =
  | 'exit'
    | 'signal'
    | 'timeout'
    | 'aborted'
    | 'spawn'
    | 'output-limit'
    | 'unknown';

/**
 Raised when sops cannot decrypt the store.

 @example
 ```ts
 try {
   await readEnvLocalJson({ keys: ['A'] });
 }
 catch (error) {
   if (error instanceof EnvLocalJsonDecryptError) console.log(error.kind);
 }
 ```
 */
export class EnvLocalJsonDecryptError extends Error {
  /**
   Store path sops was asked to decrypt.
   */
  public readonly path: string;

  /**
   Failure class, never sops output.
   */
  public readonly kind: EnvLocalJsonDecryptFailureKind;

  /**
   Numeric exit code when sops exited normally with a failure.
   */
  public readonly exitCode?: number;

  /**
   Creates one decryption failure.

   @param path - store path passed to sops

   @param kind - failure class

   @param exitCode - numeric exit code when one is known

   @example
   ```ts
   throw new EnvLocalJsonDecryptError({ path: '/repo/.env.local.json', kind: 'exit', exitCode: 128 });
   ```
   */
  public constructor({
    path,
    kind,
    exitCode,
  }: {
    readonly path: string;
    readonly kind: EnvLocalJsonDecryptFailureKind;
    readonly exitCode?: number;
  },) {
    /**
     Suffix naming the failure class and any numeric exit code.
     */
    const detail = exitCode === undefined
      ? kind
      : `${kind} ${String(exitCode,)}`;
    super(`sops could not decrypt env store ${path} (${detail})`,);
    this.name = 'EnvLocalJsonDecryptError';
    this.path = path;
    this.kind = kind;
    if (exitCode !== undefined)
      this.exitCode = exitCode;
  }
}

/**
 Why decrypted store content failed validation.
 */
export type EnvLocalJsonShapeFailureReason =
  | 'not-json'
    | 'not-object'
    | 'non-string'
    | 'blank'
    | 'encrypted-token';

/**
 Plain-language text for each shape failure reason.
 */
const SHAPE_FAILURE_TEXT: Readonly<Record<EnvLocalJsonShapeFailureReason, string>> = {
  'not-json': 'is not valid JSON',
  'not-object': 'is not a JSON object',
  'non-string': 'is not a string',
  'blank': 'is blank',
  'encrypted-token': 'still looks sops-encrypted',
};

/**
 Raised when decrypted content or a caller-supplied value has the wrong shape.

 @example
 ```ts
 try {
   await readEnvLocalJson({ keys: ['A'] });
 }
 catch (error) {
   if (error instanceof EnvLocalJsonShapeError) console.log(error.key);
 }
 ```
 */
export class EnvLocalJsonShapeError extends Error {
  /**
   Store path whose content failed validation.
   */
  public readonly path: string;

  /**
   Failure class.
   */
  public readonly reason: EnvLocalJsonShapeFailureReason;

  /**
   Key whose value failed validation; absent for whole-document failures.
   */
  public readonly key?: string;

  /**
   Creates one shape failure.

   @param path - store path under validation

   @param reason - failure class

   @param key - key whose value failed validation

   @example
   ```ts
   throw new EnvLocalJsonShapeError({ path: '/repo/.env.local.json', reason: 'non-string', key: 'A' });
   ```
   */
  public constructor({
    path,
    reason,
    key,
  }: {
    readonly path: string;
    readonly reason: EnvLocalJsonShapeFailureReason;
    readonly key?: string;
  },) {
    /**
     Detail naming the failing key when one is known.
     */
    const detail = key === undefined
      ? ''
      : ` for key ${key}`;
    super(
      `decrypted env store ${path} ${SHAPE_FAILURE_TEXT[reason]}${detail}`,
    );
    this.name = 'EnvLocalJsonShapeError';
    this.path = path;
    this.reason = reason;
    if (key !== undefined)
      this.key = key;
  }
}

//endregion Decryption and shape failures

//region Write failures

/**
 Why an atomic store write failed.
 */
export type EnvLocalJsonWriteFailureReason =
  | 'invalid-key'
    | 'blank-value'
    | 'sops-exit'
    | 'timeout'
    | 'terminated'
    | 'spawn'
    | 'output-limit'
    | 'permissions'
    | 'rename';

/**
 Plain-language text for each write failure reason.
 */
const WRITE_FAILURE_TEXT: Readonly<Record<EnvLocalJsonWriteFailureReason, string>> = {
  'invalid-key': 'is not a valid environment variable name',
  'blank-value': 'is blank; the store reader rejects blank values',
  'sops-exit': 'sops set exited with a failure',
  'timeout': 'sops set timed out',
  'terminated': 'sops set was terminated',
  'spawn': 'sops could not be started',
  'output-limit': 'sops set produced too much output',
  'permissions': 'the temporary store permissions could not be set',
  'rename': 'the temporary store could not replace the store atomically',
};

/**
 Raised by the internal atomic writer. Exported through the testing subpath
 so callers can classify failures without widening the reader index.

 @example
 ```ts
 try {
   await writeEnvLocalJsonValues({ path, values: { A: 'x' }, env, home });
 }
 catch (error) {
   if (error instanceof EnvLocalJsonWriteError) console.log(error.reason);
 }
 ```
 */
export class EnvLocalJsonWriteError extends Error {
  /**
   Store path the write targeted.
   */
  public readonly path: string;

  /**
   Failure class.
   */
  public readonly reason: EnvLocalJsonWriteFailureReason;

  /**
   Key whose value or name failed validation.
   */
  public readonly key?: string;

  /**
   Creates one write failure.

   @param path - store path the write targeted

   @param reason - failure class

   @param key - key that failed validation

   @example
   ```ts
   throw new EnvLocalJsonWriteError({ path: '/repo/.env.local.json', reason: 'invalid-key', key: 'A B' });
   ```
   */
  public constructor({
    path,
    reason,
    key,
  }: {
    readonly path: string;
    readonly reason: EnvLocalJsonWriteFailureReason;
    readonly key?: string;
  },) {
    /**
     Detail naming the failing key when one is known.
     */
    const detail = key === undefined
      ? ''
      : ` for key ${key}`;
    super(`env store write ${path} failed: ${WRITE_FAILURE_TEXT[reason]}${detail}`,);
    this.name = 'EnvLocalJsonWriteError';
    this.path = path;
    this.reason = reason;
    if (key !== undefined)
      this.key = key;
  }
}

//endregion Write failures
