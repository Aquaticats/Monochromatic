/**
 Shared callback,
 serialization,
 and dot-notation types for the config store.
 
 Callback shapes take one destructured object parameter instead of upstream
 `conf`'s positional pairs,
 matching this repository's named-parameters standard.
 Dot-notation mapping types mirror upstream `conf` 15.1.0 so typed stores
 infer the same nested access; they are the one place `undefined` unions and
 added-optionality mapped types are tolerated,
 under scoped lint disables
 justified by that mirror.
 
 @module
 */

import type { Conf, } from './conf.ts';

//region Serialization

/**
 Serializes the store object to the UTF-8 text written to the config file.
 
 @example
 ```ts
 const serialize: Serialize<Record<string, unknown>> = function serialize(value): string {
   return JSON.stringify(value);
 };
 ```
 */
export type Serialize<T> = (value: T,) => string;

/**
 Deserializes config file text back into the store object.
 
 @example
 ```ts
 const deserialize: Deserialize<Record<string, unknown>> = function deserialize(text): Record<string, unknown> {
   return JSON.parse(text) as Record<string, unknown>;
 };
 ```
 */
export type Deserialize<T> = (text: string,) => T;

//endregion Serialization

//region Change notifications

/**
 One watched-key change reported to `onDidChange` subscribers.
 
 `newValue` is omitted when the key was deleted and `oldValue` is omitted the
 first time a key is set.
 
 @example
 ```ts
 const change: ValueChange<string> = { newValue: 'dark', };
 ```
 */
export type ValueChange<T> = {
  /**
   Value after the change; absent when the key was deleted.
   */
  readonly newValue?: T;
  /**
   Value before the change; absent before the key's first set.
   */
  readonly oldValue?: T;
};

/**
 One whole-store change reported to `onDidAnyChange` subscribers.
 
 @example
 ```ts
 const change: StoreChange<Record<string, unknown>> = {
   newValue: { theme: 'dark', },
   oldValue: { theme: 'light', },
 };
 ```
 */
export type StoreChange<T> = {
  /**
   Store contents after the change.
   */
  readonly newValue: Readonly<T>;
  /**
   Store contents before the change.
   */
  readonly oldValue: Readonly<T>;
};

/**
 Receives the new and previous values of one watched key.
 
 @example
 ```ts
 const callback: OnDidChangeCallback<string> = function onThemeChange(change): void {
   console.log(change.newValue);
 };
 ```
 */
export type OnDidChangeCallback<T> = (change: ValueChange<T>,) => void;

/**
 Receives the new and previous whole store on any change.
 
 @example
 ```ts
 const callback: OnDidAnyChangeCallback<Record<string, unknown>> = function onStoreChange(change): void {
   console.log(Object.keys(change.newValue));
 };
 ```
 */
export type OnDidAnyChangeCallback<T> = (change: StoreChange<T>,) => void;

/**
 Stops a change subscription created by `onDidChange` or `onDidAnyChange`.
 
 @example
 ```ts
 const unsubscribe: Unsubscribe = config.onDidAnyChange({ callback: handle, });
 unsubscribe();
 ```
 */
export type Unsubscribe = () => void;

//endregion Change notifications

//region Migrations

/**
 Records the surrounding context of one migration step.
 
 @example
 ```ts
 const context: BeforeEachMigrationContext = {
   fromVersion: '0.9.0',
   toVersion: '1.0.0',
   finalVersion: '1.0.0',
   versions: ['1.0.0'],
 };
 ```
 */
export type BeforeEachMigrationContext = {
  /**
   Version the step migrates away from.
   */
  readonly fromVersion: string;
  /**
   Version this step migrates to.
   */
  readonly toVersion: string;
  /**
   Project version the whole pass aims for.
   */
  readonly finalVersion: string;
  /**
   Every version that runs in this migration pass.
   */
  readonly versions: readonly string[];
};

/**
 Runs before each migration step,
 for logging,
 preparation,
 or cleanup.
 
 @example
 ```ts
 const beforeEachMigration: BeforeEachMigrationCallback<Record<string, unknown>> = function logStep({ context, }): void {
   console.log(context.fromVersion);
 };
 ```
 */
