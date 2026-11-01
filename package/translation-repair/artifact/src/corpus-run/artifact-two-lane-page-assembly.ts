import { requireExactKeys, } from '../artifact-exact-guard.ts';
import {
  ArtifactParseError,
  requireArray,
  requireCount,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';

//region Artifact page assembly
// WHAT THE PAGE-LEVEL GUARD DID TO THE COMPOSED PAGE. Each lane guards its own
// assembly, but the page ships per slice whatever the polish, the
// consolidation and the contest chose, and that composition is a document
// nobody assembled: the twenty-second `hakureico` pass of 2026-09-09 carried
// the consolidation's fresh rendering of two notes at slice 14 with the
// orphan the translate lane had already trimmed, and the page guard refused
// the page. The pass now runs the same guard over the composed page and
// records here what it cut and what it took back, so the artifact says what
// the page carries and every reader composes the same page.
//
// APPLIED FIRST when a slice's text is read (`would-ship-text.ts`), ahead of
// the polish, the consolidation and the contest, since it describes the page
// those three composed.

/**
 Page-level guard outcome recorded in a settled artifact.

 @example
 ```ts
 const assembly: ArtifactPageAssembly = { trimmed: [], withdrawn: [], findings: [], };
 ```
 */
export type ArtifactPageAssembly = {
  /**
   Slices whose text the page-assembly passes or the guard rewrote, with the
   text the page carries: an orphan definition block cut, a heading or name
   restored, a date or word in its Canadian form, and the like. NAMED FOR THE
   FIRST REWRITE IT RECORDED, which was a trim. A withdrawn slice appears
   here too where a pass rewrote the archive text standing there (ledger K5),
   and this row is what the page carries.
   */
  readonly trimmed: readonly SliceReplacement[];

  /**
   Slices whose lane row the guard took back, so the page carries the
   archive's own wording there (as a `trimmed` row gives it, where a pass
   rewrote it), or nothing at an anchor.
   */
  readonly withdrawn: readonly number[];

  /**
   What the guard did, in the assembly guard's wording.
   */
  readonly findings: readonly string[];
};

/**
 Page assembly of an artifact whose guard cut nothing and took nothing back,
 and of every artifact written before the guard existed.
 */
export const NO_PAGE_ASSEMBLY: ArtifactPageAssembly = {
  trimmed: [],
  withdrawn: [],
  findings: [],
};

/**
 Keys the section is written with.
 */
const PAGE_ASSEMBLY_KEYS: readonly string[] = [
  'trimmed',
  'withdrawn',
  'findings',
];

/**
 Keys a trimmed replacement is written with.
 */
const TRIMMED_KEYS: readonly string[] = [
  'sliceIndex',
  'replacementText',
];

/**
 Reads one trimmed replacement.

 @param value - recorded replacement

 @param path - where it sits, for the parse error

 @returns The replacement

 @example
 ```ts
 const replacement = parseTrimmed({ value, path: `${path}.trimmed[0]`, },);
 ```
 */
function parseTrimmed(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): SliceReplacement {
  /**
   The replacement as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: TRIMMED_KEYS,
    path,
  },);
  return {
    sliceIndex: requireCount({
      value: record.sliceIndex,
      path: `${path}.sliceIndex`,
    },),
    replacementText: requireString({
      value: record.replacementText,
      path: `${path}.replacementText`,
    },),
  };
}

/**
 Reads the page assembly section of a settled artifact.

 @param value - recorded section, absent on artifacts written before it

 @param path - where it sits, for the parse error

 @param required - whether this generation writes the section, so its
 absence is a broken file rather than an older one

 @returns The section, empty on an older artifact

 @throws {@link ArtifactParseError} when a required section is absent, any
 field is the wrong shape, or two trimmed rows name one slice

 @example
 ```ts
 const assembly = parsePageAssembly({ value: artifact.pageAssembly, path: `${id}.pageAssembly`, required, },);
 ```
 */
export function parsePageAssembly(
  {
    value,
    path,
    required,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly required: boolean;
  },
): ArtifactPageAssembly {
  if (value === undefined) {
    if (required) {
      throw new ArtifactParseError({
        path,
        reason: 'recorded page assembly from generation thirteen',
      },);
    }
    return NO_PAGE_ASSEMBLY;
  }
  /**
   The section as a record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: PAGE_ASSEMBLY_KEYS,
    path,
  },);
  /**
   Rows giving the text the page carries, as recorded.
   */
  const trimmed = requireArray({
    value: record.trimmed,
    path: `${path}.trimmed`,
  },)
    .map(function toReplacement(
      entry,
      at,
    ): SliceReplacement {
      return parseTrimmed({
        value: entry,
        path: `${path}.trimmed[${String(at,)}]`,
      },);
    },);

  /**
   Slices a row has already named. ONE ROW PER SLICE is what the page guard
   writes: its own trims, which the splice already holds to one per slice,
   then the rows of a map keyed by slice for the slices its trims do not
   name. `pageAssemblyOverrideAt` takes the first row naming a slice on that
   footing, so a file holding two is refused here, where it is read, rather
   than answered with whichever stands first.
   */
  const named = new Set<number>();
  for (const row of trimmed) {
    if (named.has(row.sliceIndex,)) {
      throw new ArtifactParseError({
        path: `${path}.trimmed`,
        reason: `one row per slice; slice ${String(row.sliceIndex,)} appears more than once`,
      },);
    }
    named.add(row.sliceIndex,);
  }
  return {
    trimmed,
    withdrawn: requireArray({
      value: record.withdrawn,
      path: `${path}.withdrawn`,
    },)
      .map(function toIndex(
        entry,
        at,
      ): number {
        return requireCount({
          value: entry,
          path: `${path}.withdrawn[${String(at,)}]`,
        },);
      },),
    findings: requireArray({
      value: record.findings,
      path: `${path}.findings`,
    },)
      .map(function toFinding(
        entry,
        at,
      ): string {
        return requireString({
          value: entry,
          path: `${path}.findings[${String(at,)}]`,
        },);
      },),
  };
}

/**
 What the page assembly says about one slice.

 @example
 ```ts
 const override: PageAssemblyOverride = { kind: 'trimmed', text: '[^1]: one', };
 ```
 */
export type PageAssemblyOverride =
  | {
    /**
     The page carries this text, the composed text with a block cut.
     */
    readonly kind: 'trimmed';

    /**
     Text the page carries.
     */
    readonly text: string;
  }
    | {
    /**
     The page carries the archive's own wording here, or nothing at an anchor.
     */
      readonly kind: 'withdrawn';
    }
    | {
    /**
     The guard left this slice as the stages composed it.
     */
      readonly kind: 'untouched';
    };

/**
 Reads what the page assembly did to one slice.

 @param pageAssembly - recorded section

 @param sliceIndex - slice asked about

 @returns The override, or that there is none

 @example
 ```ts
 const override = pageAssemblyOverrideAt({ pageAssembly, sliceIndex: 14, },);
 ```
 */
export function pageAssemblyOverrideAt(
  {
    pageAssembly,
    sliceIndex,
  }: {
    readonly pageAssembly: ArtifactPageAssembly;
    readonly sliceIndex: number;
  },
): PageAssemblyOverride {
  /**
   The trimmed replacement for this slice, if any: READ FIRST, because a slice
   the guard took back carries one where a page pass rewrote the archive text
   standing there (ledger K5), and that text is what the page carries. At most
   one row names a slice, which `parsePageAssembly` refuses a file over.
   */
  const trimmed = pageAssembly
    .trimmed
    .find(function namesIt(replacement,): boolean {
      return replacement.sliceIndex === sliceIndex;
    },);
  if (trimmed !== undefined) {
    return {
      kind: 'trimmed',
      text: trimmed.replacementText,
    };
  }
  if (pageAssembly
    .withdrawn
    .includes(sliceIndex,))
    return { kind: 'withdrawn', };
  return { kind: 'untouched', };
}

//endregion Artifact page assembly
