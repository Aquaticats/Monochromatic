import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  artifactFileNameOf,
  artifactsDirOf,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { parseSettledTwoLaneArtifact, } from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  ENGLISH_PAGE_FILE,
  type EntryAgreement,
  FIXED_TREE_DIR,
  pageAgreement,
  type PageLengthCheck,
  PEOPLE_DIR,
} from '@monochromatic-dev/translation-repair-page-check/ts';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Verify published entry
// One entry's artifact and page, read and judged by the one definition a pass
// also republishes on (`page-agreement.ts`), and printed as one line plus a line
// for each finding. Moved out of `verify-published.ts`, which keeps the wiring.
//
// PRINTS IDS, INDICES AND COUNTS. Never a passage, never a parse-refusal
// message, because those quote the text they disagree about and a run directory
// holds unlicensed corpus wording.

/**
 Reads one entry's artifact and page, or names the class that refused them.

 @param runsDir - run directory both halves live under

 @param entryId - person entry to read

 @returns Both halves, or the refusal

 @example
 ```ts
 const read = await readEntry({ runsDir, entryId, },);
 ```
 */
async function readEntry(
  {
    runsDir,
    entryId,
  }: {
    readonly runsDir: string;
    readonly entryId: string;
  },
): Promise<
  | {
    readonly kind: 'read';
    readonly artifact: ReturnType<typeof parseSettledTwoLaneArtifact>;
    readonly pageText: string;
  }
    | {
      readonly kind: 'refused';

    /**
     Class that refused this entry, named rather than quoted: a refusal
     message quotes the passage it disagrees about.
     */
      readonly refusedBy: string;
    }
  > {
  try {
    return {
      kind: 'read',
      artifact: parseSettledTwoLaneArtifact({
        value: JSON.parse(await readFile(
          join(
            artifactsDirOf({ runsDir, },),
            artifactFileNameOf({ entryId, },),
          ),
          'utf8',
        ),) as unknown,
      },),
      pageText: await readFile(
        join(
          runsDir,
          FIXED_TREE_DIR,
          PEOPLE_DIR,
          entryId,
          ENGLISH_PAGE_FILE,
        ),
        'utf8',
      ),
    };
  } catch (error) {
    return {
      kind: 'refused',
      refusedBy: errorName({ error, },),
    };
  }
}

/**
 Renders the length column, which says three different things.

 NAMES AN UNWEIGHED ENTRY RATHER THAN PRINTING ITS SIZE, so a run of
 artifacts written before the archive text was stored cannot be read as a run
 that was checked and agreed.

 @param weight - what `pageWeighsWhatItShould` returned

 @returns Column text for the entry line

 @example
 ```ts
 console.log(weighedAs({ weight, },),);
 ```
 */
function weighedAs(
  { weight, }: { readonly weight: PageLengthCheck; },
): string {
  if (weight.kind === 'unweighable')
    return 'chars=UNWEIGHED(artifact predates stored archive text)';

  /**
   Expected length, or a mark saying the page already matches it.
   */
  const against = (weight.actual === weight.expected)
    ? '=expected'
    : `/expected ${String(weight.expected,)}`;

  /**
   Note that a filled anchor makes the expectation a floor.
   */
  const floor = weight.exact ? '' : '+separators';

  return `chars=${String(weight.actual,)}${against}${floor}`;
}

/**
 Renders the column that says whether the artifact's consistency section was
 read and at which generation.

 PRINTS TWO COUNTS, NEVER A WORDING. Without it a replay of an old artifact
 prints nothing about the section, and a section that was never read looks the
 same as one that was read and held no group.

 @param groups - how many groups of repeated wording the section holds

 @param generation - generation the artifact was read at

 @returns Column text for the entry line

 @example
 ```ts
 console.log(consistencyColumnOf({ groups: 2, generation: 16, },),);
 ```
 */
export function consistencyColumnOf(
  {
    groups,
    generation,
  }: {
    readonly groups: number;
    readonly generation: number;
  },
): string {
  return `consistency-groups=${String(groups,)} read-at-generation=${String(generation,)}`;
}

