/**
 Coverage-driver scenarios over the web storage sinks' shared helpers: the
 run-scoped key helpers over valid, flat, and malformed keys, quota
 detection under Node and under staged Deno and Bun markers, the
 quota-error classifier, and the localStorage store driven directly over a
 memory stand-in.

 @module
 */

import {
  _buildLogKey as buildLogKey,
  _compareLogKeys as compareLogKeys,
  _createLocalStorageStore as createLocalStorageStore,
  _detectLocalStorageQuotaChars as detectLocalStorageQuotaChars,
  _detectSessionStorageQuotaChars as detectSessionStorageQuotaChars,
  _isQuotaExceededError as isQuotaExceededError,
  _parseLogKey as parseLogKey,
} from '@monochromatic-dev/logger-core/ts';

import { installGlobalValue, } from './coverage-globals.ts';
import {
  createMemoryStorage,
  installFakeLocalStorage,
} from './coverage-storage-fakes.ts';

//region Fixtures

/**
 Keys the parser must refuse: the sessionStorage sink's flat shape, no
 prefix, a non-numeric stamp or index, an empty stamp, nonce, or index,
 and a missing segment.
 */
const MALFORMED_KEYS: readonly string[] = [
  'monochromatic.log.3',
  'garbage',
  'monochromatic.log.1.n.x',
  'monochromatic.log.x.n.1',
  'monochromatic.log..n.1',
  'monochromatic.log.1..1',
  'monochromatic.log.1.n.',
  'monochromatic.log.1.n',
  'monochromatic.log.',
];

/**
 Runtimes the quota tables distinguish by a marker global.
 */
const RUNTIME_MARKERS: readonly string[] = [
  'Deno',
  'Bun',
];

/**
 Web Storage quota error code, the legacy `DOMException.code` browsers
 still set alongside the `QuotaExceededError` name.
 */
const QUOTA_EXCEEDED_CODE = 22;

//endregion Fixtures

//region Scenarios

/**
 Parses valid keys covering every ordering comparison (equal stamps with
 differing nonces both ways, equal stamp and nonce with differing indices,
 differing stamps), refuses every malformed key, and compares every valid
 pair.
 */
function exerciseKeys(): void {
  /**
   Parsed forms of the valid keys.
   */
  const parsed = [
    parseLogKey(buildLogKey({
      index: 2,
      nonce: 'n',
      stamp: 1,
    },),)
      .parsed,
    parseLogKey(buildLogKey({
      index: 0,
      nonce: 'm',
      stamp: 2,
    },),)
      .parsed,
    parseLogKey(buildLogKey({
      index: 1,
      nonce: 'n',
      stamp: 1,
    },),)
      .parsed,
    parseLogKey(buildLogKey({
      index: 0,
      nonce: 'a',
      stamp: 1,
    },),)
      .parsed,
  ];
  for (const key of MALFORMED_KEYS)
    parseLogKey(key,);
  for (const first of parsed)
    for (const second of parsed)
      if ((first !== undefined) && (second !== undefined))
        compareLogKeys({
          first,
          second,
        },);
}

/**
 Quota detection under Node, then under each staged runtime marker.
 */
function exerciseQuotaDetection(): void {
  detectLocalStorageQuotaChars();
  detectSessionStorageQuotaChars();
  for (const marker of RUNTIME_MARKERS) {
    /**
     Runtime marker staged for the scope.
     */
    using _runtime = installGlobalValue({
      name: marker,
      value: {},
    },);
    detectLocalStorageQuotaChars();
    detectSessionStorageQuotaChars();
  }
}

/**
 The quota-error classifier over a named `DOMException`, a coded error, an
 unrelated error, and a non-error.
 */
function exerciseQuotaErrors(): void {
  isQuotaExceededError(new DOMException(
    'q',
    'QuotaExceededError',
  ),);
  isQuotaExceededError(Object.assign(
    new Error('q',),
    { code: QUOTA_EXCEEDED_CODE, },
  ),);
  isQuotaExceededError(new Error('other',),);
  isQuotaExceededError('not an error',);
}

/**
 The localStorage store persisting one batch over a memory stand-in.
 */
function exerciseStoreDirectly(): void {
  /**
   Memory store installed for the direct store call.
   */
  using _memory = installFakeLocalStorage({ fake: createMemoryStorage(), },);
  createLocalStorageStore()
    .persist('{"level":"info","message":"direct","timestamp":0}',);
}

//endregion Scenarios

/**
 Runs every shared-helper scenario in order.

 @example
 ```ts
 exerciseKeysAndQuota();
 ```
 */
export function exerciseKeysAndQuota(): void {
  exerciseKeys();
  exerciseQuotaDetection();
  exerciseQuotaErrors();
  exerciseStoreDirectly();
}
