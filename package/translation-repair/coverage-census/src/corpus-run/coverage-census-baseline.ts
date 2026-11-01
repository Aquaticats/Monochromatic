import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type CensusStretch,
  compareSourceThenLine,
  isUnmappedSource,
} from './coverage-census-report.ts';

//region Coverage census baseline reading
// Ledger T8: how a batch's run reads an earlier census, so the batch can prove
// it reached the stretches it claims.
//
// STRETCHES ARE MATCHED BY LINE, since a census records lines, and adding or
// removing one line renumbers the rest of the file. A source edited since the
// earlier census's commit keeps that census's stretches at lines that now
// hold other code, and matching them there read a moved stretch as run (B34's
// batch printed "ran 13" over sources it had edited). So an edited source is
// left out of the line match entirely and named with this run's own standing,
// which is all that speaks for it; the git comparison that names it is
// `sourcesEditedSince` in `coverage-census-commit.ts`.
//
// BOTH CENSUSES ARE READ, since each speaks for code the other does not: the
// baseline's stretches say what this run reached, and this run's stretches
// say what it left cold that the baseline did not (ledger B61).

/**
 What a baseline stretch reads as in a later run.

 @example
 ```ts
 const status: StretchStatus = 'ran';
 ```
 */
export type StretchStatus = 'not loaded' | 'ran' | 'still cold';

/**
 What a batch reads of an earlier census.

 @example
 ```ts
 const baseline: BaselineCensus = { head: 'c0ffee123', stretches: [], loadedSources: new Set(['src/nap.ts',],), };
 ```
 */
export type BaselineCensus = {
  /**
   Commit it was taken at, with the tree matching it, which a later reading
   compares the tree against to tell which sources have been edited since.
   */
  readonly head: string;

  /**
   Its stretches.
   */
  readonly stretches: readonly CensusStretch[];

  /**
   Sources its loaded bundles carried, which tell a claimed source holding no
   stretch because the baseline ran all of it from one holding none because
   the baseline never loaded it.
   */
  readonly loadedSources: ReadonlySet<string>;
};

/**
 A claimed source the baseline holds no stretch in, with both censuses'
 standing for it.

 @example
 ```ts
 const claim: EmptyClaim = { source: 'src/nap.ts', loadedAtBaseline: false, loadedNow: true, coldNow: 0, };
 ```
 */
export type EmptyClaim = {
  /**
   Source claimed.
   */
  readonly source: string;

  /**
   Whether a bundle the baseline loaded carried it; when none did, the
   baseline proves nothing of it and this run's standing is the only reading.
   */
  readonly loadedAtBaseline: boolean;

  /**
   Whether a bundle this run loaded carried it.
   */
  readonly loadedNow: boolean;

  /**
   Cold stretches this run left in it.
   */
  readonly coldNow: number;
};

/**
 A source edited since the baseline's commit, whose baseline lines name other
 code now, with this run's standing for it.

 @example
 ```ts
 const claim: EditedClaim = { source: 'src/nap.ts', loadedNow: true, coldNow: 0, };
 ```
 */
export type EditedClaim = {
  /**
   Source edited.
   */
  readonly source: string;

  /**
   Whether a bundle this run loaded carried it.
   */
  readonly loadedNow: boolean;

  /**
   Cold stretches this run left in it.
   */
  readonly coldNow: number;
};

/**
 Reads each baseline stretch in the named sources against a later run.

 A stretch STILL COLD overlaps a cold stretch of the later run in the same
 source; one NOT LOADED sits in a source no bundle of the later run carried,
 which proves nothing either way; the rest RAN. A source edited since the
 baseline's commit is left out, since its baseline lines name other code
 now; `editedClaimsOf` names it with this run's standing instead. Code no
 source map places is left out too (ledger B105): it has no lines to
 compare, and its source names a bundle a rebuild renames, so it read as not
 loaded at every census whose bundle hash moved; the census's own report
 counts it.

 @param baseline - stretches of the earlier census

 @param current - stretches of the later run

 @param loadedSources - sources the later run's loaded bundles carry

 @param sources - sources the batch claims, empty for every source

 @param edited - sources edited since the baseline's commit

 @returns Each baseline stretch in those sources, outside the edited ones,
 with its status

 @example
 ```ts
 const read = baselineStatusesOf({ baseline, current, loadedSources, sources: new Set(['src/nap.ts',],), edited, },);
 ```
 */
