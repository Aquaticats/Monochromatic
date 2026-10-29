/**
 Scoped global and `process` overrides for the coverage driver: install a
 value under a global name, or replace a `process` property descriptor,
 and restore the previous state on dispose. Used to stage runtimes the
 sinks probe for (Deno, Bun, a browser `window`) and hostile hosts whose
 `env`, `argv`, or `stderr` throw on access.

 @module
 */

/**
 Installs `value` under `name` on `globalThis` for a scope; disposing
 restores the previous descriptor or removes the property when none
 existed. `undefined` as the value stages an absent capability while
 keeping the property present.

 @param name - Global name.

 @param value - Value to expose.

 @returns Disposable restoring the previous state.

 @example
 ```ts
 using _deno = installGlobalValue({ name: 'Deno', value: {} });
 ```
 */
export function installGlobalValue({
  name,
  value,
}: {
  readonly name: string;
  readonly value?: unknown;
},): Disposable {
  /**
   Previous descriptor, absent when the global did not exist.
   */
  const original = Object.getOwnPropertyDescriptor(
    globalThis,
    name,
  );
  Object.defineProperty(
    globalThis,
    name,
    {
      configurable: true,
      value,
      writable: true,
    },
  );
  return {
    [Symbol.dispose](): void {
      if (original === undefined)
        Reflect.deleteProperty(
          globalThis,
          name,
        );
      else
        Object.defineProperty(
          globalThis,
          name,
          original,
        );
    },
  };
}

/**
 Replaces one `process` property with a getter that throws for a scope, so
 a probe reading it takes its failure path; disposing restores the
 original descriptor.

 @param name - Property of `process` to poison.

 @returns Disposable restoring the original descriptor.

 @throws Error when `process` has no own descriptor for `name`, because
 restoring would then be undefined.

 @example
 ```ts
 using _hostile = poisonProcessProperty({ name: 'argv' });
 ```
 */
export function poisonProcessProperty({ name, }: { readonly name: 'argv' | 'env' | 'stderr'; },): Disposable {
  /**
   Original descriptor to restore.
   */
  const original = Object.getOwnPropertyDescriptor(
    process,
    name,
  );
  if (original === undefined)
    throw new Error(`process.${name} has no own descriptor to restore`,);
  Object.defineProperty(
    process,
    name,
    {
      configurable: true,
      get(): never {
        throw new Error(`process.${name} is unavailable in this host`,);
      },
    },
  );
  return {
    [Symbol.dispose](): void {
      Object.defineProperty(
        process,
        name,
        original,
      );
    },
  };
}
