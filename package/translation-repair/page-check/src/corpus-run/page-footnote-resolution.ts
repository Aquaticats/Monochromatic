import type { FootnoteGraphFinding, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Page footnote resolution
// THE PAGE'S OWN FOOTNOTE GRAPH, NOT THE ARCHIVE'S (decision D7 of the XingZ6020 round). The archive-relative guard
// (`page-footnote-integrity.ts`) refuses only the defects a page introduces over its archive, so an archive that
// already lacked a definition lets the page ship the reference with nowhere to land: XingZ6020 published five
// references whose definitions sat in the original's tail the archive never reached. A reader meets the page, not
// the archive, so this check reads the page alone. It REPORTS and never refuses (a run always ships); the caller
// lists it among the publish defects.

/**
 Identifiers of one kind of footnote defect, under the label the message gives them.
 */
type FootnoteGroup = Readonly<{
  /**
   What the identifiers have in common.
   */
  label: string;

  /**
   Identifiers in page order.
   */
  identifiers: readonly string[];
}>;

/**
 Failure raised when the page's footnote references and definitions do not pair up.

 @example
 ```ts
 throw new PageFootnoteResolutionError({ entryId: 'BookshopCat', unresolved: ['1'], orphaned: [], duplicated: [], });
 ```
 */
export class PageFootnoteResolutionError extends Error {
  /**
   Declares message safe to forward because it names the entry and footnote identifiers only.
   */
  readonly messageNamesOnly: true = true;

  /**
   Entry whose page failed.
   */
  readonly entryId: string;

  /**
   Identifiers referenced on the page with no definition, in page order.
   */
  readonly unresolved: readonly string[];

  /**
   Identifiers defined on the page that nothing references, in page order.
   */
  readonly orphaned: readonly string[];

  /**
   Identifiers the page defines more than once, in page order.
   */
  readonly duplicated: readonly string[];

  /**
   @param entryId - entry refused

   @param unresolved - referenced identifiers with no definition

   @param orphaned - defined identifiers with no reference

   @param duplicated - identifiers defined more than once
   */
  public constructor(
    {
      entryId,
      unresolved,
      orphaned,
      duplicated,
    }: {
      readonly entryId: string;
      readonly unresolved: readonly string[];
      readonly orphaned: readonly string[];
      readonly duplicated: readonly string[];
    },
  ) {
    /**
     Every group, each named by what it holds.
     */
    const all: readonly FootnoteGroup[] = [
      {
        label: 'references without a definition',
        identifiers: unresolved,
      },
      {
        label: 'definitions without a reference',
        identifiers: orphaned,
      },
      {
        label: 'identifiers defined more than once',
        identifiers: duplicated,
      },
    ];
    /**
     The groups that have members, as the message prints them.
     */
    const groups = all
      .filter(function populated({ identifiers, },): boolean {
        return identifiers.length > 0;
      },)
      .map(function spelled({
        label,
        identifiers,
      },): string {
        return `${label}: ${identifiers.join(', ',)}`;
      },);
    super(`entry ${entryId}: page footnotes do not resolve; ${groups.join('; ',)}`,);
    this.name = 'PageFootnoteResolutionError';
    this.entryId = entryId;
    this.unresolved = unresolved;
    this.orphaned = orphaned;
    this.duplicated = duplicated;
  }
}

/**
 Identifiers of the findings of one kind, each once, in page order.

 @param findings - the page's footnote graph findings

 @param kind - which kind to keep

 @returns Distinct identifiers

 @example
 ```ts
 const unresolved = identifiersOf({ findings, kind: 'unresolved-reference', },);
 ```
 */
function identifiersOf(
  {
    findings,
    kind,
  }: {
    readonly findings: readonly FootnoteGraphFinding[];
    readonly kind: FootnoteGraphFinding['kind'];
  },
): readonly string[] {
  return [
    ...new Set(
      findings
        .filter(function ofKind(finding,): boolean {
          return finding.kind === kind;
        },)
        .map(function identifierOf({ identifier, },): string {
          return identifier;
        },),
    ),
  ];
}

/**
 Refuses a page whose footnote references and definitions do not pair up, whatever its archive carried.

 @param entryId - corpus entry being published

 @param pageText - assembled final page

 @throws {@link PageFootnoteResolutionError} when a reference lacks a definition, a definition lacks a reference, or an
 identifier is defined twice

 @example
 ```ts
 assertPageFootnotesResolve({ entryId: 'BookshopCat', pageText, },);
 ```
 */
export function assertPageFootnotesResolve(
  {
    entryId,
    pageText,
  }: {
    readonly entryId: string;
    readonly pageText: string;
  },
): void {
  /**
   The page's own footnote graph.
   */
  const { footnoteGraph, } = parseDocument({ text: pageText, },);
  /**
   Findings the page's graph holds.
   */
  const { findings, } = footnoteGraph;
  if (findings.length === 0)
    return;
  throw new PageFootnoteResolutionError({
    entryId,
    unresolved: identifiersOf({
      findings,
      kind: 'unresolved-reference',
    },),
    orphaned: identifiersOf({
      findings,
      kind: 'orphan-definition',
    },),
    duplicated: identifiersOf({
      findings,
      kind: 'duplicate-definition',
    },),
  },);
}

//endregion Page footnote resolution
