import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  isLowerHexDigit,
  textsInCodePointOrder,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { createHash, } from 'node:crypto';
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import {
  join,
  relative,
} from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Pipeline digest
// WHICH BYTES RAN, as against which commit happened to be checked out.
//
// Every artifact records `tip`, the git HEAD object id at the moment its pass
// started. That is worth keeping as provenance, and it has ancestry, which a
// digest cannot have. It is not an identity for executed code in either
// direction:
//
//   A docs-only commit moves HEAD without changing a byte that runs. Two
//   behaviourally identical passes then land in different generations, neither
//   resumable into the other, and a pool that should have been one is refused.
//
//   An uncommitted edit changes what runs without moving HEAD. Two genuinely
//   different pipelines then record the same tip and are pooled silently. That
//   direction is the dangerous one, because nothing anywhere reports it and the
//   rate that comes out describes no pipeline that ever existed.
//
// Every corpus-run task now builds first and runs its built file under
// `dist/final/node`, so the bytes that execute are a directory and a digest over
// that directory answers the question `tip` was standing in for.
//
// Measured on this build before the module was written: two clean rebuilds of
// unchanged source produced byte-identical output across all 69 emitted files,
// and a file planted in the output directory was gone after the next build, so
// the digest is a function of the source rather than of build history.
//
// WHAT IT DOES NOT COVER, stated because a digest invites the reading that it
// covers everything, and every item here can change a result while it stays
// fixed:
//
//   node_modules. The emitted files still import `yaml`, `unified`,
//   `remark-parse`, `remark-mdx`, `remark-gfm` and `p-limit` as bare
//   specifiers, measured 2026-08-15, so a dependency upgrade changes markdown
//   parsing with nothing here moving. Closing that means bundling runtime
//   dependencies, which is repo-wide build configuration rather than a change
//   to this package.
//
//   The Node binary, and anything it decides: version, flags, locale.
//
//   The provider's models. Two runs of one pipeline against `Kimi-K3` are not
//   two runs of the same system if the provider changed the weights, and
//   nothing observable from here says whether it did.
//
//   The corpus. That is covered separately and exactly, by the pinned commit
//   each artifact records as `corpusSha`.
//
//   Run configuration read from the environment. The slice cache keys on it
//   separately.
//
// So this identifies the first-party bytes that ran. That is the half that
// moved every day and was invisible; the rest either moves rarely or is
// recorded elsewhere.

/**
 Hash behind both the per-file and the combined digest.
 */
const DIGEST_ALGORITHM = 'sha256';

/**
 Characters of a sha256 hex digest.
 */
const DIGEST_LENGTH = 64;

/**
 Name of the scheme a recorded digest was produced by.

 Carried in the value itself rather than assumed, so a later scheme is a
 DIFFERENT string rather than a same-looking one. Without it, changing what is
 hashed or how it is framed would silently make two incomparable values
 comparable, which is the failure this whole module exists to prevent, and an
 artifact recorded under the old scheme would be read as a foreign generation
 rather than as one this build cannot name.
 */
const DIGEST_FORMAT = 'sha256-tree-v1';

/**
 Character between the scheme name and the hex, chosen because neither side
 can contain it.
 */
const FORMAT_SEPARATOR = ':';

/**
 What a recorded digest looks like, in words, for every message refusing one.

 BUILT FROM THE CONSTANTS the check reads, so a new scheme or length changes
 the sentence with it. The sentence was written by hand once and described
 the hex half as the whole value, omitting the scheme name every digest
 starts with (ledger B34).
 */
export const PIPELINE_DIGEST_SHAPE: string = `${JSON.stringify(`${DIGEST_FORMAT}${FORMAT_SEPARATOR}`,)} followed by ${
  String(DIGEST_LENGTH,)
} lowercase hex characters`;

/**
 Suffixes of emitted files that cannot execute, so cannot change behaviour.

 TypeScript declarations are excluded deliberately rather than for tidiness.
 The built `.mjs` carries no comments at all, while a `.d.mts` carries every
 TSDoc block verbatim, so including declarations would make a comment-only edit
 a new pipeline generation and force a fresh accumulation directory for a
 change that cannot alter a single result.
 */
const DECLARATION_SUFFIXES = [
  '.d.mts',
  '.d.ts',
  '.d.cts',
] as const;

