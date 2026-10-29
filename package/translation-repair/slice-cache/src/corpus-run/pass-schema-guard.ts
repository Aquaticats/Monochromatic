import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';


import {
  allInInputOrder,
  refusalText,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { artifactFileNameOf, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  ARTIFACT_SCHEMA_VERSION_V22,
  parseSettledTwoLaneArtifact,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import { UnplaceableArtifactError, } from './pass-generation-guard.ts';
import {
  censusBySchema,
  type SchemaCensusRow,
  type SchemaClassification,
} from './pass-schema-census.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Pass schema guard
// Refuses to resume an accumulation into a directory whose settled artifacts do
// not belong to the SCHEMA generation this pass writes.
//
// A sibling of the pipeline guard rather than more of it, because it answers a
// different question with a different remedy. That one asks which BUILD wrote
// the pool, and its opt-in exists because a mixed-build pool is still readable
// once a rate names a required commit. This one asks which SHAPE the files
// have, and no commit makes a version 1 artifact answer a two-lane question.
//
// The two overlap in practice and neither covers the other. A build that writes
// version 1 cannot share a digest with one that writes version 2, so the
// ordinary mixed-schema resume is already refused as pipeline drift. What
// reaches here is the case that opt-in lets through: an operator who accepted a
// mixed-BUILD pool, on a promise about required commits that schema drift does
// not keep.
//
// WHY IT MATTERS AT ALL, given the pass would still run: the scheduler counts
// every regular file named `*.json` as settled (`pass-settled.ts`), whatever
// its contents, so entries of the other generation are never re-run, while
// every reader that asks them a question this generation answers refuses
// them. The corpus ends up part one generation and part the other, and
// nothing in the run says so.
//
// THE LABEL IS NOT THE SHAPE, which an independent review of the first version
// of this guard pointed out and which it did not check. A version 1 body whose
// `artifactSchemaVersion` is edited to 2 satisfied every check here, was
// skipped by the scheduler, and failed only later in whatever reader asked it a
// two-lane question. So every artifact declaring the generation this pass
// writes is now PARSED with that generation's reader, at the one boundary where
// refusing is still free. The cost is one full parse per settled artifact at
// startup, against a pass that runs for days.
//
// NOT OVERRIDABLE, deliberately, and not by a variable of its own either. A
// second opt-in would recreate the hole this closes; the remedies belong in the
// message, and choosing between them costs re-running entries, which is money
// and therefore the operator's call rather than this guard's.

/**
 How many entries a refusal names per generation before it reports a count for
 the rest.
 */
const NAMED_EXAMPLES = 5;

/**
 What the census made of a file that is an artifact of some generation.
 */
type GenerationClassification = Exclude<SchemaClassification, { readonly kind: 'malformed'; }>;

/**
 A census row for a file that is an artifact of some generation.
 */
type GenerationRow = SchemaCensusRow & { readonly classification: GenerationClassification; };

/**
 Whether a census row is an artifact of some generation, rather than a file
 that is not an artifact at all.

 @param row - one settled entry's classification

 @returns Whether its classification names a generation

 @example
 ```ts
 const artifacts = rows.filter(isGenerationRow,);
 ```
 */
function isGenerationRow(row: SchemaCensusRow,): row is GenerationRow {
  /**
   What the census made of the row's file.
   */
  const { classification, } = row;
  return classification.kind !== 'malformed';
}

/**
 Phrase naming one generation, for a refusal.

 BUILT FROM THE CLASSIFICATION rather than used as its key, so a message can
 distinguish the generations it names. A file that is not an artifact at all
 gets no phrase: the guard refuses it apart, with the remedy that fits it
 (ledger B53).

 @param classification - what the census made of one artifact

 @returns Phrase a refusal groups by

 @example
 ```ts
 const label = generationLabel({ classification, },);
 ```
 */
function generationLabel(
  { classification, }: { readonly classification: GenerationClassification; },
): string {
  if (classification.kind === 'declared')
    return `schema version ${String(classification.version,)}`;

  if (classification.kind === 'unversioned')
    return 'no schema version at all';

  return 'a schema generation this build cannot read';
}

/**
 Names one generation and the entries settled under it.

 @param label - phrase naming the generation

 @param entryIds - entries settled under it

 @returns One line for the refusal

 @example
 ```ts
 const line = generationLine({ label: 'schema version 1', entryIds, },);
 ```
 */
function generationLine(
  {
    label,
    entryIds,
  }: {
    readonly label: string;
    readonly entryIds: readonly string[];
  },
): string {
  /**
   Entries named outright, capped so a corpus-sized directory still produces a
   readable refusal.
   */
  const named = entryIds.slice(
    0,
    NAMED_EXAMPLES,
  );

  /**
   Entries beyond the ones named.
   */
  const rest = entryIds.length - named.length;

  return `  ${label}: ${String(entryIds.length,)} settled, ${named.join(', ',)}${
    rest > 0 ? `, and ${String(rest,)} more` : ''
  }`;
}

/**
 Logger for the guard's own lines; the guard takes no caller-supplied logger.
 */
const gl = contextRoot({ tag: 'pass-schema-guard', },);

/**
 Ways forward every refusal here ends with, in the order an operator should
 consider them.

 ARCHIVING IS ONE OF THEM, which the first version of this message denied. It
 said deleting was not the remedy, full stop, and that is false: moving an
 incompatible artifact out of the directory is exactly what lets the scheduler
 re-run that entry, and moving it rather than deleting keeps the sound result
 it already is.
 */
const WAYS_FORWARD = [
  'Ways forward:',
  '',
  '  Start a fresh directory, with TRANSLATION_REPAIR_RUNS_DIR. The entries',
  '  already here keep their own generation and stay readable.',
  '',
  '  Restore the code those entries were settled under and resume there,',
  '  which matches the schema and the build at once.',
  '',
  '  Move the incompatible artifacts to an archive directory and resume here.',
  '  The scheduler counts filenames, so each one moved out is re-run and paid',
  '  for again, and the archived copy stays readable as the generation it is.',
  '',
  'Deleting them outright is the one thing to avoid: it costs the same re-run',
  'and destroys a sound result of the generation that wrote it.',
];

/**
 Raised when a resume would settle a second artifact generation into one pool.
 */
export class SchemaGenerationError extends StatedRefusalError {
  /**
   Declares this message safe to forward: it names a schema version.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Names every foreign generation, what this pass writes, and every way
   forward.

   @param foreign - entries per generation this pass does not write

   @param writes - generation this pass writes

   @example
   ```ts
   throw new SchemaGenerationError({ foreign, writes: 2, },);
   ```
   */
  constructor(
    {
      foreign,
      writes,
    }: {
      readonly foreign: ReadonlyMap<string, readonly string[]>;
      readonly writes: number;
    },
  ) {
    super({
      says: [
        'This artifacts directory holds artifacts of another schema generation.',
        '',
        ...[...foreign.entries(),].map(function toLine([
          label,
          entryIds,
        ]: ReadonlyEntry<string, readonly string[]>,): string {
          return generationLine({
            label,
            entryIds,
          },);
        },),
        `  this pass writes schema version ${String(writes,)}`,
        '',
        'The scheduler counts every .json NAME as settled, so those entries are',
        'never re-run, while every reader asking them a question this generation',
        'answers refuses them. Resuming here produces a corpus that is part one',
        'generation and part another, and nothing in the run reports it.',
        '',
        'The pipeline drift opt-in does not cover this and is not asked about.',
        'Drift is an opinion about which BUILD wrote a pool, and its remedy,',
        'naming a required commit alongside the rate, works because every file',
        'still answers the same questions. A file of another schema generation',
        'cannot answer them at all.',
        '',
        ...WAYS_FORWARD,
      ].join('\n',),
    },);
    this.name = 'SchemaGenerationError';
  }
}

/**
 Raised when an artifact declares the generation this pass writes and is not
 one.
 */
export class MislabelledArtifactError extends StatedRefusalError {
  /**
   Declares this message safe to print whole at a boundary: it names the entry
   and the generation and states the ways forward. The reader's own objection,
   which may quote the file it refused, goes to the run log at the throw site
   instead of riding here, which is what let the ways forward reach nobody
   while the class was unmarked.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Names the entry and the generation whose reader refused it.

   @param entryId - entry whose artifact carries the wrong label

   @param writes - generation it claims

   @example
   ```ts
   throw new MislabelledArtifactError({ entryId: 'Mittens', writes: 2, },);
   ```
   */
  constructor(
    {
      entryId,
      writes,
    }: {
      readonly entryId: string;
      readonly writes: number;
    },
  ) {
    super({
      says: [
        `${entryId} declares schema version ${
          String(writes,)
        }, which is what this pass writes, and is not one; what its reader refused is in the run log.`,
        '',
        'A label is not a shape. The scheduler counts this file as settled and',
        'never re-runs the entry, so a body that does not satisfy the generation',
        'it claims is discovered by whichever reader asks it a question first,',
        'long after the pass that could have refused it.',
        '',
        'This is not ordinary drift: no other generation writes this label, so',
        'the file was edited, truncated, or written by something that is not',
        'this pipeline.',
        '',
        ...WAYS_FORWARD,
      ].join('\n',),
    },);
    this.name = 'MislabelledArtifactError';
  }
}

/**
 Groups foreign census rows by the phrase a refusal names them with.

 @param rows - every settled entry's classification

 @param writes - generation this pass writes

 @returns Entries per foreign generation, in census order

 @example
 ```ts
 const foreign = foreignGroups({ rows, writes: 2, },);
 ```
 */
function foreignGroups(
  {
    rows,
    writes,
  }: {
    readonly rows: readonly GenerationRow[];
    readonly writes: number;
  },
): ReadonlyMap<string, readonly string[]> {
  return rows
    .filter(function isForeign({ classification, },): boolean {
      return (classification.kind !== 'declared') || (classification.version !== writes);
    },)
    .reduce(
    function group(
      groups: Map<string, readonly string[]>,
      {
        entryId,
        classification,
      },
    ): Map<string, readonly string[]> {
        /**
         Phrase this row is named under.
         */
      const label = generationLabel({ classification, },);

      return groups.set(
        label,
        [
          ...(groups.get(label,) ?? []),
          entryId,
        ],
      );
    },
    new Map<string, readonly string[]>(),
    );
}

/**
 Refuses an artifact that declares this generation and does not satisfy it.

 @param artifactsDir - directory holding the artifact

 @param entryId - entry to check

 @param writes - generation it declares

 @throws {@link MislabelledArtifactError} when this generation's reader
 refuses the body

 @example
 ```ts
 await assertBodyMatchesLabel({ artifactsDir, entryId: 'Mittens', writes: 2, },);
 ```
 */
async function assertBodyMatchesLabel(
  {
    artifactsDir,
    entryId,
    writes,
  }: {
    readonly artifactsDir: string;
    readonly entryId: string;
    readonly writes: number;
  },
): Promise<void> {
  /**
   Artifact text as it sits on disk, read again rather than carried out of the
   census: the census answers a question about every file and holding every
   body in memory to answer it would cost the whole directory at once.
   */
  const text = await readFile(
    join(
      artifactsDir,
      artifactFileNameOf({ entryId, },),
    ),
    'utf8',
  );

  try {
    parseSettledTwoLaneArtifact({ value: JSON.parse(text,), },);
  } catch (error) {
    // THE READER'S WORDS GO TO THE LOG, not into the refusal: the boundary
    // prints a marked message whole, and a parser's objection may quote the
    // file it refused.
    gl.warn(`${entryId}: its reader refused the body, ${refusalText({ error, },)}`,);
    throw new MislabelledArtifactError({
      entryId,
      writes,
    },);
  }
}

/**
 Refuses a resume that would mix artifact generations in one directory.

 Silent on a fresh directory and on a resume into one this pass wrote, which
 are the two ordinary cases. It reads the artifacts rather than a marker, so a
 directory assembled by hand is judged on what it holds.

 @param artifactsDir - directory holding one JSON per settled entry

 @param writes - generation this pass writes, defaulting to the one it writes
 today; a parameter so a caller can exercise this without rebuilding. THE
 DEFAULT MUST MOVE WITH THE WRITER: it sat at seven through generations
 eight, nine and ten, unnoticed because every launch of those days went into
 an empty directory, and on 2026-09-02 it refused XIEPT2 a directory holding
 keyword233's generation-ten artifact, the generation this same build writes.

 @throws {@link UnplaceableArtifactError} when a settled file is not an
 artifact at all (not JSON, or JSON that is not a record), whose remedy is
 removing it; the pass's placement half refuses such a file first, so only
 a caller running this guard alone meets it here (ledger B53)

 @throws {@link SchemaGenerationError} when any settled artifact belongs to
 another generation, naming every one of them

 @throws {@link MislabelledArtifactError} when an artifact declares this
 generation and this generation's reader refuses it

 @example
 ```ts
 await assertResumableSchemaGeneration({ artifactsDir, },);
 ```
 */
export async function assertResumableSchemaGeneration(
  {
    artifactsDir,
    writes = ARTIFACT_SCHEMA_VERSION_V22,
  }: {
    readonly artifactsDir: string;
    readonly writes?: number;
  },
): Promise<void> {
  /**
   Every settled entry, classified.
   */
  const rows = await censusBySchema({ artifactsDir, },);

  // A FILE THAT IS NOT AN ARTIFACT AT ALL IS REFUSED APART (ledger B53), with
  // the remedy that fits it: it is a file to investigate and remove, not a
  // sound result of another generation to archive, which is what naming it
  // among the foreign generations told an operator.
  /**
   Entries whose file is not an artifact at all.
   */
  const malformedIds = rows
    .filter(function isMalformed(row,): boolean {
      return !isGenerationRow(row,);
    },)
    .map(function idOf({ entryId, },): string {
      return entryId;
    },);
  if (malformedIds.length > 0)
    throw new UnplaceableArtifactError({ entryIds: malformedIds, });

  /**
   Every settled entry, each an artifact of some generation.
   */
  const artifacts = rows.filter(isGenerationRow,);

  /**
   Entries belonging to any other generation.
   */
  const foreign = foreignGroups({
    rows: artifacts,
    writes,
  },);

  if (foreign.size > 0) {
    throw new SchemaGenerationError({
      foreign,
      writes,
    },);
  }

  // THE LABEL CHECK PASSED, so every remaining file claims this generation.
  // Now they have to BE it, which only this generation's reader can say.
  await allInInputOrder({
    members: artifacts.map(async function checkBody({ entryId, },): Promise<void> {
      await assertBodyMatchesLabel({
        artifactsDir,
        entryId,
        writes,
      },);
    },),
  },);
}

//endregion Pass schema guard
