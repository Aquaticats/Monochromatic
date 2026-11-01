import {
  isJsonRecord,
  refusalText,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { isLowerHexDigit, } from '@monochromatic-dev/translation-repair-markup/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { readRunJson, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type ArtifactFileName,
  entryIdOfArtifact,
} from './artifact-file-name.ts';
import { isDigestShaped, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Artifact placement
// How ONE settled artifact answers "which pipeline produced you".
//
// Split from the census when generation identity moved from the recorded commit
// to the digest of the built output: placing one file became a question with
// four answers rather than two, and the census had no room left for it. The
// census aggregates; this decides.

/**
 Characters in a SHA-1 object id, the shorter of the two git uses.
 */
const SHA1_LENGTH = 40;

/**
 Characters in a SHA-256 object id, for repositories using that hash.
 */
const SHA256_LENGTH = 64;

/**
 Whether a recorded tip is a canonical full object id.

 A nonempty string was the whole test before, which accepted ` `, `HEAD`,
 `main` and any revision expression. Those are not identities: `HEAD` in a
 settled artifact resolves against the READER's checkout at read time rather
 than against whatever produced the artifact, so it silently answers a
 different question than the one asked, and a branch name answers a question
 whose answer changes.

 Scanned rather than matched with a pattern: the rule is one predicate per
 character over a fixed-length string, which is a linear pass that cannot
 backtrack, and the codebase forbids a regex where an index scan says the
 same thing.

 @param value - tip as the artifact recorded it

 @returns Whether it is 40 or 64 lowercase hex characters

 @example
 ```ts
 const usable = isObjectId({ value: 'a41fc607ea5a70d8a7625cc67d5ed8c444f53379', },);
 ```
 */
function isObjectId({ value, }: { readonly value: string; },): boolean {
  if ((value.length !== SHA1_LENGTH) && (value.length !== SHA256_LENGTH))
    return false;

  for (const character of value) {
    // Upper case is refused deliberately: git writes lower case, so an
    // upper-case id came from somewhere else, and two spellings of one commit
    // would count as two generations.
    if (!isLowerHexDigit({ character, },))
      return false;
  }

  return true;
}

/**
 Names a recorded value by its shape alone.

 THE VALUE ITSELF IS NEVER PRINTED. A malformed `id` or digest is not an entry
 id, it is whatever bytes a bad file carries, and `readPlacement` runs inside
 the pass as well as in the readers, so its lines reach a pass's stdout. This
 follows `readRunJson`: say what kind of thing was found and how large, which
 is what tells an operator a file was truncated or rewritten, and carry none
 of it across.

 @param value - field as the artifact recorded it, possibly absent

 @returns Type name, with the length for a string

 @example
 ```ts
 const shape = shapeOf({ value: recordedId, },);
 ```
 */
function shapeOf({ value, }: { readonly value: unknown; },): string {
  if (value === undefined)
    return 'absent';
  if (value === null)
    return 'null';
  if ((typeof value) === 'string')
    return `a string of ${String(value.length,)} ${
      wordForCount({
        count: value.length,
        one: 'character',
        many: 'characters',
      },)
    }`;
  if (Array.isArray(value,))
    return 'an array';
  if ((typeof value) === 'object')
    return 'an object';
  return `a ${typeof value}`;
}

/**
 How one artifact places into a generation.

 @example
 ```ts
 const placement: Placement = { kind: 'legacy', tip, };
 ```
 */
export type Placement =
  | Readonly<{
    /**
     Artifact records both what ran and where it came from.
     */
    kind: 'placed';

    /**
     Repo commit its pass started under.
     */
    tip: string;

    /**
     Built output its pass executed, which is the generation.
     */
    digest: string;
  }>
    | Readonly<{
    /**
     Artifact records a usable commit, and a pipeline THIS BUILD CANNOT NAME:
     either none at all, from before generation identity existed, or one
     written in a digest scheme this build does not read.

     Kept apart from `untagged` because the remedy differs: an untagged file
     is deleted, while these are perfectly good results whose pipeline can no
     longer be named, and the remedy is a fresh directory.

     Kept apart from a foreign generation for the same reason in the other
     direction: checking out an old commit does not recreate an identity the
     artifact never carried.
     */
      kind: 'legacy';

    /**
     Repo commit its pass started under, all the provenance it has.
     */
      tip: string;
    }>
    | Readonly<{
    /**
     Artifact would not parse at all.
     */
      kind: 'malformed';
    }>
    | Readonly<{
    /**
     Artifact parsed but recorded nothing that could identify it.
     */
      kind: 'untagged';
    }>;

/**
 Reads which pipeline one artifact records.

 Reports rather than throws, because this package already decided a corrupt
 artifact costs its own row and not the whole run. The failure kinds stay
 distinct because they are handled oppositely: a malformed file belongs to the
 reader that reports malformed files, an untagged one belongs nowhere, and a
 legacy one is a fine result whose pipeline can no longer be named.

 @param artifactsDir - directory holding the artifact

 @param name - artifact file name

 @returns How this artifact places

 @example
 ```ts
 const placement = await readPlacement({ artifactsDir, name: 'Acheron.json', },);
 ```
 */
export async function readPlacement(
  {
    artifactsDir,
    name,
  }: {
    readonly artifactsDir: string;
    readonly name: ArtifactFileName;
  },
): Promise<Placement> {
  /**
   Entry id the pool will key this artifact by, which is its file name.
   */
  const keyedId = entryIdOfArtifact({ name, },);

  if (keyedId === '')
    return { kind: 'untagged', };

  try {
    // INSIDE the try, deliberately. It used to sit outside, so a vanished
    // file, an unreadable one, or a directory named `something.json` threw
    // out of the whole census instead of costing its own row. That is the
    // opposite of this module's stated policy, and it aborts a pass at
    // startup now that the resume guard runs the census.
    /**
     Artifact as parsed JSON.

     READ AND PARSED THROUGH ONE GUARD. Opening was a bare `readFile` until
     2026-08-25, so a file that would not open arrived at this function's `catch` as an
     ordinary `Error` whose message quotes the whole path, and the only safe
     thing to say about it was `refused by Error`. This names the filesystem
     code, and names the file by base name.
     */
    const parsed: unknown = await readRunJson({
      path: `${artifactsDir}/${name}`,
    },);

    if (!isJsonRecord(parsed,))
      return { kind: 'untagged', };

    // The file name is what the pool keys on and what the scheduler calls
    // settled, while every reader downstream uses the id INSIDE. Unequal means
    // one artifact would be admitted under one identity and read under
    // another, which is how `Mittens-copy.json` becomes a second settled entry
    // and `Mittens.json.json` becomes an entry called `Mittens.json`.
    // Presence is required, not merely agreement. Guarding the comparison with
    // `'id' in parsed` meant an artifact carrying no id at all skipped the
    // check entirely and was placed on its file name alone, which is the one
    // reading the check exists to refuse: the pool would admit it under a name
    // the bytes never claimed.
    /**
     Entry id these bytes claim, absent when they claim none.
     */
    const recordedId: unknown = ('id' in parsed) ? parsed.id : undefined;

    // NAMED BY SHAPE, never echoed: the recorded id of a file that fails this
    // check is not an entry id, so the license every other reader has to print
    // one deliberately does not reach it.
    if (recordedId !== keyedId) {
      console.log(
        `POOL ${name} records an id that is not its file name (${
          shapeOf({ value: recordedId, },)
        }); treating it as unplaceable`,
      );
      return { kind: 'untagged', };
    }

    if (!('tip' in parsed))
      return { kind: 'untagged', };

    /**
     Commit as the artifact recorded it.
     */
    const { tip, } = parsed;

    if (((typeof tip) !== 'string') || (!isObjectId({ value: tip, },)))
      return { kind: 'untagged', };

    if (!('pipelineDigest' in parsed))
      return {
        kind: 'legacy',
        tip,
      };

    /**
     Built output as the artifact recorded it.
     */
    const { pipelineDigest, } = parsed;

    if ((typeof pipelineDigest) !== 'string') {
      console.log(
        `POOL ${name} records a pipeline digest that is not a string (${
          shapeOf({ value: pipelineDigest, },)
        }); treating it as unplaceable`,
      );
      return { kind: 'untagged', };
    }

    // A digest this build cannot read is treated as LEGACY rather than as
    // garbage. The recorded value names the scheme that produced it, so a
    // string it cannot parse is most likely an older scheme, and an artifact
    // written by an earlier version of this package is a sound result whose
    // pipeline can no longer be named. Calling it unplaceable would tell an
    // operator to delete good work.
    if (!isDigestShaped({ value: pipelineDigest, },)) {
      console.log(
        `POOL ${name} records a pipeline digest this build cannot read (${
          shapeOf({ value: pipelineDigest, },)
        }); treating it as legacy`,
      );
      return {
        kind: 'legacy',
        tip,
      };
    }

    return {
      kind: 'placed',
      tip,
      digest: pipelineDigest,
    };
  }
  catch (error) {
    // Logged rather than swallowed so a systematic write fault is visible
    // instead of showing up as a quietly smaller pool. A file that vanished or
    // could not be opened lands here too, now that the read happens inside the
    // try.
    //
    // A truncated artifact is NO LONGER the ordinary trace of a pass killed at
    // its hard cap, which is what this comment used to say. The final artifact
    // is written after the entry completes, and it is written to a temporary
    // name and renamed, so a kill mid-write leaves a `.partial` rather than a
    // half-written `.json`. Reaching this now means something else: a legacy
    // artifact from before atomic writes, a concurrent writer, a permission
    // fault, or storage failure. Treating it as routine would normalise every
    // one of those.
    console.log(
      `POOL malformed ${name}: ${refusalText({ error, },)}`,
    );
    return { kind: 'malformed', };
  }
}

//endregion Artifact placement
