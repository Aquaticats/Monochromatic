import {
  mkdir,
  readFile,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  allInInputOrder,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  presentNamesOfKind,
  writeFileAtomic,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import { readNamespaceGeneration, } from './slice-cache-dir-read.ts';


//region Slice cache namespace
// How two lanes share one per-entry cache directory without touching each
// other's work.
//
// A lane owns a FILE PREFIX and its own generation marker. Everything else
// follows: a lane loads only its own files, stamps only its own marker, and
// when its generation moves it deletes only its own files. One shared marker
// plus a directory-wide delete, which is what this replaced, means a translate
// change throws away every settled repair slice in the corpus and vice versa.
//
// The repair lane owns the UNPREFIXED names deliberately: those files already
// exist on disk from earlier passes, and renaming them would discard real work
// to gain nothing.

import {
  EVERY_SLICE_NAMESPACE,
  type SliceNamespace,
} from './slice-cache-claims.ts';

// RE-EXPORTED rather than moved out of sight: every consumer already imports
// its claim from here, and a split made for the line cap is not a reason to
// rewrite call sites that were never wrong.
export {
  CONSOLIDATE_NAMESPACE,
  EVERY_SLICE_NAMESPACE,
  LANE_CONTEST_NAMESPACE,
  PAGE_CONSISTENCY_NAMESPACE,
  PAGE_TITLES_NAMESPACE,
  PAIRING_NAMESPACE,
  PICTURE_READING_NAMESPACE,
  REFINE_NAMESPACE,
  REPAIR_SLICE_NAMESPACE,
  SECTION_PAIRING_NAMESPACE,
  type SliceNamespace,
  TRANSLATE_SLICE_NAMESPACE,
} from './slice-cache-claims.ts';
// LIFTED for the line cap and re-exported for the same reason as the claims.
export { readNamespaceGeneration, } from './slice-cache-dir-read.ts';

/**
 Logger root for the namespaced slice cache.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 File suffix every persisted slice carries.
 */
const JSON_SUFFIX = '.json';

/**
 Prefixes that belong to a named lane.

 DERIVED FROM `EVERY_SLICE_NAMESPACE` RATHER THAN RESTATED. This was a
 hand-written list for as long as it existed, and forgetting to add a prefix
 to it is silent: the repair lane is defined as everything NOT here, so it
 adopts the unregistered files and its `discardNamespace` deletes them on the
 next generation change while logging that it discarded its own slices.

 THAT OMISSION HAPPENED SIX TIMES, twice of them still live when this was
 derived: `contest.` and `pairing.` were both missing, so a repair generation
 change threw away an entry's contest ballots and its whole block pairing.
 Both are bought from the roster, so both cost real calls to rebuy.

 The empty prefix is dropped because it is the repair lane's own: keeping it
 would make `startsWith` true for every name and leave that lane owning
 nothing at all.
 */
const CLAIMED_PREFIXES: readonly string[] = EVERY_SLICE_NAMESPACE
  .map(function toPrefix({ prefix, },): string {
    return prefix;
  },)
  .filter(function isNamed(prefix,): boolean {
    return prefix !== '';
  },);


/**
 Whether a file in an entry's cache directory is a slice of some lane. Every
 lane writes its slices under the one suffix, and neither a generation marker
 nor a slice still being written under its atomic write's temporary name
 carries it.

 @param name - file name, without a directory

 @returns Whether it ends in the slice suffix

 @example
 ```ts
 const isSlice = isSliceFileName({ name: 'translate.0-1-tabby.json', },);
 ```
 */
export function isSliceFileName({ name, }: { readonly name: string; },): boolean {
  return name.endsWith(JSON_SUFFIX,);
}

/**
 Whether a file in a shared cache directory belongs to one lane.

 @param name - file name as `readdir` returned it

 @param namespace - lane asking

 @returns True when that lane owns the file

 @example
 ```ts
 if (belongsToNamespace({ name, namespace, },)) resumed.set(key, parsed,);
 ```
 */
export function belongsToNamespace(
  {
    name,
    namespace,
  }: {
    readonly name: string;
    readonly namespace: SliceNamespace;
  },
): boolean {
  if (!isSliceFileName({ name, },))
    return false;
  if (namespace.prefix !== '')
    return name.startsWith(namespace.prefix,);
  return !CLAIMED_PREFIXES.some(function isClaimed(prefix,): boolean {
    return name.startsWith(prefix,);
  },);
}

/**
 Slice key a file name carries.

 @param name - file name owned by this lane

 @param namespace - lane owning it

 @returns Key the driver derived

 @example
 ```ts
 const key = keyOfSliceFile({ name, namespace, },);
 ```
 */
export function keyOfSliceFile(
  {
    name,
    namespace,
  }: {
    readonly name: string;
    readonly namespace: SliceNamespace;
  },
): string {
  return name.slice(
    namespace.prefix
      .length,
    -JSON_SUFFIX.length,
  );
}

/**
 File name one lane writes a key under.

 @param key - slice key

 @param namespace - lane writing it

 @returns File name inside the entry directory

 @example
 ```ts
 const name = sliceFileName({ key, namespace, },);
 ```
 */
export function sliceFileName(
  {
    key,
    namespace,
  }: {
    readonly key: string;
    readonly namespace: SliceNamespace;
  },
): string {
  return `${namespace.prefix}${key}${JSON_SUFFIX}`;
}

/**
 Wraps one settled slice with the key it was stored under.

 WHY THE KEY IS INSIDE THE FILE as well as in its name. The name is what a
 loader derives the key from, so a payload sitting under the wrong name is
 resumed as though it belonged there, and the driver splices it into a slice
 it was never computed for. That used to be caught downstream, by both lanes
 refusing a record whose slice index disagreed with the one they asked for;
 taking the index out of the cache key made that check wrong, since a record
 now legitimately answers for any slice carrying the same texts. This is the
 check that replaces it, and it tests the thing that actually matters: not
 where the record sat, but what question it answered.

 @param key - key this slice is being stored under

 @param serialized - record as its lane serialized it

 @returns Envelope text to write

 @example
 ```ts
 const text = envelopedSlice({ key, serialized, },);
 ```
 */
function envelopedSlice(
  {
    key,
    serialized,
  }: {
    readonly key: string;
    readonly serialized: string;
  },
): string {
  return JSON.stringify({
    cacheKey: key,
    record: JSON.parse(serialized,) as unknown,
  },);
}

/**
 Reads one cache file's envelope, when it is one this loader wrote.

 @param parsed - parsed file contents

 @param key - key the file's NAME says it answers

 @returns Record inside, or nothing when the file is not an envelope or
 answers a different key

 @example
 ```ts
 const record = recordOfEnvelope({ parsed, key, },);
 ```
 */
function recordOfEnvelope(
  {
    parsed,
    key,
  }: {
    readonly parsed: unknown;
    readonly key: string;
  },
): unknown {
  if (!isJsonRecord(parsed,))
    return undefined;
  if ((!('cacheKey' in parsed)) || (!('record' in parsed)))
    return undefined;
  if (parsed.cacheKey !== key)
    return undefined;
  return parsed.record;
}

/**
 Loads one lane's settled slices, tolerating a missing directory and
 half-written files.

 @param dir - per-entry cache directory

 @param namespace - lane loading

 @param isValue - guard deciding whether a parsed file is this lane's value;
 anything it rejects is treated as absent and recomputed

 @returns Settled values keyed by slice key

 @example
 ```ts
 const resumed = await loadNamespacedSlices({ dir, namespace, isValue, },);
 ```
 */
export async function loadNamespacedSlices<ValueT,>(
  {
    dir,
    namespace,
    isValue,
  }: {
    readonly dir: string;
    readonly namespace: SliceNamespace;
    readonly isValue: (value: unknown,) => value is ValueT;
  },
): Promise<Map<string, ValueT>> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: loadNamespacedSlices.name,
    l,
  },);

  /**
   Settled values keyed by slice key.
   */
  const resumed = new Map<string, ValueT>();

  /**
   Files present under the directory. A directory named like a slice is none,
   and read as one it stopped the entry on EISDIR (ledger B65).
   */
  const names = await presentNamesOfKind({
    dir,
    kind: 'file',
  },);
  for (const name of names) {
    if (!belongsToNamespace({
      name,
      namespace,
    },))
      continue;
    try {
      /* oxlint-disable no-await-in-loop -- small per-entry cache read sequentially at setup */
      /**
       Parsed JSON of this cache file, checked before it is trusted.
       */
      const parsed: unknown = JSON.parse(await readFile(
        join(
          dir,
          name,
        ),
        'utf8',
      ),);
      /* oxlint-enable no-await-in-loop */

      /**
       Key this file's NAME says it answers.
       */
      const key = keyOfSliceFile({
        name,
        namespace,
      },);

      /**
       Record inside, absent when the file is not an envelope this loader
       wrote or when it answers some other key. Both are treated as absent
       rather than as failures: a slice nobody can vouch for simply costs what
       an uncached one costs.
       */
      const record = recordOfEnvelope({
        parsed,
        key,
      },);
      if (isValue(record,))
        resumed.set(
          key,
          record,
        );
      else
        // Said out loud, because a record the guard refuses is recomputed and
        // paid for again, and a refused record may be a settlement
        // reached while a stage heard nobody rather than a shape drift.
        rl.warn(`${namespace.marker}: ${name} is not a resumable value for this lane; it will be recomputed`,);
    }
    catch (error) {
      // A half-written file (SyntaxError) is recomputed, and SAID, since the
      // slice is bought again (ledger A11); other faults surface. Persisting
      // is atomic now, so this marks a file written before that or by hand.
      if (!(error instanceof SyntaxError))
        throw error;
      rl.warn(`${namespace.marker}: ${name} does not parse (${error.name}); it will be recomputed`,);
    }
  }
  return resumed;
}


