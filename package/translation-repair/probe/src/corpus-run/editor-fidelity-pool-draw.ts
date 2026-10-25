import { tagged, } from '@monochromatic-dev/module-logger/ts';
import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { namesOfKind, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  type CorpusPin,
  CorpusReadError,
  listCorpusPeople,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  type CommandLineOf,
  readRunCorpusPinSetting,
  reportingRefusals,
} from '@monochromatic-dev/translation-repair-run/ts';
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { REVIEWED_FIDELITY_REFERENCES, } from '../fidelity-reference-manifest.ts';
import { readReviewedFidelityReferences, } from '../fidelity-reference-read.ts';
import { EDITOR_FIDELITY_POOL, } from '../editor-fidelity-pool-manifest.ts';
import type { PoolFile, } from '../editor-fidelity-pool-source.ts';
import { runEditorFidelityPoolDraw, } from './editor-fidelity-pool-draw-run.ts';
import type { EntryPages, } from './editor-fidelity-pool-selection.ts';

//region Editor fidelity pool draw main
// Wiring only: the command line, the corpus pin, the clone's listing and page readers, the directory reader and writer, the
// committed pool and the reference reader go to `editor-fidelity-pool-draw-run.ts`, which holds the whole run. The run
// reads the pinned clone and writes nothing outside `--out`; it calls no provider.

/**
 Whether a file name in the directory is one the draw writes: a part, or the file that gathers the parts.

 @param name - file name

 @returns Whether the draw writes a file by that name

 @example
 ```ts
 const drawn = isDrawnFile({ name: 'editor-fidelity-pool-drawn-1.ts', },);
 ```
 */
function isDrawnFile({ name, }: { readonly name: string; },): boolean {
  return name.startsWith('editor-fidelity-pool-drawn',) && name.endsWith('.ts',);
}

/**
 Reads the files of the draw that a directory holds.

 @param dir - directory read

 @returns Each file by name with its text

 @example
 ```ts
 const files = await readDrawnFiles({ dir: 'src', },);
 ```
 */
async function readDrawnFiles({ dir, }: { readonly dir: string; },): Promise<readonly PoolFile[]> {
  /**
   Drawn files present in the directory.
   */
  const names = (await namesOfKind({
    dir,
    kind: 'file',
  },))
    .filter(function isDrawn(name,): boolean {
      return isDrawnFile({ name, },);
    },);
  return allInInputOrder({
    members: names.map(async function read(name,): Promise<PoolFile> {
      return {
        name,
        text: await readFile(
          join(
            dir,
            name,
          ),
          'utf8',
        ),
      };
    },),
  },);
}

/**
 Writes the files of the draw into a directory.

 @param dir - directory written

 @param files - files written

 @example
 ```ts
 await writeDrawnFiles({ dir: 'src', files, },);
 ```
 */
async function writeDrawnFiles(
  {
    dir,
    files,
  }: {
    readonly dir: string;
    readonly files: readonly PoolFile[];
  },
): Promise<void> {
  await allInInputOrder({
    members: files.map(function write(file,): Promise<void> {
      return writeFile(
        join(
          dir,
          file.name,
        ),
        file.text,
      );
    },),
  },);
}

/**
 Reads an entry's two pages, or says the entry has none.

 @param pin - pin the pages are read at

 @param entryId - entry read

 @returns The two pages, or that the entry cannot be read

 @example
 ```ts
 const pages = await readEntryPages({ pin, entryId: 'MTF_0615', },);
 ```
 */
async function readEntryPages(
  {
    pin,
    entryId,
  }: {
    readonly pin: CorpusPin;
    readonly entryId: string;
  },
): Promise<EntryPages> {
  try {
    /**
     The original page and the archive page.
     */
    const [zh, en,] = await allInInputOrder({
      members: [
        readCorpusFile({
          pin,
          relPath: `people/${entryId}/page.md`,
        },),
        readCorpusFile({
          pin,
          relPath: `people/${entryId}/page.en.md`,
        },),
      ],
    },);
    return {
      zh,
      en,
    };
  }
  catch (error) {
    // Only a page that is not in the commit is the expected missing side of an incomplete pair; a clone that cannot be
    // read at all must stop the draw, not drop every entry in silence.
    if ((!(error instanceof CorpusReadError)) || (error.kind !== 'missing-object'))
      throw error;
    tagged({ tag: 'editor-fidelity-pool-draw', },)
      .info(`entry ${entryId} has no pair of pages in the commit`,);
    return { unreadable: true, };
  }
}

/**
 Hands the run what only a real process has.

 @param line - the command line, read whole by `reportingRefusals`

 @example
 ```ts
 await main({ line, },);
 ```
 */
function main({ line, }: { readonly line: CommandLineOf<'editor-fidelity-pool-draw'>; },): Promise<void> {
  /**
   Corpus clone and commit this run reads, after any environment override.
   */
  const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
  return runEditorFidelityPoolDraw({
    line,
    pin,
    listEntries: function listEntries(): Promise<readonly string[]> {
      return listCorpusPeople({ pin, },);
    },
    readEntry: function readEntry({ entryId, },): Promise<EntryPages> {
      return readEntryPages({
        pin,
        entryId,
      },);
    },
    readFiles: readDrawnFiles,
    writeFiles: writeDrawnFiles,
    committed: EDITOR_FIDELITY_POOL,
    reviewedEntryIds: REVIEWED_FIDELITY_REFERENCES.map(function entryOf(spec,): string {
      return spec.entryId;
    },),
    reviewedTrials: REVIEWED_FIDELITY_REFERENCES.flatMap(function damagesOf(spec,) {
      return spec.damages;
    },)
      .filter(function isEditorKind(damage,): boolean {
        return (damage.kind === 'insertion') || (damage.kind === 'alteration');
      },)
      .length,
    readReferences: readReviewedFidelityReferences,
    log: tagged({ tag: 'editor-fidelity-pool-draw', },),
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'editor-fidelity-pool-draw',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Editor fidelity pool draw main
