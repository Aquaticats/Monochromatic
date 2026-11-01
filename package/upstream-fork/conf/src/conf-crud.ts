/**
 Store mutation and read methods for the config store.
 
 Groups upstream `conf`'s `get`,
 `set`,
 `has`,
 `appendToArray`,
 `reset`,
 `delete`,
 and `clear` implementations behind one factory so the store object in
 `conf.ts` stays a thin wiring layer.
 
 @module
 */

import {
  InvalidKeyError,
  MissingValueError,
  NonArrayValueError,
  ReservedKeyError,
} from './errors.ts';
import { containsReservedKey, } from './internal-key.ts';
import { checkValueType, } from './value-type.ts';
import {
  createPlainObject,
  getStoreValue,
  hasStoreValue,
  withStoreValue,
  withoutStoreValue,
} from './store-access.ts';
import type { StoreFile, } from './store-file.ts';
import type { PreparedOptions, } from './prepare-options.ts';

//region Types

/**
 One accepted `set` input:
 a single keyed value or a multi-item values object.
 
 @example
 ```ts
 const input: SetInput = { key: 'theme', value: 'dark', };
 ```
 */
export type SetInput =
  | {
    /**
     Store key the value is placed under.
     */
    readonly key: string;
    /**
     Value to place;
     omitting it throws {@link MissingValueError} like upstream `conf`.
     */
    readonly value?: unknown;
  }
    | {
    /**
     Items to place at once,
     deep-partial over the store shape.
     */
      readonly values: Record<string, unknown>;
    };

/**
 The store's CRUD surface as implemented here,
 before the public `Conf` overload typing is applied at the factory edge.
 
 @example
 ```ts
 const crud = createCrud(context);
 crud.get('theme');
 ```
 */
export type CrudApi = {
  /**
   Reads one key,
   with an optional default for missing keys.
   */
  readonly get: (keyOrOptions: string | {
    readonly key: string;
    readonly defaultValue?: unknown;
  },) => unknown;
  /**
   Places one keyed value or many at once.
   */
  readonly set: (input: SetInput,) => void;
  /**
   Reports whether a key exists.
   */
  readonly has: (key: string,) => boolean;
  /**
   Appends one item to a key's array value.
   */
  readonly appendToArray: (input: {
    readonly key: string;
    readonly value: unknown;
  },) => void;
  /**
   Restores keys to their default values.
   */
  readonly reset: (input: {
    readonly keys: readonly string[];
  },) => void;
  /**
   Removes one key.
   */
  readonly delete: (key: string,) => void;
  /**
   Resets the store to its default values.
   */
  readonly clear: () => void;
};

/**
 Everything the CRUD methods need from the store around them.
 
 @example
 ```ts
 const crud = createCrud({
   options: prepared,
   defaultValues,
   storeFile,
   getStore: function getStore(): Record<string, unknown> { return {}; },
   assignStore: function assignStore(): void {},
 });
 ```
 */
export type CrudContext<T extends Record<string, unknown>> = {
  /**
   Prepared store options controlling dot-notation mode.
   */
  readonly options: PreparedOptions<T>;
  /**
   Default values from `schema` and `defaults`,
   used by `reset` and `clear`.
   */
  readonly defaultValues: Readonly<Record<string, unknown>>;
  /**
   File pipeline reads go through.
   */
  readonly storeFile: StoreFile;
  /**
   Cache-aware whole-store read.
   */
  readonly getStore: () => T;
  /**
   Whole-store write path:
   validate,
   persist,
   and dispatch `change`.
   */
  readonly assignStore: (store: Record<string, unknown>,) => void;
};

//endregion Types

//region Factory

/**
 Reports whether a value is an array of unknown items.
 
 @param value - Candidate value read from the store.
 
 @returns `true` when the value is an array.
 
 @example
 ```ts
 isUnknownArray([1]); // => true
 ```
 */
function isUnknownArray(value: unknown,): value is unknown[] {
  return Array.isArray(value,);
}

/**
 Builds the CRUD methods bound to one store's context.
 
 @param context - Options,
 defaults,
 file pipeline,
 and the store's read/write closures.
 
 @returns CRUD methods ready to attach to the store object.
 
 @example
 ```ts
 const crud = createCrud(context);
 crud.set({ key: 'theme', value: 'dark', });
 ```
 */