/**
 Opens one lane's slice cache inside a shared entry directory.

 A cache filled by a different pipeline is DISCARDED rather than resumed, and
 only this lane's files are discarded. Resuming across pipelines is the one
 generation defect no reader can catch: the settled artifact records a single
 digest, so an entry built half from cached slices and half from current code
 looks like ordinary work to every filter downstream.

 @param dir - per-entry cache directory, shared with other lanes

 @param generation - digest of the built pipeline this pass runs

 @param namespace - lane opening

 @param isValue - guard for this lane's stored value

 @returns Cache resuming this lane's settled slices and persisting new ones

 @example
 ```ts
 const cache = await openNamespacedCache({ dir, generation, namespace, isValue, },);
 ```
 */
export async function openNamespacedCache<ValueT,>(
  {
    dir,
    generation,
    namespace,
    isValue,
  }: {
    readonly dir: string;
    readonly generation: string;
    readonly namespace: SliceNamespace;
    readonly isValue: (value: unknown,) => value is ValueT;
  },
): Promise<SliceCache<ValueT>> {
  await mkdir(
    dir,
    { recursive: true, },
  );

  /**
   Pipeline that filled this lane's slices, empty when it never wrote here.
   */
  const cached = await readNamespaceGeneration({
    dir,
    namespace,
  },);

  /**
   This lane's settled slices, kept only when the pipeline that produced them
   is the one running now.
   */
  const resumed = (cached === generation)
    ? await loadNamespacedSlices({
      dir,
      namespace,
      isValue,
    },)
    : new Map<string, ValueT>();
  if (cached !== generation) {
    await discardNamespace({
      dir,
      namespace,
      cached,
    },);
    // Written after the discard, so the marker always describes what this
    // lane now holds, and written ONLY WHEN IT DOES NOT ALREADY NAME THIS
    // GENERATION. A marker cut short reads as a mismatch on the next open,
    // which discards every slice this lane holds here, and a restamp in place
    // truncates the marker before it writes, so a write the disk then refuses
    // leaves it empty. Restamped only where this open has just discarded, the
    // marker is at risk only while the lane holds nothing to lose; the restamp
    // is atomic all the same, like the slices, so a refused write leaves the
    // earlier marker whole.
    await writeFileAtomic({
      path: join(
        dir,
        namespace.marker,
      ),
      text: `${generation}\n`,
    },);
  }

  return {
    resumed,
    persist: async function persistSlice({
      key,
      serialized,
    },): Promise<void> {
      // ATOMIC (ledger A11): a direct write cut short left a slice the next
      // open could not parse, and the slice was bought again.
      await writeFileAtomic({
        path: join(
          dir,
          sliceFileName({
            key,
            namespace,
          },),
        ),
        text: `${
          envelopedSlice({
            key,
            serialized,
          },)
        }\n`,
      },);
    },
  };
}

