import {
  readdir,
  readFile,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
} from 'node:path';

import { familyPackages, } from './family-source.test-fixture.ts';
import { linkTargets, } from './markdown-link-targets.test-fixture.ts';

//region Living repository-level docs
// The repository-level translation-repair docs that state current fact and are
// kept current (ledger D26 and D30): the decision records, the canonical
// handover and the snapshot it links, the planning docs that handover links as
// current, and the runbooks and troubleshooting docs.
// The archived documents (history segments, dated snapshots, run-continuity
// parts, interface-candidate files) are kept byte for byte and are not read;
// `doc/handover/translation-repair-document-map.md` says how to read their task
// numbers. `task-list-numbers.unit.test.ts` and `clock-time-zones.unit.test.ts`
// read through it.

/**
 Repository root, four levels above `src`.
 */
export const REPOSITORY_ROOT: string = join(
  import.meta.dirname,
  '..',
  '..',
  '..',
  '..',
);

/**
 Directory holding the family's packages, from the repository root. It also holds the family's own README and `doc`
 directory, which belong to no one package.
 */
export const FAMILY_PACKAGES: string = join(
  'package',
  'translation-repair',
);

/**
 Canonical handover, from the repository root; it links the current snapshot.
 */
const HANDOVER_INDEX = join(
  'doc',
  'handover',
  'translation-repair.md',
);

/**
 Planning docs the canonical handover or its snapshot links as current: where
 the readiness judgement lives, and the OpenRouter pass log that records every
 run since the takeover. Checked against the links on every read, so a doc
 dropped from them fails here rather than being read as current.
 */
const CURRENT_PLANNING: readonly string[] = [
  'translation-repair-readiness-signal.md',
  'translation-repair-openrouter-2026-09-03.md',
];

/**
 Name stems of the repository-level docs that belong to the family although their names do not start with
 `translation-repair`: a doc is the family's when its file name starts with one of these. A stem that is a whole doc
 name also takes the files that doc is split into. Every guard that decides which docs are the family's reads this
 list, so none can disagree with another (ledger D35).
 */
export const OTHER_STEMS: readonly string[] = [
  'accumulation-generation-restart',
  'an-archive-rebuilt-from-the-ledger-is-not-the-archive',
  'artifact-archive-text',
  'artifact-stores-',
  'barrel-star-export-drops-ambiguous-names',
  'cli-git-temporary-worktree-policy-inputs',
  'corpus-text-reached-a-public-repository',
  'coverage-wire-rerun-trade',
  'critic-attribution',
  'deleting-english-the-chinese-never-said',
  'duplicate-accepted-issues',
  'editor-and-refiner-seating',
  'every-volume-guard-is-blind-to-one-model',
  'flagged-slice-replacement-rate',
  'forbidden-strings-builtin-rule-fixture-tokens',
  'glm-4-7-flash-across-every-stage',
  'incumbent-almost-never-wins-a-contest',
  'introduced-defect-probe-false-negatives',
  'introduced-defect-probe-gating',
  'line-structure-loss-when-a-replacement-ships',
  'llm-assisted-block-pairing',
  'mise-detached-worktree-trust',
  'mise-inline-node-arguments',
  'naturalness-lane-reach',
  'node-lookup-in-agent-command-context',
  'nul-bytes-land-invisibly-in-source',
  'one-sided-pairing-census',
  'oxlint-type-aware-cgroup-memory',
  'pgrep-wait-loop-matches-itself',
  'reading-a-picture-at-the-user-boundary',
  'relocation-signature-at-flagged-slices',
  'slice-vocabulary-rename',
  'straggler-grace-remeasure',
  'stream-guards-first-production-traffic',
  'synthetic-aggregate-concurrency-stall',
  'synthetic-hf-prefix-misroute',
  'synthetic-rate-limit-default-drift',
  'tech-candidate-e-hyper-reserve-model-vet-2026-08-30',
  'tech-candidate-e-hyper-roster-expansion-vet-2026-08-30',
  'the-critics-are-shown-the-wrong-paragraph',
  'the-damage-no-instrument-was-catching',
  'the-judge-can-see-a-candidate-saying-less',
  'the-lane-contest',
  'the-third-rendering',
  'throw-assertions-name-the-message-not-the-error',
  'translation-pipeline-redesign',
  'two-lane-',
  'when-an-image-reading-makes-no-sense',
  'where-a-round-spends-its-wall-clock',
  'which-lane-ships',
  'wire-the-heading-aligner',
];

/**
 Directories under the repository's `doc` whose files are read for the family's docs.
 */
export const DOC_FAMILIES: readonly string[] = [
  'audit',
  'decision',
  'handover',
  'planning',
  'runbook',
  'troubleshooting',
];

/**
 One neighbour of a record's index, judged for whether it is a file of that record.
 */
type JudgedNeighbour = {
  /**
   Neighbour's file name.
   */
  readonly name: string;

  /**
   Whether its text names the record as the one it is part of.
   */
  readonly belongs: boolean;
};

/**
 The living repository-level docs, as paths from the repository root.
 */