/**
 What one entry came to: whether its page agreed with its artifact, and the
 lines that say so.

 @example
 ```ts
 const judged: JudgedEntry = { agreement: 'disagreed', lines: ['CatEntry1: REFUSED by SyntaxError',], };
 ```
 */
export type JudgedEntry = {
  /**
   Whether the page carries everything the artifact promised.
   */
  readonly agreement: EntryAgreement;

  /**
   Lines the report prints for this entry, in order, none printed yet: the
   run prints them in its own entry order, whichever read finishes first.
   */
  readonly lines: readonly string[];
};

/**
 Judges one entry, returning whether its page agreed with its artifact and
 the lines that report it.

 @param runsDir - run directory both halves live under

 @param entryId - person entry to read

 @param thisBuild - build doing the reading, so a disagreement over an
 artifact another build settled says the reading moved

 @returns Whether the page carries everything the artifact promised, with
 the lines to print

 @example
 ```ts
 const { agreement, lines, } = await judgePublishedEntry({ runsDir, entryId, thisBuild, },);
 ```
 */
export async function judgePublishedEntry(
  {
    runsDir,
    entryId,
    thisBuild,
  }: {
    readonly runsDir: string;
    readonly entryId: string;
    readonly thisBuild: PipelineDigest;
  },
): Promise<JudgedEntry> {
  /**
   Artifact and page as they sit on disk, or the class that refused them.
   */
  const read = await readEntry({
    runsDir,
    entryId,
  },);

  if (read.kind === 'refused') {
    return {
      agreement: 'disagreed',
      lines: [`${entryId}: REFUSED by ${read.refusedBy}`,],
    };
  }

  /**
   The page judged against its artifact, by the one definition a pass also
   republishes on (`page-agreement.ts`).
   */
  const {
    agreement,
    wording: {
      wordings,
      silentSlices,
      sourceShipped,
      missing,
    },
    weight,
    wrongLength,
  } = pageAgreement({
    artifact: read.artifact,
    pageText: read.pageText,
  },);

  // A PAGE ANOTHER BUILD SETTLED AGREED WITH THAT BUILD, since each publish
  // refuses a page that disagrees; this build reads the artifact through its own
  // typography, so a disagreement here can be the reading that moved (77 of 214
  // stored pages on 2026-09-27, 69 by the class 181 and 147 fixes alone).
  /**
   Build that settled the artifact.
   */
  const { pipelineDigest: settledBy, } = read.artifact;

  return {
    agreement,
    lines: [
      `${entryId}: wordings=${String(wordings,)} silent=${String(silentSlices,)} source-shipped=${
        String(sourceShipped,)
      } `
        + `${weighedAs({ weight, },)} missing=${String(missing.length,)} ${
        consistencyColumnOf({
          groups: read.artifact
            .consistency
            .groups
            .length,
          generation: read.artifact
            .artifactSchemaVersion,
        },)
      }`,
      ...((wrongLength && (weight.kind === 'weighed'))
        ? [
          // THE NOUN FOLLOWS THE SIZE OF THE DIFFERENCE, not its sign: a page
          // one character short is "-1 character", not "-1 characters".
          `  WRONG LENGTH: page is ${String(weight.actual - weight.expected,)} ${
            wordForCount({
              count: Math.abs(weight.actual - weight.expected,),
              one: 'character',
              many: 'characters',
            },)
          } off what the `
            + 'archive plus every slice change comes to. Text no slice decided on was lost or added',
        ]
        : []),
      ...missing.map(function missingLine(gone,): string {
        return `  MISSING slice ${String(gone.sliceIndex,)}, ${String(gone.characters,)} ${
          wordForCount({
            count: gone.characters,
            one: 'character',
            many: 'characters',
          },)
        } the page `
          + 'does not carry in order';
      },),
      ...(((agreement === 'disagreed') && (settledBy !== thisBuild))
        ? [
          `  READ BY ANOTHER BUILD: settled by ${settledBy}, read by ${thisBuild}; a pass `
            + 'started here on this build rewrites the page to this reading',
        ]
        : []),
    ],
  };
}

//endregion Verify published entry