/**
 Byte between a path and its hash, chosen because a path cannot contain it.

 Without a separator no path can carry, the concatenation is ambiguous: a file
 `ab` hashing to `cd…` and a file `a` hashing to `bcd…` would feed the combined
 hash identical input, so two different builds could claim one identity.
 */
const PATH_TERMINATOR = '\u0000';

/**
 Digest naming the built pipeline that produced a result.

 Branded so it cannot be assigned where a git object id belongs. Both are
 strings to the type system, and the scheme name this one starts with is a
 runtime difference no signature sees; they answer different questions: this
 one is NOT a commit and has no ancestry, no log entry, and nothing to check
 out.

 @example
 ```ts
 const { digest, }: { digest: PipelineDigest; } = await digestPipeline({ dir, },);
 ```
 */
export type PipelineDigest = string & { readonly __brand: 'PipelineDigest'; };

/**
 What a pass records about the code that produced its artifacts.

 Carries the file count beside the digest so a log line can say what the digest
 was taken over. A digest alone is unfalsifiable in a log: `files=51` next to it
 turns a truncated or empty output directory into something a reader notices.

 @example
 ```ts
 const stamp = await digestPipeline({ dir, },);
 console.log(`digest=${stamp.digest} files=${String(stamp.fileCount,)}`,);
 ```
 */
export type PipelineStamp = Readonly<{
  /**
   Identity of the executed build.
   */
  digest: PipelineDigest;

  /**
   Emitted files it was taken over.
   */
  fileCount: number;
}>;

/**
 Raised when the built pipeline cannot be identified.
 */
export class PipelineDigestError extends Error {
  /**
   Declares this message safe to forward: it names the directory and what is missing from it.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names the directory, what was wrong with it, and why that stops a pass.

   @param dir - directory the digest was to be taken over

   @param reason - what made it unusable, as a clause

   @example
   ```ts
   throw new PipelineDigestError({ dir, reason: 'it holds no file that runs', },);
   ```
   */
  constructor(
    {
      dir,
      reason,
    }: {
      readonly dir: string;
      readonly reason: string;
    },
  ) {
    super(
      [
        `Cannot identify the built pipeline in ${dir}: ${reason}.`,
        '',
        'This digest is what every artifact records as the generation that',
        'produced it, so a pass that cannot compute one would settle entries',
        'stamped with nothing and the pool could not tell its versions apart.',
        '',
        'Every corpus-run task depends on the build and runs the built file, so',
        'reaching this means the build did not run or its output was moved.',
      ].join('\n',),
    );
    this.name = 'PipelineDigestError';
  }
}

/**
 Narrows a string already known to be a digest of the built pipeline.

 An assertion function rather than a cast, because `no-unsafe-type-assertion`
 refuses a narrowing `as` and this repository names assertion functions as the
 mechanism for runtime narrowing.

 @param value - digest to narrow

 @returns Nothing; it narrows `value` in the caller on success

 @throws {@link TypeError} when it is not the scheme name, the separator and
 64 lowercase hex characters

 @example
 ```ts
 assertPipelineDigest(recorded,);
 ```
 */
export function assertPipelineDigest(
  value: string,
): asserts value is PipelineDigest {
  if (!isDigestShaped({ value, },))
    throw new TypeError(`A pipeline digest is ${PIPELINE_DIGEST_SHAPE}; received ${JSON.stringify(value,)}.`,);
}

/**
 Whether a string could be a digest this module produced.

 Scanned rather than matched with a pattern: the rule is one predicate per
 character over a fixed-length string, a linear pass that cannot backtrack, and
 this codebase forbids a regex where an index scan says the same thing.

 @param value - string to test

 @returns Whether it is the scheme name, the separator and 64 lowercase hex
 characters

 @example
 ```ts
 const usable = isDigestShaped({ value: recorded, },);
 ```
 */
export function isDigestShaped(
  { value, }: { readonly value: string; },
): boolean {
  if (!value.startsWith(`${DIGEST_FORMAT}${FORMAT_SEPARATOR}`,))
    return false;

  /**
   Hex half, once the scheme name is off.
   */
  const hex = value.slice(
    `${DIGEST_FORMAT}${FORMAT_SEPARATOR}`.length,
  );

  if (hex.length !== DIGEST_LENGTH)
    return false;

  for (const character of hex) {
    // Upper case is refused because this module only ever emits lower case,
    // so another spelling came from elsewhere and would count as a second
    // generation.
    if (!isLowerHexDigit({ character, },))
      return false;
  }

  return true;
}

