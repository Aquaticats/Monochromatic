import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  contributorAuthorityFindings,
  type DeclaredNamePair,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  type DisputedWording,
  disputedWordingFindings,
} from './disputed-wording.ts';
import { validateFrontMatterTranslation, } from './front-matter-translation.ts';
import { compareLineCounts, } from './line-structure-guard.ts';
import {
  compareBlocks,
  type PageGrammar,
  quotedPassageFloorFindings,
  readSliceSkeleton,
  type SliceSkeleton,
  sourceOnlyBreakFindings,
  substituteBreakFindings,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import { atomFindings, } from './translate-atom-rendering.ts';
import { neutralPronounFindings, } from './translate-neutral-pronoun.ts';
import { definitionLeakFindings, } from './translate-definition-leak.ts';
import { leakedEscapeFindings, } from './translate-escape-leak.ts';
import { readMarkdownGround, } from './translate-floor-ground.ts';
import { sourceCarryFindings, } from './translate-source-carry.ts';
import { sheetLeakFindings, } from './translate-sheet-leak.ts';
import { untranslatedOrResidueFindings, } from './translate-han-residue.ts';
import { proseNameFindings, } from './translate-prose-name.ts';
import { signerHandleFindings, } from './translate-signer-handle.ts';

//region Translate validation
// Compares a candidate translation against its ORIGINAL on everything that
// survives translation: the block skeleton, and the references and code inside
// it.
//
// This exists because the deterministic apply gate cannot serve here. Every
// policy in that gate is anchored to an EDIT bounded by an envelope some
// accepted issue named, and a whole-slice replacement has no envelope. Faking
// one that spans the slice fails in both directions: with no licensed quotes
// the preservation rule rejects nearly every legitimate translation, and
// licensing the whole envelope makes it vacuous.
//
// FINDINGS ARE WRITTEN FOR THE MODEL THAT WROTE THE CANDIDATE, not for a log.
// By user decision of 2026-08-15 an invalid candidate is not dropped: it goes
// back to its own author with these sentences, and that model answers with a
// revision, an inability, or a defence of what it produced. So each finding
// names what the original has, what the candidate has, and nothing else.
//
// A QUOTED PASSAGE NESTED BELOW THE TOP LEVEL IS COMPARED TOO (ledger B110).
// The block comparison reads only the top-level block sequence, so a
// candidate keeping every top-level block while turning a blockquote nested
// inside a container tag, a list, a footnote, or another blockquote into
// prose passed here before; `quotedPassageFloorFindings` reads the parse's
// own any-depth count and refuses that candidate before any judge sees it.

/**
 What comparing a candidate against its original found.

 @example
 ```ts
 const validation: SliceValidation = { kind: 'valid', pageGrammar: 'strict', };
 ```
 */
export type SliceValidation =
  | {
    readonly kind: 'valid';

    /**
     Grammar that read the page behind this pass.

     ON THE PASS AND NOT THE REFUSAL, because a refusal already names the
     blocks it compared and shows which reading produced them, while a pass
     carries no evidence at all. A pass resting on the relaxed grammar is
     weaker than one resting on the strict grammar, and the repo's parser
     policy is that a downgrade never happens silently.
     */
    readonly pageGrammar: PageGrammar;
  }
    | {
      readonly kind: 'invalid';

    /**
     One sentence per divergence, addressed to the model that wrote the
     candidate.
     */
      readonly findings: readonly string[];
    }
    | {
      readonly kind: 'unknown';

    /**
     Why no comparison was possible.
     */
      readonly detail: string;
    };

/**
 Verdict where no grammar could read the original or the page: the floors
 that read text rather than blocks still run (ledger F-5), and with none of
 them refusing, the candidate is left unvalidated rather than passed.

 @param sourceText - original slice

 @param candidateText - proposed translation of it

 @param pageText - text this candidate would replace, empty where the slice
 has none

 @param lineStructured - whether the line-structure rule governs this slice

 @param declared - name pairs the front matter declares, which govern a
 signer they name

 @param unread - which side no grammar read and why, the verdict's detail
 when no text floor refuses

 @returns A text floor's refusal, or no verdict

 @example
 ```ts
 return grammarFreeVerdict({ sourceText, candidateText, pageText, lineStructured, declared, unread, },);
 ```
 */
function grammarFreeVerdict(
  {
    sourceText,
    candidateText,
    pageText,
    lineStructured,
    declared,
    unread,
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
    readonly pageText: string;
    readonly lineStructured: boolean;
    readonly declared: readonly DeclaredNamePair[];
    readonly unread: string;
  },
): SliceValidation {
  /**
   Findings of the floors that read text rather than blocks.
   */
  const grammarFree = [
    ...untranslatedOrResidueFindings({
      sourceText,
      candidateText,
      pageText,
    },),
    ...signerHandleFindings({
      sourceText,
      candidateText,
      pageText,
      declared,
    },),
    ...proseNameFindings({
      sourceText,
      candidateText,
      pageText,
      declared,
    },),
    ...compareLineCounts({
      lineStructured,
      sourceText,
      candidateText,
      pageText,
    },),
    ...neutralPronounFindings({
      sourceText,
      candidateText,
    },),
  ];
  if (grammarFree.length > 0)
    return {
      kind: 'invalid',
      findings: grammarFree,
    };
  return {
    kind: 'unknown',
    detail: unread,
  };
}

/**
 Checks one candidate translation against the original and the page it
 replaces.

 @param sourceText - original slice

 @param candidateText - proposed translation of it

 @param pageText - text this candidate would replace, empty where the slice
 has none. Its shape is a floor the candidate carries rather than a ceiling,
 so a rendering restoring what the page left out stays valid

 @param syntax - explicit syntax role, absent for ordinary Markdown

 @param lineStructured - whether the line-structure rule governs this slice,
 which makes merging its lines a fault. Defaults to false, so a caller that
 cannot say leaves the check off rather than guessing at it from the slice
 alone: the decision is a union over the slice AND its enclosing chunk, and
 the slice half alone covers 55 slices where the union covers 211

 @param declared - name pairs the front matter declares, so a linked title
 naming a declared person is held to the declared form (class one hundred
 fourteen). Defaults to none, which leaves that floor silent

 @param disputedWordings - wordings a disputed slice refuses, whose disputed
 reading the repair did not fix (owner, 2026-09-27, "No eligible
 standing"). Defaults to none

 @returns Verdict, with findings written for the model that wrote it

 @example
 ```ts
 const validation = validateTranslatedSlice({ sourceText, candidateText, },);
 ```
 */
export function validateTranslatedSlice(
  {
    sourceText,
    candidateText,
    pageText = '',
    syntax,
    lineStructured = false,
    declared = [],
    disputedWordings = [],
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
    readonly pageText?: string;
    readonly syntax?: SliceSyntax;
    readonly lineStructured?: boolean;
    readonly declared?: readonly DeclaredNamePair[];
    readonly disputedWordings?: readonly DisputedWording[];
  },
): SliceValidation {
  /**
   Disputed wordings this candidate is, refused before any shape is read.
   */
  const disputedFindings = disputedWordingFindings({
    candidateText,
    disputedWordings,
  },);
  if (disputedFindings.length > 0) {
    return {
      kind: 'invalid',
      findings: disputedFindings,
    };
  }
  if (syntax === 'front-matter') {
    return validateFrontMatterTranslation({
      sourceText,
      pageText,
      candidateText,
    },);
  }
  /**
   Contributor authority floor applies even when source grammar is unreadable.
   */
  const contributorFindings = contributorAuthorityFindings({
    texts: Array.of(
      pageText,
      candidateText,
    ),
  },);
  if (contributorFindings.length > 0) {
    return {
      kind: 'invalid',
      findings: contributorFindings,
    };
  }
  /**
   JSON escapes that leaked into the text, refused before any shape is read.
   */
  const escapeFindings = leakedEscapeFindings({
    sourceText,
    pageText,
    candidateText,
  },);
  if (escapeFindings.length > 0) {
    return {
      kind: 'invalid',
      findings: escapeFindings,
    };
  }
  /**
   Sheet evidence blocks copied into the candidate (class forty-four), refused
   before any shape is read.
   */
  const leakFindings = sheetLeakFindings({
    sourceText,
    pageText,
    candidateText,
  },);
  if (leakFindings.length > 0) {
    return {
      kind: 'invalid',
      findings: leakFindings,
    };
  }

  /**
   A footnote the candidate defines though the original passage only refers
   to it (class forty-nine), refused before the assembly can find it twice.
   */
  const definitionFindings = definitionLeakFindings({
    sourceText,
    pageText,
    candidateText,
  },);
  if (definitionFindings.length > 0) {
    return {
      kind: 'invalid',
      findings: definitionFindings,
    };
  }
  /**
   What the original carries that the candidate must carry too: its footnote
   markers (class ninety-two), its second-person address (class
   ninety-seven), its bracketed work titles in English (class
   ninety-eight) and a declared name inside a linked title (class one
   hundred fourteen).
   */
  const carryFindings = sourceCarryFindings({
    sourceText,
    candidateText,
    pageText,
    declared,
  },);
  if (carryFindings.length > 0) {
    return {
      kind: 'invalid',
      findings: carryFindings,
    };
  }
  /**
   The original and the page, read by the one definition of whether this
   floor can compare anything, which the translate stage asks before it buys
   a round (`translate-floor-ground.ts`, ledger B43).
   */
  const ground = readMarkdownGround({
    sourceText,
    pageText,
  },);

  // THE FLOORS THAT NEED NO GRAMMAR STILL RUN (ledger F-5) where no grammar
  // reads the original: a candidate left untranslated or leaving Han in its
  // English (F-3), merging the lines of a governed slice, or keeping the
  // neutral pronoun is refused whatever the original's grammar, since none of
  // those reads a block.
  if ((ground.kind === 'blind') && (ground.side === 'original')) {
    return grammarFreeVerdict({
      sourceText,
      candidateText,
      pageText,
      lineStructured,
      declared,
      unread: ground.detail,
    },);
  }

  /**
   Shape the candidate carries.
   */
  const candidate = readSliceSkeleton({ text: candidateText, },);
  if (candidate.kind === 'unparseable')
    return {
      kind: 'invalid',
      findings: [
        `Your translation could not be parsed as Markdown: ${candidate.detail}`,
      ],
    };

  // A PAGE NEITHER GRAMMAR READS is treated as an unreadable original is, once
  // the candidate has been read, since a candidate the strict grammar refuses
  // against a readable original is that candidate's fault whatever the page
  // is. It used to fall back to the original alone and pass a candidate the
  // page never measured, reported as read by the relaxed grammar (ledger T8,
  // sixth batch).
  if (ground.kind === 'blind') {
    return grammarFreeVerdict({
      sourceText,
      candidateText,
      pageText,
      lineStructured,
      declared,
      unread: ground.detail,
    },);
  }

  /**
   Grammar that read the page, reported with a pass.
   */
  const { pageGrammar, } = ground;

  /**
   Page's shape, empty only where there is no page.

   A PAGE THE STRICT GRAMMAR REFUSES IS NOT A CANDIDATE'S FAULT, and an archive
   written before this grammar existed can be one, so `readPageSkeleton`
   (`translate-skeleton-page.ts`) downgrades the page to plain markdown rather
   than refusing the candidate.

   IT NO LONGER FALLS BACK TO THE ORIGINAL ALONE, which was a check answering
   yes to a question it had never evaluated. Measured on the sixth
   consolidation bed: a slice boundary between an opening details tag and its
   closing tag made the page unparseable, the floor lost its block list, and a
   164-character rendering passed against a 3875-character page.
   */
  const page: SliceSkeleton = ground.page;

  /**
   Page's blocks, named so the emptiness check is one step rather than three.
   */
  const pageBlocks = page.blocks;

  /**
   Whether there is a page shape to stand on.
   */
  const hasPage = (pageBlocks.length > 0);

  /**
   Original's shape, now known readable.
   */
  const expected: SliceSkeleton = ground.source;

  /**
   Candidate's shape, now known readable.
   */
  const actual: SliceSkeleton = candidate.skeleton;

  /**
   Sequence the candidate has to carry.
   */
  const floor = hasPage ? pageBlocks : expected.blocks;

  /**
   What a finding calls that sequence.
   */
  const floorName = hasPage ? 'PAGE AS IT STANDS' : 'ORIGINAL';

  /**
   What a finding calls the side an atom came from.
   */
  const atomSource = hasPage ? 'ORIGINAL or the PAGE AS IT STANDS' : 'ORIGINAL';

  /**
   Every divergence, structure first because a reference finding reads
   differently once the blocks it sits in are known to differ.
   */
  const findings = [
    ...compareBlocks({
      floor,
      floorName,
      source: expected.blocks,
      candidate: actual.blocks,
    },),
    // QUOTED PASSAGES AT EVERY DEPTH, against the same floor (ledger B110):
    // `compareBlocks` reads only the top-level sequence, so a quote nested
    // inside a container tag, a list, a footnote, or another blockquote
    // could be made prose under a matching top level.
    ...quotedPassageFloorFindings({
      floorName,
      floorQuotedPassages: hasPage ? page.quotedPassages : expected.quotedPassages,
      candidateQuotedPassages: actual.quotedPassages,
    },),
    ...sourceOnlyBreakFindings({
      pageText,
      source: expected.explicitBreaks,
      candidate: actual.explicitBreaks,
    },),
    // CLASS FORTY-TWO: a kind the page never rendered owes the original's
    // breaks even though the page has text for the slice.
    ...(hasPage
      ? substituteBreakFindings({
        pageBlocks,
        sourceBlocks: expected.blocks,
        sourceBreaks: expected.explicitBreaks,
        candidateBlocks: actual.blocks,
        candidateBreaks: actual.explicitBreaks,
      },)
      : []),
    // LEDGER F-3: the copied original, or Han left standing in English prose.
    ...untranslatedOrResidueFindings({
      sourceText,
      candidateText,
      pageText,
    },),
    // LEDGER A17: a Han signer left in Han or romanized with no meaning.
    ...signerHandleFindings({
      sourceText,
      candidateText,
      pageText,
      declared,
    },),
    ...proseNameFindings({
      sourceText,
      candidateText,
      pageText,
      declared,
    },),
    ...compareLineCounts({
      lineStructured,
      sourceText,
      candidateText,
      pageText,
    },),
    ...atomFindings({
      page: page.atoms,
      source: expected.atoms,
      candidate: actual.atoms,
      referenceName: atomSource,
    },),
    ...neutralPronounFindings({
      sourceText,
      candidateText,
    },),
  ];
  if (findings.length === 0)
    return {
      kind: 'valid',
      pageGrammar,
    };
  return {
    kind: 'invalid',
    findings,
  };
}

//endregion Translate validation