/**
 Removes one lane's slices from a shared directory, leaving every other lane's
 work in place.

 @param dir - per-entry cache directory

 @param namespace - lane whose slices go

 @param cached - digest those slices were filled by, for the log line

 @example
 ```ts
 await discardNamespace({ dir, namespace, cached, },);
 ```

 @internal
 */
export async function discardNamespace(
  {
    dir,
    namespace,
    cached,
  }: {
    readonly dir: string;
    readonly namespace: SliceNamespace;
    readonly cached: string;
  },
): Promise<void> {
  /**
   This lane's files, named before any of them is removed. Files only: a
   directory named like a slice is no lane's, and removing it raised EISDIR
   out of the open that asked for the discard (ledger B65).
   */
  const owned = (await presentNamesOfKind({
    dir,
    kind: 'file',
  },))
    .filter(function isOwned(name,): boolean {
      return belongsToNamespace({
        name,
        namespace,
      },);
    },);
  if (owned.length > 0) {
    /**
     Logger pre-tagged with this function's name.
     */
    const dl = tagged({
      tag: discardNamespace.name,
      l,
    },);
    dl.info(
      `SLICE discarding ${String(owned.length,)} cached ${
        wordForCount({
          count: owned.length,
          one: 'slice',
          many: 'slices',
        },)
      } in ${dir}: `
        + `filled by ${cached === '' ? '(unstamped)' : cached}`,
    );
  }
  await allInInputOrder({
    members: owned.map(async function removeOne(name,): Promise<void> {
      await rm(
        join(
          dir,
          name,
        ),
        { force: true, },
      );
    },),
  },);
}

//endregion Slice cache namespace