/**
 Whether an emitted file is a TypeScript declaration.

 @param name - file name as the directory reported it

 @returns Whether it carries a declaration suffix

 @example
 ```ts
 const skipped = isDeclarationFile({ name: 'index.d.mts', },);
 ```
 */
function isDeclarationFile({ name, }: { readonly name: string; },): boolean {
  return DECLARATION_SUFFIXES.some(function carries(suffix,): boolean {
    return name.endsWith(suffix,);
  },);
}

/**
 Identifies the built pipeline in a directory by what it holds.

 Order-independent by construction: each file contributes one line pairing its
 path with the hash of its bytes, and the lines are sorted before the combined
 hash sees them, so a directory read in a different order yields one digest.

 @param dir - directory of built output, ordinarily `import.meta.dirname` of a
 runner, which resolves to `dist/final/node`

 @returns Digest of the executable files it holds, with how many there were

 @throws {@link PipelineDigestError} when the directory holds a symbolic link,
 or holds no file that could execute

 @example
 ```ts
 const { digest, fileCount, } = await digestPipeline({ dir: import.meta.dirname, },);
 ```
 */
export async function digestPipeline(
  { dir, }: { readonly dir: string; },
): Promise<PipelineStamp> {
  /**
   Everything the build left behind, at any depth.
   */
  const entries = await readdir(
    dir,
    {
      recursive: true,
      withFileTypes: true,
    },
  );

  // A link is refused rather than followed or skipped. The build emits none, so
  // one being here means something else wrote into the output directory, and
  // both other answers are wrong: following it digests bytes from outside the
  // pipeline, and skipping it silently drops code that will run.
  /**
   Entries that are neither a regular file nor a directory: links, sockets,
   fifos, devices. The build emits none of them, and each fails the same way,
   by being something Node may load whose bytes this cannot identify.
   */
  const foreign = entries.filter(function isForeign(entry,): boolean {
    return (!entry.isFile()) && (!entry.isDirectory());
  },);

  if (foreign.length > 0) {
    /**
     One of them, named so the message points somewhere: the first by name,
     since the directory reports its entries in no promised order and the
     same directory should give the same message. The list is not empty here,
     so there is one.
     */
    const firstName = nonNullishOrThrow(
      textsInCodePointOrder({ texts: foreign
        .map(function nameOf(entry,): string {
          return entry.name;
        },), },)[0],
    );

    throw new PipelineDigestError({
      dir,
      reason: `it holds ${String(foreign.length,)} ${
        wordForCount({
          count: foreign.length,
          one: 'entry that is',
          many: 'entries that are',
        },)
      } neither a regular file nor a directory, such as ${firstName}, which the build never emits`,
    },);
  }

  /**
   Emitted files that can actually execute.
   */
  const files = entries.filter(function runs(entry,): boolean {
    return entry.isFile() && (!isDeclarationFile({ name: entry.name, },));
  },);

  if (files.length === 0)
    throw new PipelineDigestError({
      dir,
      reason: 'it holds no file that could execute',
    },);

  /**
   One line per file, pairing its path with the hash of its bytes.
   */
  const lines = await allInInputOrder({
    members: files.map(async function toLine(entry,): Promise<string> {
      /**
       Absolute path of this emitted file.
       */
      const path = join(
        entry.parentPath,
        entry.name,
      );

      /**
       Hash of exactly the bytes that will be executed.
       */
      const hash = createHash(DIGEST_ALGORITHM,)
        .update(await readFile(path,),)
        .digest('hex',);

      return `${
        relative(
          dir,
          path,
        )
      }${PATH_TERMINATOR}${hash}\n`;
    },),
  },);

  /**
   Lines in one order whatever order the directory reported its entries in.
   */
  const ordered = textsInCodePointOrder({ texts: lines, },);

  /**
   Combined hash over the sorted per-file lines.
   */
  const hex = createHash(DIGEST_ALGORITHM,)
    .update(ordered.join('',),)
    .digest('hex',);

  /**
   Recorded value, naming the scheme that produced it.
   */
  const digest = `${DIGEST_FORMAT}${FORMAT_SEPARATOR}${hex}`;

  assertPipelineDigest(digest,);

  return {
    digest,
    fileCount: files.length,
  };
}

//endregion Pipeline digest
