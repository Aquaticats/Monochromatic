import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import { access, } from 'node:fs/promises';
import { join, } from 'node:path';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type PairingRecipe,
  readSettledArtifact,
  readSettledEntryInfo,
  rebuildPreparation,
  type RebuildReproduction,
  recipeOf,
  type ParsedTwoLaneArtifact
} from '@monochromatic-dev/translation-repair-artifact/ts';
import type {
  CorpusPin,
  PreparedDocumentPair,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { rethrowUnlessMissingPath, } from '@monochromatic-dev/translation-repair-typography/ts';
import { readRunJson, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type ArtifactFileName,
  artifactFileNameOf,
  artifactsDirOf,
  entryIdOfArtifact,
  listArtifactFiles,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';

//region Settled carve
// Carving a corpus entry the way the pass carved it when it settled.
//
// THE ONE ENTRY POINT FOR INSTRUMENTS. Every probe and census that measures
// slices used to call the bare `prepareDocumentPair` over the corpus, under a
// comment claiming "slices exactly as the lanes would see them". The lanes see
// slices the roster shell carved, and the only durable record of that carve is
// the settled artifact, so an instrument wanting the lanes' slices reads the
// artifact's recipe and rebuilds through it. An entry with no settled artifact
// has no such slicing to measure, and says so rather than substituting the
// deterministic carve under the same name.
//
// THE PIN COMES FROM THE ARTIFACT, as the rendering audit's does: the run pin
// can move, and a recipe recorded against one commit describes that commit's
// pair of documents, not whatever the pin now names.
//
// THE CARVE IS `rebuildPreparation`'S, not a second one here (ledger F-6, A18).
// A carve of its own called `prepareDocumentPair` without the front-matter
// authority and the seal the artifact's generation carved with, shifting slice
// indices by one on archive-authority entries, and over the corpus copy rather
// than the archive the pass reshaped and stored.

/**
 What the settled artifacts directory says about one entry's recipe.

 @example
 ```ts
 const recipe: SettledRecipe = { kind: 'unsettled', };
 ```
 */
export type SettledRecipe = {
  /**
   A two-lane artifact records this entry, and here is its recipe.
   */
  readonly kind: 'settled';

  /**
   Corpus commit the artifact was settled against.
   */
  readonly corpusSha: string;

  /**
   Pairing recipe the artifact records, with the halves it lacks named.
   */
  readonly recipe: PairingRecipe;

  /**
   The artifact itself, which also records the flags and the archive the
   run carved (ledger F-6, A18).
   */
  readonly artifact: ParsedTwoLaneArtifact;
} | {
  /**
   An artifact exists but predates the two-lane shape, so it records no
   preparation and no recipe.
   */
  readonly kind: 'legacy';
} | {
  /**
   No artifact records this entry.
   */
  readonly kind: 'unsettled';
};

/**
 One entry carved through its settled recipe, or why it could not be.

 @example
 ```ts
 const carve: SettledCarve = await carveSettled({ entryId, runsDir, cloneDir, readFile: readCorpusFile, },);
 ```
 */
export type SettledCarve = {
  /**
   Carved through the artifact's recipe over the pair at its own commit.
   */
  readonly kind: 'settled';

  /**
   Corpus commit the pair was read at.
   */
  readonly corpusSha: string;

  /**
   Whole original at that commit.
   */
  readonly sourceText: string;

  /**
   Archive text the carve ran over: the one the artifact stored, which the
   pass reshaped before carving, else the corpus copy at that commit.
   */
  readonly targetText: string;

  /**
   Slicing the lanes saw, or the closest the recorded recipe reaches.
   */
  readonly prepared: PreparedDocumentPair;

  /**
   Recipe that produced it, with any defaulted halves named.
   */
  readonly recipe: PairingRecipe;

  /**
   Whether the carve is the run's own, read off the rows the run recorded.
   */
  readonly reproduction: RebuildReproduction;
} | {
  /**
   An artifact exists but records no recipe.
   */
  readonly kind: 'legacy';
} | {
  /**
   No artifact records this entry.
   */
  readonly kind: 'unsettled';
};

/**
 Lists the entries a runs directory holds settled artifacts for.

 @param runsDir - runs directory whose `artifacts/` subdirectory is read

 @returns Entry ids in sorted order, empty when the directory has none

 @example
 ```ts
 const entryIds = await listSettledEntryIds({ runsDir, },);
 ```
 */
export async function listSettledEntryIds(
  { runsDir, }: { readonly runsDir: string; },
): Promise<readonly string[]> {
  /**
   Where a pass writes its artifacts.
   */
  const artifactsDir = artifactsDirOf({ runsDir, },);
  /**
   Artifact files the directory holds, regular files only, as the census and
   the scheduler list them (ledger B64).
   */
  const names = await (async function listed(): Promise<readonly ArtifactFileName[]> {
    try {
      return await listArtifactFiles({ artifactsDir, },);
    }
    catch (error) {
      // A runs directory nothing has settled into yet has no artifacts
      // subdirectory, which is an ordinary state rather than a fault.
      rethrowUnlessMissingPath({ error, },);
      return [];
    }
  })();
  return textsInCodePointOrder({ texts: names
    .map(function toEntryId(name,): string {
      return entryIdOfArtifact({ name, },);
    },), },);
}

/**
 Reads the recipe one entry's settled artifact records, without touching
 the corpus.

 @param entryId - corpus entry

 @param runsDir - runs directory holding `artifacts/<entryId>.json`

 @returns Recipe and commit, or the reason there is none

 @example
 ```ts
 const settled = await readSettledRecipe({ entryId, runsDir, },);
 ```
 */
export async function readSettledRecipe(
  {
    entryId,
    runsDir,
  }: {
    readonly entryId: string;
    readonly runsDir: string;
  },
): Promise<SettledRecipe> {
  /**
   Where this entry's artifact would sit.
   */
  const path = join(
    artifactsDirOf({ runsDir, },),
    artifactFileNameOf({ entryId, },),
  );
  try {
    await access(path,);
  }
  catch (error) {
    rethrowUnlessMissingPath({ error, },);
    return { kind: 'unsettled', };
  }

  /**
   Artifact as written, dispatched by generation.
   */
  const reading = readSettledArtifact({ value: await readRunJson({ path, },), },);
  if (reading.kind !== 'version-2')
    return { kind: 'legacy', };

  /**
   Two-lane artifact, which is the generation that records a preparation.
   */
  const { artifact, } = reading;
  return {
    kind: 'settled',
    corpusSha: artifact.corpusSha,
    recipe: recipeOf({ artifact, },),
    artifact,
  };
}

/**
 Carves one entry through its settled recipe over the pair at the
 artifact's own commit.

 @param entryId - corpus entry

 @param runsDir - runs directory holding the artifact

 @param cloneDir - corpus clone the artifact's commit is read from

 @param readFile - reads a corpus file at a pin: `readCorpusFile` in a run, a
 test's scripted reader otherwise, so the order the two pages answer in is a
 thing a case sets

 @returns Slicing the lanes saw, or the reason there is none

 @example
 ```ts
 const carve = await carveSettled({ entryId, runsDir, cloneDir, readFile: readCorpusFile, },);
 ```
 */
export async function carveSettled(
  {
    entryId,
    runsDir,
    cloneDir,
    readFile,
  }: {
    readonly entryId: string;
    readonly runsDir: string;
    readonly cloneDir: string;
    readonly readFile: typeof readCorpusFile;
  },
): Promise<SettledCarve> {
  /**
   Recipe the artifact records, if any.
   */
  const settled = await readSettledRecipe({
    entryId,
    runsDir,
  },);
  if (settled.kind !== 'settled')
    return settled;

  /**
   Pin taken from the artifact, so the pair read is the pair it describes.
   */
  const pin: CorpusPin = {
    cloneDir,
    commitSha: settled.corpusSha,
  };

  /**
   Both sides at that commit. When both reads are refused the refusal is the
   original's, as the input lists it first, whichever read ends first.
   */
  const [
    sourceText,
    targetText,
    entryInfo,
  ] = await allInInputOrder({
    members: [
      readFile({
        pin,
        relPath: `people/${entryId}/page.md`,
      },),
      readFile({
        pin,
        relPath: `people/${entryId}/page.en.md`,
      },),
      // THE DEATH THE RUN'S MODELS WERE TOLD, read at the artifact's own commit
      // like the pages, listed last so a refused page read is still the
      // refusal an input reports.
      readSettledEntryInfo({
        generation: settled.artifact
          .artifactSchemaVersion,
        entryId,
        pin,
        readFile,
      },),
    ],
  },);

  /**
   The carve the artifact's recipe, flags and stored archive give.
   */
  const {
    prepared,
    reproduction,
  } = rebuildPreparation({
    artifact: settled.artifact,
    sourceText,
    targetText,
    ...((entryInfo.kind === 'read') ? { sourceEntryInfo: entryInfo.info, } : {}),
  },);
  return {
    kind: 'settled',
    corpusSha: settled.corpusSha,
    sourceText,
    targetText: prepared.targetText,
    prepared,
    recipe: settled.recipe,
    reproduction,
  };
}

/**
 Names a recipe's completeness for a log line.

 @param recipe - recipe as read

 @returns `complete recipe`, or the halves the deterministic default stood in for

 @example
 ```ts
 log.info(`${entryId}: carved from its settled artifact (${recipeLabel({ recipe, },)})`,);
 ```
 */
export function recipeLabel(
  { recipe, }: { readonly recipe: PairingRecipe; },
): string {
  /**
   Halves the file did not record.
   */
  const { unrecorded, } = recipe;
  if (unrecorded.length === 0)
    return 'complete recipe';
  return `deterministic default for ${unrecorded.join(', ',)}`;
}

//endregion Settled carve
