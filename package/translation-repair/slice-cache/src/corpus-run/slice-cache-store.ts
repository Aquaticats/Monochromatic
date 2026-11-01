import { rm, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  type BlockPair,
  isIndexPairList,
  type PairedDocumentRecord,
  refinementIsResumable,
  silentStagesOfRecord,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { isStringList, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { PairedSectionRecord, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { isQuotedPassages, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { RefinedSliceSettlement, } from '@monochromatic-dev/translation-repair-repair/ts';
import type { ChunkRepairOutcome, } from '@monochromatic-dev/translation-repair-assembly/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  TRANSLATE_SLICE_CACHE_VERSION,
  type TranslateDisposition,
  type TranslateSliceRecord,
} from '@monochromatic-dev/translation-repair-translate-lane/ts';
import { presentNamesOfKind, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  belongsToNamespace,
  openNamespacedCache,
  PAIRING_NAMESPACE,
  REFINE_NAMESPACE,
  SECTION_PAIRING_NAMESPACE,
  REPAIR_SLICE_NAMESPACE,
  TRANSLATE_SLICE_NAMESPACE,
} from './slice-cache-namespace.ts';

//region Slice cache store
// Disk-backed per-entry slice cache making a large corpus document resumable:
// every settled slice is one JSON file named by its hash, so a run aborted at
// the hard cap resumes from the last settled slice on the next attempt.
//
// TWO LANES SHARE ONE ENTRY DIRECTORY, each owning a file prefix and its own
// generation marker: see `slice-cache-namespace.ts`. This file holds what each
// lane stores and how a settled entry is dropped.

/**
 Whether a parsed cache file is a usable repair outcome. A half-written or
 stale file that misses these fields is treated as absent and recomputed.

 @param value - parsed JSON of a cache file

 @returns True when the value carries the outcome's own fields

 @example
 ```ts
 if (isChunkRepairOutcome(parsed,)) resumed.set(key, parsed,);
 ```
 */
function isChunkRepairOutcome(value: unknown,): value is ChunkRepairOutcome {
  return isJsonRecord(value,)
    && ((typeof value.sliceIndex) === 'number')
    && ((typeof value.repairedText) === 'string')
    && ((typeof value.changed) === 'boolean')
    && ((typeof value.nonTranslationStanding) === 'boolean')
    && Array.isArray(value.issues,)
    && Array.isArray(value.resolvedIssueIds,)
    && Array.isArray(value.repairRegions,)
    && Array.isArray(value.candidateResolvedIssueIds,)
    && ((typeof value.accuracyPatchSelected) === 'boolean')
    && ((typeof value.refined) === 'boolean')
    && Array.isArray(value.claimAttributions,)
    && Array.isArray(value.heardCriticIds,)
    && Array.isArray(value.rounds,)
    && isJsonRecord(value.authorship,)
    && Array.isArray(value.droppedDeclaredNames,)
    && Array.isArray(value.findings,)
    // REQUIRED, so a record written before the field existed is not resumed: it could not say that its panel heard
    // nobody (XingZ6018 slice 85 was cached so, its findings located by cluster).
    && Array.isArray(value.silentStages,);
}

/**
 Dispositions a stored translate record may carry.

 KEYED BY `TranslateDisposition` rather than listed, so the compiler refuses a
 disposition the lane gains that this lacks: every record carrying it would
 otherwise be read as absent and the slice bought again.
 */
const TRANSLATE_DISPOSITIONS: Readonly<Record<TranslateDisposition, true>> = {
  'stage-result': true,
  'refused-alignment': true,
  'refused-quote-loss': true,
  'refused-declared-name': true,
};

/**
 Whether a stored value names a disposition a translate record may carry.

 @param value - disposition as the cache file carries it

 @returns Whether it is one of the dispositions `TRANSLATE_DISPOSITIONS` holds

 @example
 ```ts
 const known = isTranslateDisposition('stage-result',);
 ```
 */
function isTranslateDisposition(value: unknown,): value is TranslateDisposition {
  return ((typeof value) === 'string')
    && Object.hasOwn(
      TRANSLATE_DISPOSITIONS,
      value,
    );
}

/**
 Whether a parsed cache file is a usable translate record.

 Checks the LANE and the SCHEMA before anything else. A repair outcome carries
 neither, so it can never be resumed as a translation however the file is
 named, and a record written under an older schema is recomputed rather than
 read with fields that have since changed meaning.

 A DECLARED-NAME REFUSAL CARRIES THE NAMES IT DROPPED, which the report
 names from the record; the driver has always written them, so one without
 them came from no driver and is recomputed (ledger T8, sixth batch). A
 QUOTE-LOSS REFUSAL CARRIES THE COUNTS IT COMPARED, for the same reason
 (ledger B42).

 @param value - parsed JSON of a cache file

 @returns True when the value is this schema's translate record

 @example
 ```ts
 if (isTranslateSliceRecord(parsed,)) resumed.set(key, parsed,);
 ```
 */
function isTranslateSliceRecord(
  value: unknown,
): value is TranslateSliceRecord {
  return isJsonRecord(value,)
    && (value.kind === 'translate-slice')
    && (value.schemaVersion === TRANSLATE_SLICE_CACHE_VERSION)
    && ((typeof value.sliceIndex) === 'number')
    && ((typeof value.outputText) === 'string')
    && ((typeof value.changed) === 'boolean')
    && isTranslateDisposition(value.disposition,)
    && ((value.disposition !== 'refused-declared-name') || Array.isArray(value.droppedDeclaredNames,))
    && ((value.disposition !== 'refused-quote-loss') || isQuotedPassages(value.quotedPassages,))
    && isJsonRecord(value.stageResult,)
    && isJsonRecord(value.alignment,)
    && Array.isArray(value.findings,);
}

/**
 Lists entries under the slice-cache root that carry at least one settled
 slice in EITHER lane, so the pass can resume an in-flight document to
 completion before starting fresh ones.

 A settled entry (directory discarded) or one that aborted before settling
 anything (empty directory) contributes nothing, and neither does anything
 the pass never writes there: a file under the root, or a symlink, which
 resumed an entry with progress a second time under another name (ledger
 B65).

 @param dir - slice-cache root holding one subdirectory per entry

 @returns Set of entry ids carrying resumable progress, empty when none

 @example
 ```ts
 const resumable = await listResumableEntries({ dir: sliceCacheDir, },);
 ```
 */
export async function listResumableEntries(
  { dir, }: { readonly dir: string; },
): Promise<Set<string>> {
  /**
   Entry ids with one or more settled slices on disk.
   */
  const resumable = new Set<string>();

  /**
   Per-entry subdirectory names under the cache root.
   */
  const ids = await presentNamesOfKind({
    dir,
    kind: 'directory',
  },);
  for (const id of ids) {
    /* oxlint-disable no-await-in-loop -- small one-time setup scan over per-entry dirs */
    /**
     Files inside this entry's cache directory, none where a settled entry's
     discard removed it since the root was listed.
     */
    const names = await presentNamesOfKind({
      dir: join(
        dir,
        id,
      ),
      kind: 'file',
    },);
    /* oxlint-enable no-await-in-loop */
    if (names.some(function isSliceFile(name,): boolean {
      return belongsToNamespace({
        name,
        namespace: REPAIR_SLICE_NAMESPACE,
      },)
        || belongsToNamespace({
          name,
          namespace: TRANSLATE_SLICE_NAMESPACE,
        },);
    },))
      resumable.add(id,);
  }
  return resumable;
}

/**
 Opens an entry's REPAIR slice cache.

 @param dir - per-entry slice-cache directory

 @param generation - digest of the built pipeline this pass runs

 @returns Cache resuming settled repair slices and persisting new ones

 @example
 ```ts
 const sliceCache = await openSliceCache({ dir: entryCacheDir, generation, },);
 ```
 */
export async function openSliceCache(
  {
    dir,
    generation,
  }: {
    readonly dir: string;
    readonly generation: string;
  },
): Promise<SliceCache<ChunkRepairOutcome>> {
  return await openNamespacedCache({
    dir,
    generation,
    namespace: REPAIR_SLICE_NAMESPACE,
    isValue: isResumableRepairOutcome,
  },);
}

/**
 Whether a parsed value is a list of correspondences.

 SHAPE ONLY. What a pairing must satisfy against the blocks it describes is
 `readBlockPairing`'s question, which needs the blocks. A cached block pairing
 is read against its section's blocks by `prepareBlockPairing`, through
 `readBlockPairing` itself, before anything uses it; a record that reader
 refuses is a miss there. A cached section pairing, which this test reads
 too, is read the same way by `buySectionPairing`, through
 `readSectionPairing`.

 @param value - candidate list, still unknown in type

 @returns Whether every entry names two integer block indices

 @example
 ```ts
 const ok = isPairList([{ source: 0, target: 0, },],);
 ```
 */
function isPairList(value: unknown,): value is readonly BlockPair[] {
  return isIndexPairList(value,);
}

/**
 Whether a parsed cache file is a usable pairing record, block or section:
 correspondences beside the findings their round produced. The two caches
 store one shape and only their key spaces tell them apart
 (`openSectionPairingCache`), so one test reads both.

 REFUSES A BARE ARRAY, which the block namespace stored until 2026-08-22,
 when the findings a section produced became half the record. Nothing has to
 read the old shape: the namespace is discarded whenever the stored
 generation differs from the running pipeline digest, and editing these files
 changes that digest. The refusal is the belt beside that brace.

 Each finding must be text, as both record types say; the two copies this
 replaced checked only that findings were a list (audit area six, ledger B18).

 @param value - parsed JSON of a cache file

 @returns Whether it carries correspondences beside their findings

 @example
 ```ts
 if (isCachedPairingRecord(parsed,)) resumed.set(key, parsed,);
 ```
 */
function isCachedPairingRecord(value: unknown,): value is PairedSectionRecord & PairedDocumentRecord {
  return isJsonRecord(value,)
    && isStringList(value.findings,)
    && isPairList(value.pairs,);
}

/**
 Opens an entry's block-pairing cache.

 PAIRING IS BOUGHT ONCE PER DOCUMENT PAIR AND NEVER AGAIN. Without this a
 resumed entry that buys nothing else still spends a round per section, which
 `pass-entry`'s own test caught: it asserts a fully cached resume makes no
 calls at all.

 @param dir - per-entry slice-cache directory

 @param generation - digest of the built pipeline this pass runs

 @returns Cache resuming settled pairings and persisting new ones

 @example
 ```ts
 const pairingCache = await openPairingCache({ dir: entryCacheDir, generation, },);
 ```
 */
export async function openPairingCache(
  {
    dir,
    generation,
  }: {
    readonly dir: string;
    readonly generation: string;
  },
): Promise<SliceCache<PairedSectionRecord>> {
  return await openNamespacedCache({
    dir,
    generation,
    namespace: PAIRING_NAMESPACE,
    isValue: isCachedPairingRecord,
  },);
}

/**
 Opens an entry's whole-document SECTION-pairing cache.

 ITS OWN NAMESPACE beside the block one. Both records carry a list of
 `{source, target}` and a list of findings, so nothing in the stored shape
 separates a section answer from a block answer and only the key space can.

 @param dir - per-entry slice-cache directory

 @param generation - digest of the built pipeline this pass runs

 @returns Cache resuming a settled section pairing and persisting a new one

 @example
 ```ts
 const sectionCache = await openSectionPairingCache({ dir: entryCacheDir, generation, },);
 ```
 */
export async function openSectionPairingCache(
  {
    dir,
    generation,
  }: {
    readonly dir: string;
    readonly generation: string;
  },
): Promise<SliceCache<PairedDocumentRecord>> {
  return await openNamespacedCache({
    dir,
    generation,
    namespace: SECTION_PAIRING_NAMESPACE,
    isValue: isCachedPairingRecord,
  },);
}

/**
 Opens an entry's TRANSLATE slice cache, beside the repair one.

 @param dir - per-entry slice-cache directory

 @param generation - digest of the built pipeline this pass runs

 @returns Cache resuming settled translate slices and persisting new ones

 @example
 ```ts
 const translateCache = await openTranslateSliceCache({ dir: entryCacheDir, generation, },);
 ```
 */
export async function openTranslateSliceCache(
  {
    dir,
    generation,
  }: {
    readonly dir: string;
    readonly generation: string;
  },
): Promise<SliceCache<TranslateSliceRecord>> {
  return await openNamespacedCache({
    dir,
    generation,
    namespace: TRANSLATE_SLICE_NAMESPACE,
    isValue: isTranslateSliceRecord,
  },);
}

/**
 Discards a settled entry's whole slice cache, bounding the cache directory to
 documents still in flight.

 Takes the DIRECTORY rather than one lane, because it runs when the entry is
 finished: every lane is done with it, and leaving one lane's files behind
 would keep the entry listed as resumable forever.

 @param dir - per-entry slice-cache directory

 @example
 ```ts
 await discardSliceCache({ dir: entryCacheDir, },);
 ```
 */
export async function discardSliceCache(
  { dir, }: { readonly dir: string; },
): Promise<void> {
  await rm(
    dir,
    {
      recursive: true,
      force: true,
    },
  );
}

/**
 Whether a parsed cache file is a usable refinement settlement.

 CHECKS THE OUTCOME INSIDE rather than only the wrapper, because bytes off
 disk become published text here: a settlement whose outcome is malformed
 would splice a broken slice into the document with no later stage able to
 tell it from a fresh one.

 @param value - parsed JSON of a cache file

 @returns True when it carries a settlement over a well-formed outcome

 @example
 ```ts
 if (isRefinedSliceSettlement(parsed,)) resumed.set(key, parsed,);
 ```
 */
function isRefinedSliceSettlement(value: unknown,): value is RefinedSliceSettlement {
  return isJsonRecord(value,)
    && Array.isArray(value.findings,)
    && isChunkRepairOutcome(value.outcome,);
}

/**
 Whether a stored repair outcome may be resumed: the shape is right AND every
 stage that settled it was heard. A record written while a stage fell short
 of quorum is an outage frozen as a decision, and is recomputed.

 @param value - parsed cache file

 @returns True when the value is an outcome worth resuming

 @example
 ```ts
 if (isResumableRepairOutcome(parsed,)) resumed.set(key, parsed,);
 ```
 */
function isResumableRepairOutcome(value: unknown,): value is ChunkRepairOutcome {
  if (!isChunkRepairOutcome(value,))
    return false;
  /**
   The stages the settlement names as silent, as data or as a finding.
   */
  const silent = silentStagesOfRecord({ record: value, },);
  return silent.length === 0;
}

/**
 Whether a stored refinement may be resumed, on the same rule as the repair
 outcome: right shape, every stage heard. The question is the one the refine
 phase asks before it writes (`refinementIsResumable`), so a record this reader
 would discard is never written.

 @param value - parsed cache file

 @returns True when the value is a settlement worth resuming

 @example
 ```ts
 if (isResumableRefinement(parsed,)) resumed.set(key, parsed,);
 ```
 */
function isResumableRefinement(value: unknown,): value is RefinedSliceSettlement {
  if (!isRefinedSliceSettlement(value,))
    return false;
  return refinementIsResumable({ settlement: value, },);
}

/**
 Opens an entry's REFINEMENT cache.

 Separate from the repair lane's own cache because the naturalness lane runs
 after the accuracy pass has already persisted, so its answers cannot ride in
 a record written before it was asked. Without this the lane was rebought on
 every resumed run and published different text on identical inputs.

 @param dir - per-entry slice-cache directory

 @param generation - digest of the built pipeline this pass runs

 @returns Cache resuming settled refinements and persisting new ones

 @example
 ```ts
 const refineCache = await openRefineSliceCache({ dir: entryCacheDir, generation, },);
 ```
 */
export async function openRefineSliceCache(
  {
    dir,
    generation,
  }: {
    readonly dir: string;
    readonly generation: string;
  },
): Promise<SliceCache<RefinedSliceSettlement>> {
  return await openNamespacedCache({
    dir,
    generation,
    namespace: REFINE_NAMESPACE,
    isValue: isResumableRefinement,
  },);
}

//endregion Slice cache store