type LivingRepositoryDocs = {
  /**
   Every `doc/decision/translation-repair*.md`.
   */
  readonly decisionRecords: readonly string[];

  /**
   The canonical handover and the snapshot it links.
   */
  readonly handover: readonly string[];

  /**
   The planning docs the handover links as current.
   */
  readonly currentPlanning: readonly string[];

  /**
   Every translation-repair runbook and troubleshooting doc, the operational
   references a pass is run and debugged by (ledger D30).
   */
  readonly operations: readonly string[];
};

/**
 The Markdown a package keeps: every `.md` directly under its `doc`, sorted, then its README.

 @param root - package directory, from the repository root

 @returns Paths from the repository root; a doc directory or README the package lacks adds none

 @example
 ```ts
 const docs = await markdownDocsIn({ root: 'package/translation-repair/archive', },);
 ```
 */
async function markdownDocsIn({ root, }: { readonly root: string; },): Promise<readonly string[]> {
  /**
   Names directly in the package directory.
   */
  const entries = await readdir(join(
    REPOSITORY_ROOT,
    root,
  ),);
  /**
   Markdown directly under its `doc`.
   */
  const docs = entries.includes('doc',)
    ? (await readdir(join(
      REPOSITORY_ROOT,
      root,
      'doc',
    ),))
      .filter(function isMarkdown(name,): boolean {
        return name.endsWith('.md',);
      },)
      .toSorted()
      .map(function underDoc(name,): string {
        return join(
          root,
          'doc',
          name,
        );
      },)
    : [];
  return [
    ...docs,
    ...(entries.includes('README.md',) ? [join(
      root,
      'README.md',
    ),] : []),
  ];
}

/**
 The package-level Markdown the guards read: the family's own README and docs, which are about the family as a whole,
 then each family package's README and docs.

 @returns Paths from the repository root, the family's own first

 @example
 ```ts
 const paths = await readPackageDocPaths();
 ```
 */
export async function readPackageDocPaths(): Promise<readonly string[]> {
  /**
   Roots whose Markdown is read: the family directory, then each package.
   */
  const roots = [
    FAMILY_PACKAGES,
    ...(await familyPackages()).map(function underFamily(name,): string {
      return join(
        FAMILY_PACKAGES,
        name,
      );
    },),
  ];
  return (await Promise.all(roots.map(function docsOf(root,): Promise<readonly string[]> {
    return markdownDocsIn({ root, },);
  },),)).flat();
}

/**
 Every translation-repair Markdown doc in one `doc` family, sorted so failures
 read in a stable order.

 @param family - directory under `doc`, such as `decision`

 @returns Paths from the repository root

 @example
 ```ts
 const records = await translationRepairDocs({ family: 'decision', },);
 ```
 */
async function translationRepairDocs({ family, }: { readonly family: string; },): Promise<readonly string[]> {
  return (await readdir(join(
    REPOSITORY_ROOT,
    'doc',
    family,
  ),))
    .filter(function isTranslationRepair(name,): boolean {
      return name.startsWith('translation-repair',) && name.endsWith('.md',);
    },)
    .toSorted()
    .map(function underFamily(name,): string {
      return join(
        'doc',
        family,
        name,
      );
    },);
}

/**
 Every Markdown doc of the repository's `doc` families whose file name starts with one of the stems, sorted within each
 family so failures read in a stable order.

 @param stems - name stems that mark a doc as the family's

 @returns Paths from the repository root

 @example
 ```ts
 const paths = await readStemDocs({ stems: ['the-cat-nap',], },);
 ```
 */
export async function readStemDocs({ stems, }: { readonly stems: readonly string[]; },): Promise<readonly string[]> {
  return (await Promise.all(DOC_FAMILIES.map(async function inFamily(family,): Promise<readonly string[]> {
    return (await readdir(join(
      REPOSITORY_ROOT,
      'doc',
      family,
    ),))
      .filter(function hasStem(name,): boolean {
        return name.endsWith('.md',) && stems.some(function opens(stem,): boolean {
          return name.startsWith(stem,);
        },);
      },)
      .toSorted()
      .map(function underFamily(name,): string {
        return join(
          'doc',
          family,
          name,
        );
      },);
  },),)).flat();
}

/**
 Every stem that no doc's file name starts with, each as one finding, so a doc renamed away from its stem cannot leave
 the stem reading nothing.

 @param stems - name stems that mark a doc as the family's

 @param paths - docs' paths from the repository root

 @returns Findings, in the order of the stems

 @example
 ```ts
 staleStemFindings({ stems: ['cat-nap',], paths: ['doc/planning/dog-walk.md',], },); // one finding
 ```
 */
export function staleStemFindings(
  {
    stems,
    paths,
  }: {
    readonly stems: readonly string[];
    readonly paths: readonly string[];
  },
): readonly string[] {
  return stems
    .filter(function matchesNone(stem,): boolean {
      return !paths.some(function opens(path,): boolean {
        /**
         Doc's file name, without its folder.
         */
        const name = basename(path,);
        return name.startsWith(stem,);
      },);
    },)
    .map(function finding(stem,): string {
      return `no doc's name starts with the stem ${stem}; drop it from OTHER_STEMS or rename the doc back`;
    },);
}