export type BeforeEachMigrationCallback<T extends Record<string, unknown>> = (details: {
  /**
   Store the step will mutate.
   */
  readonly store: Conf<T>;
  /**
   Surrounding context of the step about to run.
   */
  readonly context: BeforeEachMigrationContext;
},) => void;

/**
 Version-to-handler map: each key is a concrete version or semver range,
 each value migrates the store toward it.
 
 @example
 ```ts
 const migrations: Migrations<Record<string, unknown>> = {
   '1.0.0': function toV1(store): void {
     store.delete('debugPhase');
   },
 };
 ```
 */
export type Migrations<T extends Record<string, unknown>> = Readonly<Record<string, (store: Conf<T>,) => void>>;

//endregion Migrations

//region Dot-notation mapping

/**
 Every dotted path addressing a property of `T`,
 including nested object properties.
 
 Mirrors upstream `conf` 15.1.0.
 
 @example
 ```ts
 type Paths = DotNotationKeyOf<{ a: { b: string }, }>;
 // => 'a' | 'a.b'
 ```
 */
export type DotNotationKeyOf<T extends Record<string, unknown>> = {
  [K in keyof Required<T>]: K extends string
    ? Required<T>[K] extends Record<string, unknown>
      ? K | `${K}.${DotNotationKeyOf<Required<T>[K]>}`
      : K
    : never
}[keyof T];

/* oxlint-disable no-restricted-syntax/no-nullish-union -- Mirrors upstream `conf` 15.1.0's DotNotationValueOf, where optional nested properties legitimately yield undefined; modeling that as `T | undefined` is the upstream contract typed stores compile against. */

/**
 Value type behind one dotted path of `T`,
 `undefined` where the path crosses an optional property.
 
 Mirrors upstream `conf` 15.1.0.
 
 @example
 ```ts
 type Value = DotNotationValueOf<{ a: { b: string }, }, 'a.b'>;
 // => string
 ```
 */
export type DotNotationValueOf<T extends Record<string, unknown>, K extends DotNotationKeyOf<T>> =
  K extends `${infer Head}.${infer Tail}`
    ? Head extends keyof T
      ? T[Head] extends Record<string, unknown>
        ? Tail extends DotNotationKeyOf<T[Head]>
          ? DotNotationValueOf<T[Head], Tail>
          : never
        : Required<T>[Head] extends Record<string, unknown>
          ? Tail extends DotNotationKeyOf<Required<T>[Head]>
            ? DotNotationValueOf<Required<T>[Head], Tail> | undefined
            : never
          : never
      : never
    : K extends keyof T
      ? T[K]
      : never;

/* oxlint-enable no-restricted-syntax/no-nullish-union */

//endregion Dot-notation mapping

//region Deep partial

/**
 Types whose identity survives `PartialObjectDeep` recursion untouched.
 
 @example
 ```ts
 const date: ImmutablePrimitives = new Date();
 ```
 */
export type ImmutablePrimitives = Date | RegExp | URL | Error;

/* oxlint-disable no-restricted-syntax/no-optional-escape -- Mirrors upstream `conf` 15.1.0's PartialObjectDeep, whose added optionality is exactly what multi-item `set({ values })` callers compile against. */

/**
 `T` with every nested object property made optional at every depth,
 while maps,
 sets,
 arrays,
 and functions keep their shapes.
 
 Mirrors upstream `conf` 15.1.0.
 
 @example
 ```ts
 const partial: PartialObjectDeep<{ a: { b: string, }, }> = { a: {}, };
 ```
 */
export type PartialObjectDeep<T> =
  T extends ImmutablePrimitives
    ? T
    : T extends Map<infer K, infer V>
      ? Map<PartialObjectDeep<K>, PartialObjectDeep<V>>
      : T extends Set<infer U>
        ? Set<PartialObjectDeep<U>>
        : T extends (infer U)[]
          ? PartialObjectDeep<U>[]
          : T extends (...args: never[]) => unknown
            ? T
            : T extends Record<string, unknown>
              ? {[K in keyof T]?: PartialObjectDeep<T[K]>}
              : T;

/* oxlint-enable no-restricted-syntax/no-optional-escape */

//endregion Deep partial
