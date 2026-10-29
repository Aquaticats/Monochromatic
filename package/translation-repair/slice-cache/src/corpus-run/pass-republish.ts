import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { readCorpusFile, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  type CorpusPairReader,
  republishSettledPages,
  settledEntryIds,
} from '@monochromatic-dev/translation-repair-page-check/ts';
import { readSettledEntryInfo, } from '@monochromatic-dev/translation-repair-artifact/ts';
import { removeDeclinedPages, } from './pass-decline.ts';

//region Pass republish
// The pass's side of ledger A16c: before any entry runs, every page the
// artifacts in this runs directory say should ship differently, or that is
// missing, is rewritten from its artifact (`page-republish.ts`). Split out of
// `corpus-pass.ts`, which keeps to its line budget.

/**
 Reads an entry's pair from the run's corpus clone at the commit its artifact
 was settled at, which the run pin need not share.

 @param cloneDir - clone the run reads

 @param entryId - entry whose pair to read

 @param corpusSha - commit its artifact was settled at

 @param generation - generation of its artifact, which decides whether the
 entry's `info.yml` is read at all

 @param l - logger the absence of an `info.yml` is named on

 @returns Original and archive's English at that commit, and
 entry's parsed `info.yml` where the artifact's generation reads it and the
 entry has one

 @example
 ```ts
 const pair = await readPinnedPair({ cloneDir, entryId: 'tabby', corpusSha, generation: 15, l, },);
 ```
 */
async function readPinnedPair(
  {
    cloneDir,
    entryId,
    corpusSha,
    generation,
    l,
  }: Parameters<CorpusPairReader>[0] & {
    readonly cloneDir: string;
    readonly l: Logger;
  },
): ReturnType<CorpusPairReader> {
  /**
   The clone the run reads, at the artifact's own commit.
   */
  const pin = {
    cloneDir,
    commitSha: corpusSha,
  };
  /**
   The original and the archive's English at that commit.
   */
  const [
    sourceText,
    targetText,
    entryInfo,
  ] = await allInInputOrder({
    members: [
      readCorpusFile({
        pin,
        relPath: `people/${entryId}/page.md`,
      },),
      readCorpusFile({
        pin,
        relPath: `people/${entryId}/page.en.md`,
      },),
      readSettledEntryInfo({
        generation,
        entryId,
        pin,
        readFile: readCorpusFile,
        l,
      },),
    ],
  },);
  return {
    sourceText,
    targetText,
    ...((entryInfo.kind === 'read') ? { sourceEntryInfo: entryInfo.info, } : {}),
  };
}

/**
 Rewrites from its artifact every page in a runs directory that is missing or
 disagrees with what the artifact ships, and removes every page standing for
 a declined entry, before the pass settles anything.

 Never stops the pass on a page it cannot rewrite, or on an artifacts
 directory it cannot list: each is logged and the pass goes on.

 @param runsDir - run directory holding the artifacts

 @param artifactsDir - settled artifact root

 @param declinedDir - directory of decline records

 @param publishDir - root of the mirrored tree

 @param cloneDir - corpus clone the pairs are read from, at each artifact's own commit

 @param l - logger the republish lines go through

 @example
 ```ts
 await republishRunPages({ runsDir, artifactsDir, declinedDir, publishDir, cloneDir, l, },);
 ```
 */
export async function republishRunPages(
  {
    runsDir,
    artifactsDir,
    declinedDir,
    publishDir,
    cloneDir,
    l,
  }: {
    readonly runsDir: string;
    readonly artifactsDir: string;
    readonly declinedDir: string;
    readonly publishDir: string;
    readonly cloneDir: string;
    readonly l: Logger;
  },
): Promise<void> {
  await removeDeclinedPages({
    declinedDir,
    publishDir,
  },);
  /**
   Entries this directory settled.
   */
  const settled = await settledEntryIds({ runsDir, },);
  if (settled.kind === 'unreadable') {
    l.warn(`republish: artifacts unreadable (${settled.reason}); no page judged`,);
    return;
  }
  await republishSettledPages({
    entryIds: settled.names,
    artifactsDir,
    publishDir,
    readPair: function readPair({
      entryId,
      corpusSha,
      generation,
    },): ReturnType<CorpusPairReader> {
      return readPinnedPair({
        cloneDir,
        entryId,
        corpusSha,
        generation,
        l,
      },);
    },
    l,
  },);
}

//endregion Pass republish