export function baselineStatusesOf(
  {
    baseline,
    current,
    loadedSources,
    sources,
    edited,
  }: {
    readonly baseline: readonly CensusStretch[];
    readonly current: readonly CensusStretch[];
    readonly loadedSources: ReadonlySet<string>;
    readonly sources: ReadonlySet<string>;
    readonly edited: ReadonlySet<string>;
  },
): readonly {
  readonly stretch: CensusStretch;
  readonly status: StretchStatus;
}[] {
  return baseline
    .filter(function claimed(stretch,): boolean {
      return ((sources.size === 0) || sources.has(stretch.source,))
        && (!edited.has(stretch.source,))
        && (!isUnmappedSource({ source: stretch.source, },));
    },)
    .map(function statusOf(stretch,) {
      if (!loadedSources.has(stretch.source,))
        return {
          stretch,
          status: 'not loaded',
        } as const;
      /**
       Whether a later cold stretch in the same source shares a line with it.
       */
      const overlapped = current.some(function overlaps(later,): boolean {
        return (later.source === stretch.source)
          && (later.startLine <= stretch.endLine)
          && (later.endLine >= stretch.startLine);
      },);
      return {
        stretch,
        status: overlapped ? 'still cold' : 'ran',
      } as const;
    },);
}

/**
 Counts this run's cold stretches in one source.

 @param source - source counted

 @param current - stretches of this run

 @returns How many of them sit in that source

 @example
 ```ts
 const coldNow = coldStretchesIn({ source: 'src/nap.ts', current, },);
 ```
 */
function coldStretchesIn(
  {
    source,
    current,
  }: {
    readonly source: string;
    readonly current: readonly CensusStretch[];
  },
): number {
  return current.filter(function inSource(stretch,): boolean {
    return stretch.source === source;
  },)
    .length;
}

/**
 Names each claimed source the baseline holds no stretch in, which the
 stretch statuses count nowhere: the baseline either ran all of it or never
 loaded it (a source only an unloaded bundle carried), and in the second case
 only this run's standing speaks for it. The placement batch of ledger T8
 read `ran 0, still cold 0, not loaded 0` for a source its baseline never
 loaded, which reads as nothing left to do. A source edited since the
 baseline's commit is left to `editedClaimsOf`, since "ran whole there" says
 nothing of code written after it.

 @param baseline - earlier census read

 @param edited - sources edited since its commit

 @param current - stretches of this run

 @param loadedSources - sources this run's loaded bundles carry

 @param sources - sources the batch claims, empty for every source

 @returns Each such claimed source, sorted, with both censuses' standing

 @example
 ```ts
 const claims = emptyClaimsOf({ baseline, edited, current, loadedSources, sources: new Set(['src/nap.ts',],), },);
 ```
 */
export function emptyClaimsOf(
  {
    baseline,
    edited,
    current,
    loadedSources,
    sources,
  }: {
    readonly baseline: BaselineCensus;
    readonly edited: ReadonlySet<string>;
    readonly current: readonly CensusStretch[];
    readonly loadedSources: ReadonlySet<string>;
    readonly sources: ReadonlySet<string>;
  },
): readonly EmptyClaim[] {
  /**
   The baseline's stretches, and the sources its loaded bundles carried.
   */
  const {
    stretches: baselineStretches,
    loadedSources: loadedAtBaseline,
  } = baseline;
  /**
   Sources holding a baseline stretch.
   */
  const stretched = new Set(baselineStretches.map(function sourceOf(stretch,): string {
    return stretch.source;
  },),);
  return textsInCodePointOrder({ texts: [...sources,]
    .filter(function holdsNone(source,): boolean {
      return (!stretched.has(source,)) && (!edited.has(source,));
    },), },)
    .map(function standing(source,): EmptyClaim {
      return {
        source,
        loadedAtBaseline: loadedAtBaseline.has(source,),
        loadedNow: loadedSources.has(source,),
        coldNow: coldStretchesIn({
          source,
          current,
        },),
      };
    },);
}

/**
 Names each source edited since the baseline's commit that the reading would
 otherwise have read, with this run's standing for it.

 WHICH SOURCES: the claimed ones, or, when the batch claims none, those
 either census holds a stretch in, so a reading of every source does not
 list every document and test changed since, and does list a source added or
 edited since that this run left cold. Reading the baseline's sources alone
 counted such a source nowhere, since the baseline holds no stretch there
 (ledger B61).

 @param baseline - earlier census read

 @param edited - sources edited since its commit

 @param current - stretches of this run

 @param loadedSources - sources this run's loaded bundles carry

 @param sources - sources the batch claims, empty for every source

 @returns Each such source, sorted, with whether this run loaded it and the
 cold stretches it left

 @example
 ```ts
 const claims = editedClaimsOf({ baseline, edited, current, loadedSources, sources: new Set(['src/nap.ts',],), },);
 ```
 */
