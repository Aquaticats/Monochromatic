import {
  parseDocument,
  type ParseFinding,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type FootnoteGraphFinding,
  type MarkdownParseError,
  requireMarkdownRefusal,
  withMarkdownExhaustionRefused,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Assembly regressions
// Whole-document differences shared by ordinary assembly and counterfactual withdrawal.

/**
 Key identifying a footnote defect across two documents.

 Deliberately drops the node id: block indices move when a slice changes
 length, so keeping it would report every surviving defect as a new one.

 @param finding - defect from a parsed document's footnote graph

 @returns Stable key

 @example
 ```ts
 const key = findingKey({ finding, },);
 ```
 */
function findingKey(
  { finding, }: { readonly finding: FootnoteGraphFinding; },
): string {
  return `${finding.kind} ${finding.convention} ${finding.identifier}`;
}

/**
 Footnote defects present in an assembled document that its incumbent did not
 already carry.

 Counted rather than set-differenced, so a second duplicate definition of an
 identifier the archive already duplicated is still reported.

 @param incumbentText - translation as it stands

 @param assembledText - document spliced from the surviving replacements

 @returns Defects the assembly introduced

 @example
 ```ts
 const introduced = introducedFootnoteFindings({ incumbentText, assembledText, },);
 ```
 */
export function introducedFootnoteFindings(
  {
    incumbentText,
    assembledText,
  }: {
    readonly incumbentText: string;
    readonly assembledText: string;
  },
): readonly FootnoteGraphFinding[] {
  /**
   Defects the archive already carried, counted by key.
   */
  const inherited = new Map<string, number>();
  for (const finding of parseDocument({ text: incumbentText, },)
    .footnoteGraph
    .findings) {
    /**
     Key of one inherited defect.
     */
    const key = findingKey({ finding, },);
    inherited.set(
      key,
      (inherited.get(key,) ?? 0) + 1,
    );
  }

  /**
   Defects with no inherited counterpart left to account for them.
   */
  const introduced: FootnoteGraphFinding[] = [];
  for (const finding of parseDocument({ text: assembledText, },)
    .footnoteGraph
    .findings) {
    /**
     Key of one assembled defect.
     */
    const key = findingKey({ finding, },);

    /**
     Inherited defects of this key still unaccounted for.
     */
    const remaining = inherited.get(key,) ?? 0;
    if (remaining > 0) {
      inherited.set(
        key,
        remaining - 1,
      );
      continue;
    }
    introduced.push(finding,);
  }
  return introduced;
}

/**
 Parse tolerances that mean the document became LESS parseable, rather than
 that the parser worked around something ordinary.

 A masked comment and a blanked invisible line are ordinary. An unterminated
 comment swallows everything after it, and an MDX downgrade means the strict
 parser refused the document and the loose one accepted it as plain markdown.
 Both are whole-document effects that a per-slice check cannot see: masking
 runs over the whole body before parsing, so one slice's stray `<!--` hides
 markers in slices nobody touched.

 A DELIBERATE SUBSET of the parser's finding kinds, typed by them so a
 misspelt member fails to compile. LEFT OUT are `html-comment-skipped` (the
 masked comment) and `invisible-line-masked` (the blanked line), the two
 ordinary cases. A finding kind the parser gains is therefore not
 a regression until it is added here.
 */
const STRUCTURAL_REGRESSION_KINDS: readonly ParseFinding['kind'][] = [
  'unterminated-html-comment',
  'mdx-downgraded',
];

/**
 The regression an assembly carries when its page is read by no grammar.

 THE STRONGEST ONE THERE IS. The nesting scan keeps what a fence holds across
 lines, so a fence one replacement opens and never closes is closed by the fence
 line of the replacement after it, and what that one skipped as code reads on
 the page as a paragraph nested past the bound. Each replacement was read in
 its own slice, where it read.
 */
export const UNREADABLE_PAGE = 'unreadable-page';

/**
 Counts one parse-finding kind in a document.

 @param text - document to parse

 @param kind - finding kind to count

 @returns How many the parser reported

 @example
 ```ts
 const count = countParseFindings({ text, kind: 'mdx-downgraded', },);
 ```
 */
function countParseFindings(
  {
    text,
    kind,
  }: {
    readonly text: string;
    readonly kind: string;
  },
): number {
  return withMarkdownExhaustionRefused({
    parse: parseDocument,
    input: { text, },
  },)
    .parseFindings
    .filter(function isKind(finding,): boolean {
      return finding.kind === kind;
    },)
    .length;
}

/**
 Structural parse regressions an assembled document carries beyond its
 incumbent's.

 @param incumbentText - translation as it stands

 @param assembledText - document spliced from the surviving replacements

 @returns Kinds the assembly carries MORE of, each named once; how many more
 is deliberately not reported, since one is already enough to withdraw over.
 A page the plain grammar refuses, nested past the bound, is the one kind
 {@link UNREADABLE_PAGE} alone, since it has no findings to count

 @throws {@link MarkdownParseError} when the
 incumbent is read by no grammar, which is the archive's own text and was read
 when the entry was prepared

 @example
 ```ts
 const worse = introducedStructuralRegressions({ incumbentText, assembledText, },);
 ```
 */
export function introducedStructuralRegressions(
  {
    incumbentText,
    assembledText,
  }: {
    readonly incumbentText: string;
    readonly assembledText: string;
  },
): readonly string[] {
  /**
   What the incumbent carries of each kind, read first so that a refusal of
   the incumbent is never charged to the assembly.
   */
  const standing = STRUCTURAL_REGRESSION_KINDS.map(function counted(kind,): {
    readonly kind: ParseFinding['kind'];
    readonly count: number;
  } {
    return {
      kind,
      count: countParseFindings({
        text: incumbentText,
        kind,
      },),
    };
  },);
  try {
    return standing
      .filter(function worsened({
        kind,
        count,
      },): boolean {
        return countParseFindings({
          text: assembledText,
          kind,
        },) > count;
      },)
      .map(function kindOf({ kind, },): string {
        return kind;
      },);
  }
  catch (error) {
    // Only the plain grammar's own refusal makes the page unreadable; any
    // other failure is an unexpected state that must keep propagating.
    requireMarkdownRefusal({ error, },);
    return [UNREADABLE_PAGE,];
  }
}

//endregion Assembly regressions
