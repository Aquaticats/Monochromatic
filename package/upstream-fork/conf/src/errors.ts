/**
 Error classes thrown by the config store.
 
 Each class extends the same base `TypeError` or `Error` that upstream `conf`
 throws, so callers migrating from `conf` keep their `instanceof` checks while
 gaining typed diagnostics. Message texts match upstream `conf` 15.1.0
 verbatim.
 
 Every class is named after itself. The `Error` subclasses take that name from
 `NamedError`; the `TypeError` subclasses cannot, because upstream `conf`
 throws `TypeError` there and a caller's `instanceof TypeError` must keep its
 answer, so each takes it from its own class rather than from a repeated
 literal.
 
 @module
 */

import { NamedError, } from '@monochromatic-dev/named-error-core/ts';

/* oxlint-disable unicorn/custom-error-definition -- Every class here is named after its own class: the `Error` subclasses through `NamedError`, the `TypeError` subclasses through their own identifier, so no hand-written literal is left to check; `doc/troubleshooting/oxlint-custom-error-name.md` records why the rule cannot be satisfied here. */

//region Argument errors

/**
 Thrown when a key argument is not a string.
 
 @example
 ```ts
 try {
   config.get(42 as unknown as string);
 }
 catch (error) {
   error instanceof InvalidKeyError; // => true
 }
 ```
 */
export class InvalidKeyError extends TypeError {
  /**
   Builds the error with upstream `conf`'s message text so migrating callers
   see identical diagnostics.
   
   @param message - Full upstream message naming the expected and actual types.
   */
  constructor(message: string,) {
    super(message,);
    this.name = InvalidKeyError.name;
  }
}

/**
 Thrown when a callback argument is not a function.
 
 @example
 ```ts
 try {
   config.onDidAnyChange('nope' as unknown as () => void);
 }
 catch (error) {
   error instanceof InvalidCallbackError; // => true
 }
 ```
 */
export class InvalidCallbackError extends TypeError {
  /**
   Builds the error with upstream `conf`'s message text so migrating callers
   see identical diagnostics.
   
   @param message - Full upstream message naming the expected and actual types.
   */
  constructor(message: string,) {
    super(message,);
    this.name = InvalidCallbackError.name;
  }
}

/**
 Thrown when `set` receives `undefined` as the value to store.
 
 @example
 ```ts
 try {
   config.set({ key: 'theme', value: undefined, });
 }
 catch (error) {
   error instanceof MissingValueError; // => true
 }
 ```
 */
export class MissingValueError extends TypeError {
  /**
   Builds the error with upstream `conf`'s `Use delete() to clear values`
   message text.
   */
  constructor() {
    super('Use `delete()` to clear values',);
    this.name = MissingValueError.name;
  }
}

/**
 Thrown when a write would touch the reserved `__internal__` bookkeeping key.
 
 @example
 ```ts
 try {
   config.set({ key: '__internal__', value: {}, });
 }
 catch (error) {
   error instanceof ReservedKeyError; // => true
 }
 ```
 */
export class ReservedKeyError extends TypeError {
  /**
   Builds the error with upstream `conf`'s message text so migrating callers
   see identical diagnostics.
   */
  constructor() {
    super(
      "Please don't use the __internal__ key, as it's used to manage this module internal operations.",
    );
    this.name = ReservedKeyError.name;
  }
}

/**
 Thrown when a value cannot round-trip through JSON.
 
 @example
 ```ts
 try {
   config.set({ key: 'fn', value: (): void => {}, });
 }
 catch (error) {
   error instanceof UnsupportedValueTypeError; // => true
 }
 ```
 */
export class UnsupportedValueTypeError extends TypeError {
  /**
   Builds the error with upstream `conf`'s message text so migrating callers
   see identical diagnostics.
   
   @param type - `typeof` result of the rejected value, reported verbatim.
   
   @param key - Store key the value was destined for, reported verbatim.
   */
  constructor({
    type,
    key,
  }: {
    readonly type: string;
    readonly key: string;
  },) {
    super(
      `Setting a value of type \`${type}\` for key \`${key}\` is not allowed as it's not supported by JSON`,
    );
    this.name = UnsupportedValueTypeError.name;
  }
}

/**
 Thrown when `appendToArray` targets a key holding a non-array value.
 
 @example
 ```ts
 try {
   config.set({ key: 'items', value: 'not-an-array', });
   config.appendToArray({ key: 'items', value: 'entry', });
 }
 catch (error) {
   error instanceof NonArrayValueError; // => true
 }
 ```
 */
export class NonArrayValueError extends TypeError {
  /**
   Builds the error with upstream `conf`'s message text so migrating callers
   see identical diagnostics.
   
   @param key - Store key whose current value is not an array.
   */
  constructor({ key, }: { readonly key: string; },) {
    super(`The key \`${key}\` is already set to a non-array value`,);
    this.name = NonArrayValueError.name;
  }
}

//endregion Argument errors

//region Option errors

/**
 Thrown when the `encryptionAlgorithm` option names an unsupported algorithm.
 
 @example
 ```ts
 try {
   createConf({ cwd, encryptionAlgorithm: 'rot13' as never, });
 }
 catch (error) {
   error instanceof InvalidEncryptionAlgorithmError; // => true
 }
 ```
 */
