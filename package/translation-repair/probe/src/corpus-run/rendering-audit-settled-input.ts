import {
  allInInputOrder,
  refusalText,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  basename,
  join,
} from 'node:path';


import type {
  CorpusPin,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { readRunJson, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  namesOfKind,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import type { listArtifactFiles, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  type ParsedTwoLaneArtifact,
  parseSettledTwoLaneArtifact,
  readSettledEntryInfo,
  rebuildPreparation,
  type RebuiltPreparation,
  type RecipeHalf,
  verifyArtifactMeasurements,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  identityOf,
  type SettledAuditSubject,
  subjectsOf,
} from './rendering-audit-settled-subject.ts';

//region Settled audit input
// Turns an archive of settled version 2 artifacts into audit subjects, without
// asking a single model anything.
//
// WHY THIS IS ITS OWN MODULE. Everything here is free, and everything in the
// driver costs quota. Splitting them means the whole population, the
// retained-versus-replaced split and the provenance check can be read, tested
// and re-read at no cost, and a mistake in them is found before any roster is
// woken up.
//
// WHERE THE TEXT COMES FROM, which is the decision that shaped this module. The
// artifact carries `sourceText` on every delivery row and `acceptedText` on
// every decided outcome, so the audit reads the two texts STRAIGHT OUT OF THE
// ARTIFACT and never re-slices to obtain them. That is not a shortcut:
// auditing re-sliced text would audit a different input than the one the judges
// actually saw, and the question is what the audit says about the decisions
// that were really made.
//
// SO WHY RE-PREPARE AT ALL. Two things live only in the preparation:
//
// -   `identityContext`, the declared names and handles from both sides' front
//     matter. The producing judges had it, and both archived entries carry a
//     real one: `玖月折耳猫` is declared as `Zheermao September`, `柠檬酸` as
//     `Citric Acid`. An auditor shown the rendering WITHOUT that block sees a
//     name it cannot derive from the source and has every reason to call it a
//     fabrication. That defect was found once already, and rediscovering
//     it as "defects" would poison every name-bearing slice.
// -   Provenance. The rebuild's rows are checked against the rows the run
//     recorded, and the artifact's measurements against the rebuild, since the
//     artifact stores measurements of the pair rather than the pair. NOT the
//     recorded identity (ledger A12b): it also hashes the declared names as the
//     run's build worded them, so no rebuild today names itself as the run did,
//     and every settled artifact read REFUSED.
//
// THE CITED REFERENCES ARE THE THIRD INPUT, and they are NOT read here
// (ledger B29). The producing critics and panels were shown what the pages
// the original links say, which is not in the preparation at all: it is an
// outside read, a web fetch on a cache miss. So this free module carries the
// whole original on every subject, and the driver reads its links only for
// the subjects it buys.
//
// RE-PREPARED WITH THE RECIPE THE ARTIFACT RECORDS, through
// `rebuildPreparation`, never with the bare deterministic carve. The pass
// carves through the roster shell, whose section and block rounds move slices,
// so a bare re-preparation disagreed with every roster-paired artifact and the
// provenance verdict read `refused` on exactly the artifacts it existed to
// verify. An artifact that records only part of the recipe is rebuilt with the
// deterministic default for the rest, and the gap is named.
//
// A REFUSAL IS A FINDING, NOT A STOP. Verification answering no would say the
// slicing moved under a settled artifact, which is worth recording loudly; it
// does not make that artifact's rows unreadable, because they are what the run
// actually judged. So it is carried as a value.
//
// A MISMATCH BESIDE A NAMED GAP IS NOT A REFUSAL. When the artifact does not
// record a recipe half, the rebuild guessed it, and a disagreement may be the
// guess rather than a moved slicing. That reads `unverifiable`, which is the
// honest verdict for every artifact settled before the recipe was recorded.
// A MATCH is evidence either way: the identity hashes every slice's placement
// and text, so a rebuild that reproduces it reproduced the slicing.
//
// THE PIN COMES FROM THE ARTIFACT, never from the run's own. The run pin can
// move; a file naming its own corpus commit answers for itself forever. The
// read goes through the commit object rather than the working tree, so a dirty
// clone cannot change the answer either.

/**
 Whether the recorded preparation identity still describes the pair.

 @example
 ```ts
 const verification: SettledVerification = { kind: 'verified', };
 ```
 */
export type SettledVerification = {
  /**
   Preparation recomputed from the corpus matched what the artifact recorded.
   */
  readonly kind: 'verified';
} | {
  /**
   It did not, which says the slicing moved under a settled artifact.
   */
  readonly kind: 'refused';

  /**
   What the check objected to, kept as text since nothing reads it by field.
   */
  readonly detail: string;
} | {
  /**
   It did not, and the artifact records only part of the recipe the rebuild
   needed, so the disagreement may be the defaulted half rather than a
   moved slicing.
   */
  readonly kind: 'unverifiable';

  /**
   Recipe halves the artifact does not record.
   */
  readonly unrecorded: readonly RecipeHalf[];

  /**
   What the check objected to, for a reader deciding whether the gap
   explains it.
   */
  readonly detail: string;
};

/**
 One artifact, read.

 @example
 ```ts
 const reading: SettledArtifactReading = { runSet, entryId, verification, subjects, };
 ```
 */
export type SettledArtifactReading = {
  /**
   Archive subdirectory it was found in.
   */
  readonly runSet: string;

  /**
   File name within that subdirectory.
   */
  readonly artifactFile: string;

  /**
   Corpus entry.
   */
  readonly entryId: string;

  /**
   Built output that produced it.
   */
  readonly artifactDigest: string;

  /**
   Provenance answer, carried rather than thrown.
   */
  readonly verification: SettledVerification;

  /**
   Every slice this artifact offers the audit.
   */
  readonly subjects: readonly SettledAuditSubject[];
};

/**
 Where one artifact sits inside the archive.

 @example
 ```ts
 const at: ArtifactLocation = { runSet: 'two-lane-cost-2026-08-16', artifactFile: 'Aniloviraw.json', };
 ```
 */
type ArtifactLocation = {
  /**
   Subdirectory to look in, EMPTY when the artifact sits at the archive root.

   SEPARATE FROM `runSet`, which it used to be the same field as, and the two
   came apart the moment a flat layout was accepted: the path segment must be
   empty for an artifact at the root, while the label must still name
   something a reader can trace. One field cannot be both, and a test caught
   it doing neither.
   */
  readonly runSetDir: string;

  /**
   What every row calls this settlement.

   For a nested archive this is the subdirectory. For a flat one it is the
   archive's own name, because the directory IS the settlement there.
   */
  readonly runSet: string;

  /**
   File name within it.
   */
  readonly artifactFile: string;
};

/**
 Runs the provenance check and turns its refusal into a value.

 @param artifact - parsed artifact

 @param rebuilt - preparation rebuilt from the corpus with the artifact's
 recorded recipe, beside the recipe halves it lacked

 @returns Answer, never a throw

 @example
 ```ts
 const verification = verifySettled({ artifact, rebuilt, },);
 ```
 */
function verifySettled(
  {
    artifact,
    rebuilt,
  }: {
    readonly artifact: ParsedTwoLaneArtifact;
    readonly rebuilt: RebuiltPreparation;
  },
): SettledVerification {
  /**
   Whether the rebuild's rows are the run's carve, and the recipe halves it
   had to default.
   */
  const {
    reproduction,
    unrecorded,
  } = rebuilt;
  if (reproduction.kind === 'moved') {
    return (unrecorded.length === 0)
      ? {
        kind: 'refused',
        detail: reproduction.detail,
      }
      : {
        kind: 'unverifiable',
        unrecorded,
        detail: reproduction.detail,
      };
  }
  try {
    verifyArtifactMeasurements({
      artifact,
      prepared: rebuilt.prepared,
    },);
    return { kind: 'verified', };
  }
  catch (error) {
    // The rows reproduced, so a defaulted recipe half cannot explain a
    // measurement the rebuild does not have: it is a refusal either way.
    return {
      kind: 'refused',
      detail: refusalText({ error, },),
    };
  }
}

/**
 Reads one settled artifact into audit subjects.

 @param archiveDir - directory holding run-set subdirectories

 @param runSet - subdirectory this artifact lives in

 @param artifactFile - file name within it

 @param cloneDir - corpus clone the artifact's own commit is read from

 @param readFile - reads one page at a commit, passed in so a case scripts which of the two page reads ends first
 instead of the clone deciding it

 @returns Everything that artifact offers, provenance included

 @throws {@link ArtifactParseError} when the file is not a settled version 2
 artifact, which is a defect in what was archived rather than a finding

 @example
 ```ts
 const reading = await readArtifactSubjects({ archiveDir, runSetDir, runSet, artifactFile, cloneDir, readFile: readCorpusFile, },);
 ```
 */
export async function readArtifactSubjects(
  {
    archiveDir,
    runSetDir,
    runSet,
    artifactFile,
    cloneDir,
    readFile,
  }: {
    readonly archiveDir: string;
    readonly runSetDir: string;
    readonly runSet: string;
    readonly artifactFile: string;
    readonly cloneDir: string;
    readonly readFile: typeof readCorpusFile;
  },
): Promise<SettledArtifactReading> {
  /**
   Artifact as written, untyped until parsed.
   */
  const raw: unknown = await readRunJson({
    path: join(
      archiveDir,
      runSetDir,
      artifactFile,
    ),
  },);

  /**
   Artifact checked against the version 2 read contract.
   */
  const artifact = parseSettledTwoLaneArtifact({ value: raw, },);

  /**
   Corpus pin taken from the artifact, so the file answers for itself even
   after the run pin moves.
   */
  const pin: CorpusPin = {
    cloneDir,
    commitSha: artifact.corpusSha,
  };

  /**
   Pair as it stood at that commit.
   */
  const [
    sourceText,
    targetText,
    entryInfo,
  ] = await allInInputOrder({
    members: [
      readFile({
        pin,
        relPath: `people/${artifact.id}/page.md`,
      },),
      readFile({
        pin,
        relPath: `people/${artifact.id}/page.en.md`,
      },),
      // THE DEATH THE RUN'S MODELS WERE TOLD, read at the artifact's own commit
      // like the pages, so the identity block the auditors are shown is the one
      // the run's models saw; listed last so a refused page read is still the
      // refusal an input reports.
      readSettledEntryInfo({
        generation: artifact.artifactSchemaVersion,
        entryId: artifact.id,
        pin,
        readFile,
      },),
    ],
  },);

  /**
   Preparation rebuilt from that pair with the recipe the artifact records,
   wanted for its identity block and for the provenance check, NOT for slice
   text.
   */
  const rebuilt = rebuildPreparation({
    artifact,
    sourceText,
    targetText,
    ...((entryInfo.kind === 'read') ? { sourceEntryInfo: entryInfo.info, } : {}),
  },);

  return {
    runSet,
    artifactFile,
    entryId: artifact.id,
    artifactDigest: artifact.pipelineDigest,
    verification: verifySettled({
      artifact,
      rebuilt,
    },),
    subjects: subjectsOf({
      artifact,
      runSet,
      identity: identityOf({ prepared: rebuilt.prepared, },),
      pageSourceText: sourceText,
    },),
  };
}

/**
 Lists every artifact under an archive directory, in a stable order.

 TAKES TWO LAYOUTS, because two exist and only one was accepted before.

 NESTED, `archive/<run set>/<entry>.json`, which is what the hand-built
 archive at `~/translation-repair-v2-archive/` carries: each subdirectory is
 one settlement of the corpus and its name is the run set.

 FLAT, `<runs dir>/artifacts/<entry>.json`, which is what `corpus-pass`
 ACTUALLY WRITES. A pass produces exactly one settlement, so it has no reason
 to invent a subdirectory for it, and the four archived artifacts read today
 only because somebody copied them into run-set directories by hand. Pointing
 this reader at a pass's own output found nothing and the audit refused with
 "no artifacts under", which reads like an empty pass rather than like a
 layout it cannot see.

 REFUSES A DIRECTORY CARRYING BOTH rather than choosing. A directory with
 artifacts at its root AND in subdirectories is either two populations or a
 half-finished move, and reading one while ignoring the other would report a
 population smaller than the archive holds without saying so. That is the
 defect this whole instrument keeps finding in other places.

 @param archiveDir - directory of run sets, or of artifacts

 @param listFiles - lists the artifacts one directory holds, passed in so a case scripts which of two run sets'
 listings ends first

 @returns Locations sorted by run set then file

 @throws {@link StatedRefusalError} when artifacts sit at both levels

 @example
 ```ts
 const located = await locateSettledArtifacts({ archiveDir, listFiles: listArtifactFiles, },);
 ```
 */
async function locateSettledArtifacts(
  {
    archiveDir,
    listFiles,
  }: {
    readonly archiveDir: string;
    readonly listFiles: typeof listArtifactFiles;
  },
): Promise<readonly ArtifactLocation[]> {
  /**
   Run-set subdirectories, sorted.
   */
  const runSets = textsInCodePointOrder({ texts: (await namesOfKind({
    dir: archiveDir,
    kind: 'directory',
  },)), },);

  /**
   Artifacts sitting at the archive root, which is the layout a pass writes:
   regular files named like one, as every artifact reader lists them.
   */
  const loose = textsInCodePointOrder({ texts: (await listFiles({ artifactsDir: archiveDir, },)), },);

  // STATED, NOT FAULTED: the archive is the operator's, the path is what they
  // typed, and the remedy is theirs, so `reportingRefusals` prints this line
  // and exits 6 rather than printing frames for a bug that is not one.
  if ((loose.length > 0) && (runSets.length > 0))
    throw new StatedRefusalError({
      says: `${archiveDir} carries artifacts at its root AND in subdirectories`
        + ` (${String(loose.length,)} loose, ${String(runSets.length,)} run sets).`
        + ` Reading one and ignoring the other would report a smaller population than`
        + ` the archive holds. Move them together, then re-run.`,
    },);

  if (loose.length > 0)
    return loose.map(function atRoot(artifactFile,): ArtifactLocation {
      return {
        runSetDir: '',
        // The archive is itself the one settlement, so it names the run set.
        // Its own basename rather than a placeholder, because that name lands
        // in every row and a reader tracing a row back wants the directory.
        runSet: basename(archiveDir,),
        artifactFile,
      };
    },);

  return (await allInInputOrder({
    members: runSets.map(
      async function within(runSet,): Promise<readonly ArtifactLocation[]> {
        // Regular files only, as the loose layout already required: a directory
        // named like an artifact inside a run set was read as JSON and stopped
        // the whole archive reading (ledger B64).
        return textsInCodePointOrder({ texts: (await listFiles({
          artifactsDir: join(
            archiveDir,
            runSet,
          ),
        },)), },)
          .map(function at(artifactFile,): ArtifactLocation {
            return {
              runSetDir: runSet,
              runSet,
              artifactFile,
            };
          },);
      },
    ),
  },))
    .flat();
}

/**
 Reads every settled artifact under an archive directory.

 ORDER IS DETERMINISTIC, sorted by run set then file, so two invocations
 produce rows in the same order and a capped run always buys the same prefix.

 @param archiveDir - directory whose subdirectories are run sets

 @param cloneDir - corpus clone every artifact's own commit is read from

 @param readFile - reads one page at a commit, passed in so a case scripts which artifact's page read is
 refused first

 @param listFiles - lists the artifacts one directory holds, passed in so a case scripts which run set's
 listing is refused first

 @returns One reading per artifact, in archive order

 @example
 ```ts
 const readings = await readArchiveSubjects({ archiveDir, cloneDir, readFile: readCorpusFile, listFiles: listArtifactFiles, },);
 ```
 */
export async function readArchiveSubjects(
  {
    archiveDir,
    cloneDir,
    readFile,
    listFiles,
  }: {
    readonly archiveDir: string;
    readonly cloneDir: string;
    readonly readFile: typeof readCorpusFile;
    readonly listFiles: typeof listArtifactFiles;
  },
): Promise<readonly SettledArtifactReading[]> {
  /**
   Every artifact path, flattened in archive order.
   */
  const located = await locateSettledArtifacts({
    archiveDir,
    listFiles,
  },);

  return await allInInputOrder({
    members: located.map(
      async function read(at,): Promise<SettledArtifactReading> {
        return await readArtifactSubjects({
          archiveDir,
          runSetDir: at.runSetDir,
          runSet: at.runSet,
          artifactFile: at.artifactFile,
          cloneDir,
          readFile,
        },);
      },
    ),
  },);
}

//endregion Settled audit input