/**
 A record and the files it is kept in: a doc past the line limit keeps its
 path as an index and files its sections beside it as
 `<stem>-<what the file holds>.md` (ledger M165), so a guard reading the index
 alone would read little of its text. A file belongs to the record when its
 name opens with the record's stem and its own text says which record it is
 part of; a neighbour that only shares the stem, as a dated snapshot shares the
 canonical handover's, is not taken.

 @param path - record's path from the repository root

 @returns The record's index, then its files in name order

 @example
 ```ts
 const paths = await withFiles({ path: 'doc/planning/naps.md', },); // ['doc/planning/naps.md', 'doc/planning/naps-on-the-sofa.md']
 ```
 */
export async function withFiles({ path, }: { readonly path: string; },): Promise<readonly string[]> {
  /**
   Folder holding the record, from the repository root.
   */
  const folder = dirname(path,);
  /**
   Record's file name.
   */
  const indexName = basename(path,);
  /**
   What a file's name opens with.
   */
  const opening = `${basename(
    path,
    '.md',
  )}-`;
  /**
   Neighbours whose names open with the record's stem.
   */
  const candidates = (await readdir(join(
    REPOSITORY_ROOT,
    folder,
  ),))
    .filter(function sharesStem(name,): boolean {
      return name.startsWith(opening,) && name.endsWith('.md',);
    },)
    .toSorted();
  /**
   Each candidate with whether its text names this record as the one it is part of.
   */
  const judged = await Promise.all(candidates.map(async function judge(name,): Promise<JudgedNeighbour> {
    /**
     Candidate's lines.
     */
    const lines = (await readFile(
      join(
        REPOSITORY_ROOT,
        folder,
        name,
      ),
      'utf8',
    )).split('\n',);
    return {
      name,
      belongs: lines.some(function namesRecord(line,): boolean {
        return line.startsWith('Part of [',) && line.endsWith(`](${indexName}),`,);
      },),
    };
  },),);
  return [
    path,
    ...judged
      .filter(function belongsToRecord({ belongs, },): boolean {
        return belongs;
      },)
      .map(function inFolder({ name, },): string {
        return join(
          folder,
          name,
        );
      },),
  ];
}

/**
 The living repository-level docs, located from the canonical handover.

 @returns Paths from the repository root, by kind

 @throws {@link Error} when the canonical handover links no snapshot, or when a
 planning doc listed as current is linked from neither the handover nor its
 snapshot, since the guards would then read other than they claim

 @example
 ```ts
 const { decisionRecords, handover, currentPlanning, operations, } = await readLivingRepositoryDocs();
 ```
 */
export async function readLivingRepositoryDocs(): Promise<LivingRepositoryDocs> {
  /**
   Links in the canonical handover.
   */
  const indexLinks = linkTargets({ text: await readFile(
    join(
      REPOSITORY_ROOT,
      HANDOVER_INDEX,
    ),
    'utf8',
  ), },);
  /**
   The snapshot's file name, from the first link to one.
   */
  const snapshot = indexLinks.find(function isSnapshot(target,): boolean {
    return target.startsWith('translation-repair-handover-',) && target.endsWith('.md',);
  },);
  if (snapshot === undefined)
    throw new Error(`${HANDOVER_INDEX} links no handover snapshot`,);
  /**
   The snapshot, from the repository root.
   */
  const snapshotPath = join(
    'doc',
    'handover',
    snapshot,
  );
  /**
   The snapshot and the files it is kept in.
   */
  const snapshotFiles = await withFiles({ path: snapshotPath, },);
  /**
   Links in the snapshot and in each of its files.
   */
  const snapshotLinks = (await Promise.all(snapshotFiles.map(async function linksIn(path,): Promise<readonly string[]> {
    return linkTargets({ text: await readFile(
      join(
        REPOSITORY_ROOT,
        path,
      ),
      'utf8',
    ), },);
  },),)).flat();
  /**
   Every target the handover and its snapshot link.
   */
  const linked = new Set([
    ...indexLinks,
    ...snapshotLinks,
  ],);
  /**
   Planning docs listed as current that neither links.
   */
  const unlinked = CURRENT_PLANNING.filter(function isUnlinked(name,): boolean {
    return !linked.has(join(
      '..',
      'planning',
      name,
    ),);
  },);
  if (unlinked.length > 0)
    throw new Error(`neither ${HANDOVER_INDEX} nor ${snapshotPath} links ${unlinked.join(', ',)} any more`,);
  return {
    decisionRecords: await translationRepairDocs({ family: 'decision', },),
    handover: [
      HANDOVER_INDEX,
      ...snapshotFiles,
    ],
    operations: [
      ...await translationRepairDocs({ family: 'runbook', },),
      ...await translationRepairDocs({ family: 'troubleshooting', },),
    ],
    currentPlanning: (await Promise.all(CURRENT_PLANNING.map(function underPlanning(name,): Promise<readonly string[]> {
      return withFiles({ path: join(
        'doc',
        'planning',
        name,
      ), },);
    },),)).flat(),
  };
}

//endregion Living repository-level docs