export class InvalidEncryptionAlgorithmError extends TypeError {
  /**
   Builds the error with upstream `conf`'s message text so migrating callers
   see identical diagnostics.
   
   @param supported - Algorithm identifiers the store accepts, reported verbatim.
   */
  constructor({ supported, }: { readonly supported: readonly string[]; },) {
    super(`The \`encryptionAlgorithm\` option must be one of: ${supported.join(', ')}`,);
    this.name = InvalidEncryptionAlgorithmError.name;
  }
}

/**
 Thrown when neither `cwd` nor `projectName` resolves a config directory.
 
 @example
 ```ts
 try {
   createConf({});
 }
 catch (error) {
   error instanceof MissingProjectNameError; // => true
 }
 ```
 */
export class MissingProjectNameError extends NamedError {
  /**
   Builds the error with upstream `conf`'s message text so migrating callers
   see identical diagnostics.
   */
  constructor() {
    super('Please specify the `projectName` option.',);
  }
}

/**
 Thrown when `migrations` are configured without a `projectVersion`.
 
 @example
 ```ts
 try {
   createConf({ cwd, migrations: {}, });
 }
 catch (error) {
   error instanceof MissingProjectVersionError; // => true
 }
 ```
 */
export class MissingProjectVersionError extends NamedError {
  /**
   Builds the error with upstream `conf`'s message text so migrating callers
   see identical diagnostics.
   */
  constructor() {
    super('Please specify the `projectVersion` option.',);
  }
}

/**
 Thrown when the `schema` option is not an object.
 
 @example
 ```ts
 try {
   createConf({ cwd, schema: 'nope' as never, });
 }
 catch (error) {
   error instanceof InvalidSchemaError; // => true
 }
 ```
 */
export class InvalidSchemaError extends TypeError {
  /**
   Builds the error with upstream `conf`'s message text so migrating callers
   see identical diagnostics.
   */
  constructor() {
    super('The `schema` option must be an object.',);
    this.name = InvalidSchemaError.name;
  }
}

/**
 Thrown when `rootSchema` carries `properties`, which only `schema` may set.
 
 @example
 ```ts
 try {
   createConf({ cwd, schema: {}, rootSchema: { properties: {}, } as never, });
 }
 catch (error) {
   error instanceof RootSchemaPropertiesError; // => true
 }
 ```
 */
export class RootSchemaPropertiesError extends TypeError {
  /**
   Builds the error with upstream `conf`'s message text so migrating callers
   see identical diagnostics.
   */
  constructor() {
    super('The `rootSchema` option must not contain a `properties` key. Use the `schema` option for properties.',);
    this.name = RootSchemaPropertiesError.name;
  }
}

//endregion Option errors

//region Data errors

/**
 Thrown when stored data violates the configured schema.
 
 @example
 ```ts
 try {
   config.set({ key: 'port', value: 'eighty', });
 }
 catch (error) {
   error instanceof SchemaViolationError; // => true
 }
 ```
 */
export class SchemaViolationError extends NamedError {
  /**
   Builds the error with upstream `conf`'s `Config schema violation: ...`
   message text.
   
   @param violations - Per-property violation texts, joined verbatim.
   */
  constructor({ violations, }: { readonly violations: readonly string[]; },) {
    super(`Config schema violation: ${violations.join('; ')}`,);
  }
}

/**
 Thrown when encrypted data cannot be decrypted with the configured key.
 
 @example
 ```ts
 try {
   createConf({ cwd, encryptionKey: 'wrong-key', encryptionAlgorithm: 'aes-256-gcm', });
 }
 catch (error) {
   error instanceof DecryptionFailedError; // => true
 }
 ```
 */
export class DecryptionFailedError extends NamedError {
  /**
   Builds the error with upstream `conf`'s message text so migrating callers
   see identical diagnostics.
   */
  constructor() {
    super('Failed to decrypt config data.',);
  }
}

/**
 Thrown when an `aes-256-gcm` payload is too short to hold its auth tag.
 
 @example
 ```ts
 try {
   decryptConfigData({ data: truncated, encryptionKey: key, });
 }
 catch (error) {
   error instanceof InvalidAuthenticationTagError; // => true
 }
 ```
 */
export class InvalidAuthenticationTagError extends NamedError {
  /**
   Builds the error with upstream `conf`'s message text so migrating callers
   see identical diagnostics.
   */
  constructor() {
    super('Invalid authentication tag length.',);
  }
}

/**
 Thrown when a migration step fails; the store is restored to its pre-step
 state before this is thrown.
 
 @example
 ```ts
 try {
   createConf({ cwd, projectVersion: '2.0.0', migrations: { '2.0.0': boom, }, });
 }
 catch (error) {
   error instanceof MigrationFailedError; // => true
 }
 ```
 */
export class MigrationFailedError extends NamedError {
  /**
   Builds the error with upstream `conf`'s message text so migrating callers
   see identical diagnostics.
   
   @param reason - Text of the failure that aborted the migration step.
   */
  constructor({ reason, }: { readonly reason: string; },) {
    super(
      `Something went wrong during the migration! Changes applied to the store until this failed migration will be restored. ${reason}`,
    );
  }
}

//endregion Data errors

/* oxlint-enable unicorn/custom-error-definition */