export function editedClaimsOf(
  {
    baseline,
    edited,
    current,
    loadedSources,
    sources,
  }: {
    readonly baseline: BaselineCensus;
    readonly edited: ReadonlySet<string>;
    readonly current: readonly CensusStretch[];
    readonly loadedSources: ReadonlySet<string>;
    readonly sources: ReadonlySet<string>;
  },
): readonly EditedClaim[] {
  /**
   Sources the reading covers.
   */
  const read = (sources.size === 0)
    ? new Set([
      ...baseline.stretches,
      ...current,
    ].map(function sourceOf(stretch,): string {
      return stretch.source;
    },),)
    : sources;
  return textsInCodePointOrder({ texts: [...read,]
    .filter(function wasEdited(source,): boolean {
      return edited.has(source,);
    },), },)
    .map(function standing(source,): EditedClaim {
      return {
        source,
        loadedNow: loadedSources.has(source,),
        coldNow: coldStretchesIn({
          source,
          current,
        },),
      };
    },);
}

/**
 A stretch this run left cold holding a line no baseline stretch of its
 source held.

 @example
 ```ts
 const cold: ColdSince = { stretch, loadedAtBaseline: true, };
 ```
 */
export type ColdSince = {
  /**
   The stretch, as this run recorded it.
   */
  readonly stretch: CensusStretch;

  /**
   Whether a bundle the baseline loaded carried its source: when one did, the
   baseline ran the lines this stretch gained; when none did, the baseline
   proves nothing of them.
   */
  readonly loadedAtBaseline: boolean;
};

/**
 Whether earlier stretches hold every line of a later one, read line by line
 so that two stretches meeting across a gap do not hold the line between.

 @param stretch - later stretch

 @param held - earlier stretches in its source

 @returns Whether every line it spans lies in one of them

 @example
 ```ts
 const unchanged = heldThroughout({ stretch, held, },);
 ```
 */
function heldThroughout(
  {
    stretch,
    held,
  }: {
    readonly stretch: CensusStretch;
    readonly held: readonly CensusStretch[];
  },
): boolean {
  /**
   First and last line of the later stretch.
   */
  const {
    startLine,
    endLine,
  } = stretch;
  return Array.from(
    { length: (endLine - startLine) + 1, },
    function lineAt(
      _unused,
      offset,
    ): number {
      return startLine + offset;
    },
  )
    .every(function isHeld(line,): boolean {
      return held.some(function holds(earlier,): boolean {
        return (earlier.startLine <= line) && (line <= earlier.endLine);
      },);
    },);
}

/**
 Names each stretch this run left cold in lines the baseline did not, which
 reading the baseline's own stretches counts nowhere: before ledger B61 a
 change that left code cold read clean, and the three stretches of other
 packages `census-5PAw2O` left cold again went unprinted. A stretch reaching
 past the baseline's lines is named too, since the lines it gained went cold.
 A source edited since the baseline's commit is left to `editedClaimsOf`,
 since its baseline lines name other code now. Code no source map places is
 left out too: it has no lines to compare, and it takes its bundle's name, so
 a rebuild that renamed the bundle read its unchanged stretches as cold
 since; the census's own report counts it.

 @param baseline - earlier census read

 @param current - stretches of this run

 @param sources - sources the batch claims, empty for every source

 @param edited - sources edited since the baseline's commit

 @returns Each such stretch with whether the baseline loaded its source,
 sorted by source and then line

 @example
 ```ts
 const cold = coldSinceOf({ baseline, current, sources: new Set(), edited, },);
 ```
 */
export function coldSinceOf(
  {
    baseline,
    current,
    sources,
    edited,
  }: {
    readonly baseline: BaselineCensus;
    readonly current: readonly CensusStretch[];
    readonly sources: ReadonlySet<string>;
    readonly edited: ReadonlySet<string>;
  },
): readonly ColdSince[] {
  /**
   The baseline's stretches, and the sources its loaded bundles carried.
   */
  const {
    stretches: baselineStretches,
    loadedSources: loadedAtBaseline,
  } = baseline;
  /**
   The baseline's stretches by source.
   */
  const heldBySource = Map.groupBy(
    baselineStretches,
    function sourceOf(stretch,): string {
      return stretch.source;
    },
  );
  return current
    .filter(function coldSince(stretch,): boolean {
      /**
       Source it sits in.
       */
      const { source, } = stretch;
      return ((sources.size === 0) || sources.has(source,))
        && (!edited.has(source,))
        && (!isUnmappedSource({ source, },))
        && (!heldThroughout({
          stretch,
          held: heldBySource.get(source,) ?? [],
        },));
    },)
    .toSorted(function bySourceThenLine(
      left,
      right,
    ): number {
      return compareSourceThenLine({
        left,
        right,
      },);
    },)
    .map(function standing(stretch,): ColdSince {
      return {
        stretch,
        loadedAtBaseline: loadedAtBaseline.has(stretch.source,),
      };
    },);
}

//endregion Coverage census baseline reading