export function createCrud<T extends Record<string, unknown>>(context: CrudContext<T>,): CrudApi {
  /**
   Whether dots address nested properties.
   */
  const {accessPropertiesByDotNotation} = context.options;

  /**
   Builds the store copy carrying one placed value.
   
   @param store - Freshly-read store the copy starts from.
   
   @param key - Key path receiving the value.
   
   @param value - JSON-representable value to place.
   
   @returns Store copy carrying the placed value.
   */
  function setOne({
    store,
    key,
    value,
  }: {
    readonly store: Readonly<Record<string, unknown>>;
    readonly key: string;
    readonly value: unknown;
  },): Record<string, unknown> {
    checkValueType({
      key,
      value,
    },);
    return withStoreValue({
      store,
      key,
      value,
      accessPropertiesByDotNotation,
    },);
  }

  /**
   Places one keyed value or many at once,
   reading the file fresh so concurrent writers' changes survive.
   
   @param input - Single keyed value or multi-item values object to place.
   
   @throws InvalidKeyError when the key argument shape is wrong.
   
   @throws MissingValueError when the single form omits its value.
   
   @throws ReservedKeyError when the payload addresses `__internal__`.
   */
  function set(input: SetInput,): void {
    if (((typeof input) !== 'object') || (input === null))
      throw new InvalidKeyError(`Expected \`key\` to be of type \`string\` or \`object\`, got ${typeof input}`);
    if ('values' in input) {
      if (((typeof input.values) !== 'object') || (input.values === null))
        throw new InvalidKeyError(`Expected \`key\` to be of type \`string\` or \`object\`, got ${typeof input.values}`);
      if (containsReservedKey(input.values,))
        throw new ReservedKeyError();
      /**
       Store read fresh from disk so concurrent writers' changes survive.
       */
      const store = context.storeFile
        .readStore();
      context.assignStore(Object.entries(input.values,)
        .reduce(
        function placeEntry(
          currentStore: Record<string, unknown>,
          entry: [
            string,
            unknown
          ],
        ): Record<string, unknown> {
          return setOne({
            store: currentStore,
            key: entry[0],
            value: entry[1],
          },);
        },
        store,
        ),);
      return;
    }
    /**
     Single key path as the caller wrote it.
     */
    const {key} = input;
    if ((typeof key) !== 'string')
      throw new InvalidKeyError(`Expected \`key\` to be of type \`string\` or \`object\`, got ${typeof key}`);
    if (input.value === undefined)
      throw new MissingValueError();
    if (containsReservedKey(key,))
      throw new ReservedKeyError();
    /**
     Store read fresh from disk so concurrent writers' changes survive.
     */
    const store = context.storeFile
      .readStore();
    context.assignStore(setOne({
      store,
      key,
      value: input.value,
    },),);
  }

  /**
   Reads one key,
   with an optional default for missing keys.
   
   @param keyOrOptions - Key path alone,
   or an object carrying the key and its default.
   
   @returns Stored value,
   the supplied default,
   or `undefined`.
   */
  function get(keyOrOptions: string | {
    readonly key: string;
    readonly defaultValue?: unknown;
  },): unknown {
    if (((typeof keyOrOptions) !== 'string')
      && (((typeof keyOrOptions) !== 'object') || (keyOrOptions === null)))
      throw new InvalidKeyError(`Expected \`key\` to be of type \`string\`, got ${typeof keyOrOptions}`);
    /**
     Key path as the caller wrote it.
     */
    const key = (typeof keyOrOptions) === 'string' ? keyOrOptions : keyOrOptions.key;
    /**
     Default reported when the key is absent.
     */
    const defaultValue = (typeof keyOrOptions) === 'string' ? undefined : keyOrOptions.defaultValue;
    if ((typeof key) !== 'string')
      throw new InvalidKeyError(`Expected \`key\` to be of type \`string\`, got ${typeof key}`);
    return getStoreValue({
      store: context.getStore(),
      key,
      defaultValue,
      accessPropertiesByDotNotation,
    },);
  }

  /**
   Reports whether a key exists.
   
   @param key - Key path probed in the current store.
   
   @returns `true` when the key resolves to a present value.
   */
  function has(key: string,): boolean {
    return hasStoreValue({
      store: context.getStore(),
      key,
      accessPropertiesByDotNotation,
    },);
  }

  /**
   Appends one item to a key's array value,
   creating the array when the key is absent.
   
   @param input - Key whose array grows,
   and the item appended.
   
   @throws NonArrayValueError when the key holds a non-array value.
   */
  function appendToArray(input: {
    readonly key: string;
    readonly value: unknown;
  },): void {
    checkValueType({
      key: input.key,
      value: input.value,
    },);
    /**
     Store read fresh from disk so concurrent writers' changes survive.
     */
    const store = context.storeFile
      .readStore();
    /**
     Current array value;
     missing keys append onto a fresh array.
     */
    const current = getStoreValue({
      store,
      key: input.key,
      defaultValue: [],
      accessPropertiesByDotNotation,
    },);
    if (!isUnknownArray(current,))
      throw new NonArrayValueError({
        key: input.key,
      },);
    set({
      key: input.key,
      value: [
        ...current,
        input.value,
      ],
    },);
  }

  /**
   Restores keys to their default values.
   
   @param input - Keys reset to their defaults;
   keys without defaults are left untouched.
   */
  function reset(input: {
    readonly keys: readonly string[];
  },): void {
    for (const key of input.keys) {
      /**
       Default value for this key,
       absent when the key has none.
       */
      const defaultValue = getStoreValue({
        store: context.defaultValues,
        key,
        accessPropertiesByDotNotation,
      },);
      if (defaultValue !== undefined)
        set({
          key,
          value: defaultValue,
        },);
    }
  }

  /**
   Removes one key.
   
   @param key - Key path removed from the current store.
   */
  function deleteItem(key: string,): void {
    /**
     Store read fresh from disk so concurrent writers' changes survive.
     */
    const store = context.storeFile
      .readStore();
    context.assignStore(withoutStoreValue({
      store,
      key,
      accessPropertiesByDotNotation,
    },),);
  }

  /**
   Resets the store to its default values.
   */
  function clear(): void {
    /**
     Fresh store seeded with every default value.
     */
    const newStore = Object.entries(context.defaultValues,)
      .reduce(
      function placeDefault(
        currentStore: Record<string, unknown>,
        entry: [
          string,
          unknown
        ],
      ): Record<string, unknown> {
        /**
         Default entry being placed into the accumulating store copy.
         */
        const [key, value,] = entry;
        if (value === undefined)
          return currentStore;
        checkValueType({
          key,
          value,
        },);
        return withStoreValue({
          store: currentStore,
          key,
          value,
          accessPropertiesByDotNotation,
        },);
      },
      createPlainObject(),
      );
    context.assignStore(newStore,);
  }

  return {
    get,
    set,
    has,
    appendToArray,
    reset,
    delete: deleteItem,
    clear,
  };
}

//endregion Factory
