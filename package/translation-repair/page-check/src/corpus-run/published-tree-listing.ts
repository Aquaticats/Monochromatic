import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import { access, } from 'node:fs/promises';
import { join, } from 'node:path';

import { rethrowUnlessMissingPath, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  artifactFilesIn,
  artifactsDirOf,
  entryIdOfArtifact,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  type DirectoryReading,
  namesIn,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  ENGLISH_PAGE_FILE,
  FIXED_TREE_DIR,
  PEOPLE_DIR,
} from './publish-fixed.ts';

//region Published tree listing
// What a run left on disk, and whether it left anything worth checking.
//
// SPLIT OUT OF `verify-published.ts` FOR THE DEFECT this
// module exists to make impossible. The listing used to answer an absent
// directory with an empty array and print the absence on STDERR, so a run with
// no artifacts directory at all printed the same stdout summary as a run whose
// every page agreed with its artifact, and left the same exit code behind. A
// verification that verified nothing has to be readable as one, both by a
// person reading the report and by whatever runs it as a gate.
//
// THE TWO ABSENCES ARE NOT THE SAME ANSWER, which is why the verdict is a
// function rather than a pair of guards at the call site. No artifacts
// directory is nothing to check. No published tree BESIDE REAL ARTIFACTS is
// every settled entry unpublished, which is the most serious finding this
// check can make, so it must stay a finding rather than collapse into silence.
//
// PRINTS AND RETURNS IDS, NAMES AND COUNTS. Never a passage, because a run
// directory holds unlicensed corpus wording.

/**
 What a run leaves to verify, or why it leaves nothing.

 @example
 ```ts
 const run: VerifiableRun = { kind: 'nothing-verified', why: 'no run', };
 ```
 */
export type VerifiableRun =
  | {
    readonly kind: 'checkable';

    /**
     Entry ids the run settled, sorted.
     */
    readonly settled: readonly string[];

    /**
     Entry ids the run published, sorted, empty where the tree is absent.
     */
    readonly published: readonly string[];
  }
    | {
      readonly kind: 'nothing-verified';

    /**
     Why nothing could be checked, phrased as a clause a report can carry.
     */
      readonly why: string;
    };

/**
 Lists the entries a run settled, by the artifacts it wrote.

 Listed as the census and the scheduler list them, regular files only, so a
 directory or a symlink named like an artifact is never an entry to verify or
 republish (ledger B64).

 @param runsDir - run directory holding the artifacts

 @returns Entry ids, sorted, or why the artifacts directory could not be read

 @example
 ```ts
 const settled = await settledEntryIds({ runsDir, },);
 ```
 */
export async function settledEntryIds(
  { runsDir, }: { readonly runsDir: string; },
): Promise<DirectoryReading> {
  /**
   The artifacts the directory holds, or why it holds nothing here.
   */
  const listing = await artifactFilesIn({ dir: artifactsDirOf({ runsDir, },), },);

  if (listing.kind === 'unreadable')
    return listing;

  return {
    kind: 'read',
    names: textsInCodePointOrder({ texts: listing
      .names
      .map(function toId(name,): string {
        return entryIdOfArtifact({ name, },);
      },), },),
  };
}

/**
 Whether an entry's directory in the published tree holds its page.

 @param peopleDir - people directory of the published tree

 @param entryId - entry to look for

 @returns Whether the page file is there

 @example
 ```ts
 const there = await carriesPage({ peopleDir, entryId, },);
 ```
 */
async function carriesPage(
  {
    peopleDir,
    entryId,
  }: {
    readonly peopleDir: string;
    readonly entryId: string;
  },
): Promise<boolean> {
  try {
    await access(join(
      peopleDir,
      entryId,
      ENGLISH_PAGE_FILE,
    ),);
    return true;
  }
  catch (error) {
    rethrowUnlessMissingPath({ error, },);
    return false;
  }
}

/**
 Lists the entries a run published, by the pages it wrote.

 BY THE PAGE FILE, NOT THE ENTRY'S DIRECTORY (ledger A16b). A directory whose
 page is gone read as published, so the entry was paired as matched, its read
 then failed, and the report said `REFUSED by Error` where it should have said
 the entry was settled and never published.

 @param runsDir - run directory holding the fixed tree

 @returns Entry ids, sorted, or why the published tree could not be read

 @example
 ```ts
 const published = await publishedEntryIds({ runsDir, },);
 ```
 */
export async function publishedEntryIds(
  { runsDir, }: { readonly runsDir: string; },
): Promise<DirectoryReading> {
  /**
   People directory of the fixed tree.
   */
  const peopleDir = join(
    runsDir,
    FIXED_TREE_DIR,
    PEOPLE_DIR,
  );
  /**
   Entry directories it holds. Only a directory is an entry the pass
   published: a file there raised ENOTDIR when its page was looked for, and a
   symlink counted a page under a second name (ledger B65).
   */
  const reading = await namesIn({
    dir: peopleDir,
    kind: 'directory',
  },);

  if (reading.kind === 'unreadable')
    return reading;

  /**
   Entry directories, by entry id.
   */
  const { names, } = reading;

  /**
   Each entry directory beside whether it holds its page.
   */
  const carried = await allInInputOrder({
    members: names.map(async function check(entryId,): Promise<{
      readonly entryId: string;
      readonly hasPage: boolean;
    }> {
      return {
        entryId,
        hasPage: await carriesPage({
          peopleDir,
          entryId,
        },),
      };
    },),
  },);

  return {
    kind: 'read',
    names: textsInCodePointOrder({ texts: carried
      .filter(function holdsPage({ hasPage, },): boolean {
        return hasPage;
      },)
      .map(function idOf({ entryId, },): string {
        return entryId;
      },), },),
  };
}

/**
 Decides whether a run has anything to verify at all.

 TWO WAYS TO VERIFY NOTHING, and both have to leave a verdict that says so,
 which is the verifier's one nonzero exit. An artifacts directory that is not
 there means the caller is pointed at something that is not a run. An
 artifacts directory holding no artifact means the run settled no entry.
 Neither is a clean run, and both once read as one.

 AN ABSENT PUBLISHED TREE IS DELIBERATELY NOT ONE OF THEM. Beside real
 artifacts it means every settled entry was never published: no pass settles
 those entries again, and the archive ships for each until the next pass
 started in the runs directory writes its page from the artifact (ledger
 A16c). Reporting that as an empty tree keeps it a finding the caller counts,
 rather than a silence that ends the report.

 @param settled - what the artifacts directory listed

 @param published - what the published tree listed

 @returns Ids to check, or why there are none

 @example
 ```ts
 const run = whatThereIsToVerify({ settled, published, },);
 ```
 */
export function whatThereIsToVerify(
  {
    settled,
    published,
  }: {
    readonly settled: DirectoryReading;
    readonly published: DirectoryReading;
  },
): VerifiableRun {
  if (settled.kind === 'unreadable')
    return {
      kind: 'nothing-verified',
      why: `no artifacts directory under the run (${settled.reason})`,
    };

  /**
   How many artifacts the directory turned out to hold.
   */
  const settledCount = settled
    .names
    .length;

  if (settledCount === 0)
    return {
      kind: 'nothing-verified',
      why: 'the artifacts directory holds no settled artifact',
    };

  return {
    kind: 'checkable',
    settled: settled.names,
    published: (published.kind === 'read')
      ? published.names
      : [],
  };
}

//endregion Published tree listing
