/**
 Public store type for the config store.
 
 Member shapes mirror upstream `conf` 15.1.0's `Conf` instance with the
 call shapes this repository requires:
 multi-argument members take one destructured object,
 `set` distinguishes its single and multi forms by the `key` and `values`
 fields,
 and members are property signatures carrying overloaded call signatures.
 
 @module
 */

import type {
  DotNotationKeyOf,
  DotNotationValueOf,
  OnDidAnyChangeCallback,
  OnDidChangeCallback,
  PartialObjectDeep,
  Unsubscribe,
} from './types.ts';

//region Store type

/**
 One config store:
 the frozen object `createConf` returns.
 
 @example
 ```ts
 const config: Conf<{ theme: string, }> = createConf({
   projectName: 'foo',
 });
 ```
 */
export type Conf<T extends Record<string, unknown> = Record<string, unknown>> = {
  /**
   Path of the config file this store persists to.
   */
  readonly path: string;
  /**
   Dispatches `change` whenever the config changes;
   `onDidChange` and `onDidAnyChange` subscribe through it.
   */
  readonly events: EventTarget;
  /**
   Item count of the user-visible store.
   */
  readonly size: number;
  /**
   The whole store:
   read it for every item,
   assign it to replace everything.
   */
  store: T;

  /**
   Gets one item:
   by key,
   by dotted path,
   or with a `defaultValue` for missing keys.
   */
  get: {
    <Key extends keyof T>(key: Key,): T[Key];
    <Key extends keyof T>(input: {
      readonly key: Key;
      readonly defaultValue: Required<T>[Key];
    },): Required<T>[Key];
    <Key extends DotNotationKeyOf<T>>(key: Key,): DotNotationValueOf<T, Key>;
    <Key extends DotNotationKeyOf<T>>(input: {
      readonly key: Key;
      readonly defaultValue: NonNullable<DotNotationValueOf<T, Key>>;
    },): NonNullable<DotNotationValueOf<T, Key>>;
    <Value = unknown>(keyOrOptions: string | {
      readonly key: string;
      readonly defaultValue?: Value;
    },): Value;
  };

  /**
   Sets one item under `key`,
   or multiple items under `values`.
   */
  set: {
    <Key extends keyof T>(input: {
      readonly key: Key;
      readonly value?: T[Key];
    },): void;
    <Key extends DotNotationKeyOf<T>>(input: {
      readonly key: Key;
      readonly value?: DotNotationValueOf<T, Key>;
    },): void;
    (input: {
      readonly key: string;
      readonly value?: unknown;
    } | {
      readonly values: PartialObjectDeep<T>;
    },): void;
  };

  /**
   Reports whether an item exists at a key or dotted path.
   */
  has: (key: keyof T | DotNotationKeyOf<T>,) => boolean;

  /**
   Appends one item to a key's or dotted path's array value.
   */
  appendToArray: {
    <Key extends keyof T>(input: {
      readonly key: Key;
      readonly value: T[Key] extends readonly (infer U)[] ? U : unknown;
    },): void;
    <Key extends DotNotationKeyOf<T>>(input: {
      readonly key: Key;
      readonly value: DotNotationValueOf<T, Key> extends readonly (infer U)[] ? U : unknown;
    },): void;
    (input: {
      readonly key: string;
      readonly value: unknown;
    },): void;
  };

  /**
   Restores keys or dotted paths to their default values.
   */
  reset: (input: {
    readonly keys: readonly (keyof T | DotNotationKeyOf<T>)[];
  },) => void;

  /**
   Deletes one item at a key or dotted path.
   */
  delete: (key: keyof T | DotNotationKeyOf<T>,) => void;

  /**
   Deletes all items,
   restoring defaults.
   */
  clear: () => void;

  /**
   Subscribes to changes of one key's or dotted path's value.
   */
  onDidChange: {
    <Key extends keyof T>(input: {
      readonly key: Key;
      readonly callback: OnDidChangeCallback<T[Key]>;
    },): Unsubscribe;
    <Key extends DotNotationKeyOf<T>>(input: {
      readonly key: Key;
      readonly callback: OnDidChangeCallback<DotNotationValueOf<T, Key>>;
    },): Unsubscribe;
    (input: {
      readonly key: string;
      readonly callback: OnDidChangeCallback<unknown>;
    },): Unsubscribe;
  };

  /**
   Subscribes to whole-store changes.
   */
  onDidAnyChange: (callback: OnDidAnyChangeCallback<T>,) => Unsubscribe;

  /**
   Closes the file watcher when one exists.
   */
  closeWatcher: () => void;

  /**
   Iterates the user-visible store's entries.
   */
  [Symbol.iterator]: () => IterableIterator<[
    keyof T,
    T[keyof T]
  ]>;
};

//endregion Store type
