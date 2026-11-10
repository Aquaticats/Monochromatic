/**
 Coverage driver exercise block for the item surface:
 sets and reads,
 defaults,
 error paths,
 whole-store replacement,
 item counts and iteration,
 and change subscriptions.

 Each assertion is a fixed input, so the store paths it reaches and the V8
 coverage they produce are reproducible.

 @module
 */

import { writeFile, } from 'node:fs/promises';

import { createConf, } from '@monochromatic-dev/upstream-fork-conf/ts';

import {
  FRESH_VALUE,
  NON_FUNCTION_CALLBACK,
  NON_STRING_KEY,
  SEED_VERSION,
  ensure,
  functionValue,
  observeChange,
  swallow,
} from './coverage-fixtures.ts';
import { createTempStoreDir, } from './store-fixture.ts';
import { STORE_DEFAULTS, } from './store-operations.ts';

/**
 Exercises the item surface with its error paths,
 store replacement with bookkeeping preservation,
 item counts and iteration,
 and change subscriptions.
 
 @example
 ```ts
 await exerciseStoreField();
 ```
 */
export async function exerciseStoreField(): Promise<void> {
  /**
   Disposable directory holding this store's config file.
   */
  await using dir = await createTempStoreDir();
  /**
   Store exercising the item surface.
   */
  const config = createConf({
    cwd: dir.path,
    defaults: STORE_DEFAULTS,
  });

  config.set({
    key: 'theme',
    value: FRESH_VALUE,
  },);
  config.set({
    values: {
      retries: 2,
      'nested.flag': false,
    },
  },);
  ensure({
    holds: config.get('theme',) === FRESH_VALUE,
    label: 'single-set readback',
  },);
  ensure({
    holds: config.get({
      key: 'missing',
      defaultValue: 'fallback',
    },) === 'fallback',
    label: 'default readback',
  },);
  ensure({
    holds: config.get({
      key: 'missing',
      defaultValue: 'fallback2',
    },) === 'fallback2',
    label: 'object default readback',
  },);
  ensure({
    holds: config.has('theme',),
    label: 'probe hit',
  },);
  ensure({
    holds: !config.has('missing',),
    label: 'probe miss',
  },);

  config.appendToArray({
    key: 'items',
    value: FRESH_VALUE,
  },);
  config.appendToArray({
    key: 'fresh-array',
    value: FRESH_VALUE,
  },);
  config.reset({
    keys: [
      'theme',
      'no-default',
    ],
  },);
  config.reset({
    keys: [],
  },);
  config.delete('retries',);
  config.delete('missing',);

  //region Item error paths

  /**
   Multi-item payload carrying one `undefined` value,
   driving the unsupported-value-type path of multi-`set`.
   */
  const undefinedEntry: Record<string, unknown> = Object.fromEntries([
    [
      'theme',
      undefined,
    ],
  ],);
  /**
   Multi-item payload carrying one function value.
   */
  const functionEntry: Record<string, unknown> = Object.fromEntries([
    [
      'theme',
      functionValue,
    ],
  ],);

  swallow(function missingSingleValue() {
    config.set({
      key: 'theme',
      value: undefined,
    },);
  },);
  swallow(function unsupportedFunctionValue() {
    config.set({
      key: 'theme',
      value: functionValue,
    },);
  },);
  swallow(function unsupportedSymbolValue() {
    config.set({
      key: 'theme',
      value: Symbol('store value of unsupported type',),
    },);
  },);
  swallow(function unsupportedMultiValue() {
    config.set({
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- deliberately type-invalid payload exercising the unsupported-value-type path of multi-item set.
      values: undefinedEntry as never,
    },);
  },);
  swallow(function unsupportedFunctionMultiValue() {
    config.set({
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- deliberately type-invalid payload exercising the unsupported-value-type path of multi-item set.
      values: functionEntry as never,
    },);
  },);
  swallow(function unsupportedAppendValue() {
    config.appendToArray({
      key: 'items',
      value: undefined,
    },);
  },);
  swallow(function nonArrayAppend() {
    config.appendToArray({
      key: 'theme',
      value: FRESH_VALUE,
    },);
  },);
  swallow(function reservedSingleKey() {
    config.set({
      key: '__internal__',
      value: FRESH_VALUE,
    },);
  },);
  swallow(function reservedPathKey() {
    config.set({
      key: '__internal__.x',
      value: FRESH_VALUE,
    },);
  },);
  swallow(function reservedMultiKey() {
    config.set({
      values: {
        __internal__: FRESH_VALUE,
      },
    },);
  },);
  swallow(function reservedNestedKey() {
    config.set({
      values: {
        nested: {
          __internal__: FRESH_VALUE,
        },
      },
    },);
  },);
  swallow(function invalidSetKey() {
    config.set(NON_STRING_KEY,);
  },);
  swallow(function invalidGetKey() {
    config.get(NON_STRING_KEY,);
  },);

  //endregion Item error paths

  //region Store surface

  config.store = {
    theme: FRESH_VALUE,
  };
  ensure({
    holds: config.size
      === Object.keys(config.store,)
        .length,
    label: 'store size',
  },);
  ensure({
    holds: [...config,].length === config.size,
    label: 'store iteration',
  },);

  /**
   Raw file contents carrying bookkeeping the replacement must preserve.
   */
  const fileWithBookkeeping = JSON.stringify({
    __internal__: {
      migrations: {
        version: SEED_VERSION,
      },
    },
    theme: FRESH_VALUE,
  },);
  await writeFile(
    config.path,
    fileWithBookkeeping,
  );
  config.store = {
    theme: FRESH_VALUE,
  };
  ensure({
    holds: config.store
      .theme
      === FRESH_VALUE,
    label: 'store replacement',
  },);

  //endregion Store surface

  //region Change subscriptions

  /**
   Unsubscribe handle of a key subscription.
   */
  const stopKeyWatch = config.onDidChange({
    key: 'theme',
    callback: observeChange,
  },);
  /**
   Unsubscribe handle of a whole-store subscription.
   */
  const stopStoreWatch = config.onDidAnyChange(observeChange,);
  config.set({
    key: 'theme',
    value: FRESH_VALUE,
  },);
  stopKeyWatch();
  stopStoreWatch();
  swallow(function invalidCallback() {
    config.onDidAnyChange(NON_FUNCTION_CALLBACK,);
  },);
  config.closeWatcher();

  //endregion Change subscriptions

  config.clear();
}
