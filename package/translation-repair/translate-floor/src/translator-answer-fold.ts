import { foldInvisibleVariants, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { HeardVoice, } from '@monochromatic-dev/translation-repair-stage/ts';
import type { TranslateReportWire, } from './translate-wire.ts';

//region Translator answer fold
// A TRANSLATOR'S ANSWER AS IT SHIPS, folded where a lane first holds it and
// before anything reads it (ledger B112).
//
// The translate and consolidation lanes used to fold only when building the
// slate, after the publication rule, the repair turn and the floor had read
// the answer as the model wrote it. A line holding only a no-break space is no
// blank line to the page parser, so a two-paragraph answer passed against a
// one-paragraph page as one paragraph and shipped as two; a disputed wording
// written with U+2011 for its hyphen was not that wording until folded. The
// repair lane folds at its wire readers (`edit-wire.ts`, `refine-wire.ts`);
// this is the same rule for the two lanes that read `TranslateReportWire`.
//
// THREE INTAKES CALL IT: the translate gather (`translate-produce.ts`), the
// consolidation gather (`consolidate-produce.ts`) and an author's revision in
// the repair turn (`translate-repair.ts`). `buildTranslateCandidates` folds
// nothing and refuses a voice that arrives unfolded.

/**
 Voices with every translation folded, and what folding replaced.

 @example
 ```ts
 const intake: FoldedVoices = { voices: [], findings: [], };
 ```
 */
export type FoldedVoices = {
  /**
   Voices in the order given, each translation as it would ship.
   */
  readonly voices: readonly HeardVoice<TranslateReportWire>[];

  /**
   One finding per code point folded in a voice, naming the voice's model, in
   voice order; empty when nothing was folded.
   */
  readonly findings: readonly string[];
};

/**
 A voice reached the slate with a translation the fold would still change.

 A FAULT IN THIS CODE rather than a fact about any answer: every lane folds
 at intake, so an unfolded voice here means a new caller skipped the fold and
 every check before the slate read bytes that will not ship.

 @example
 ```ts
 throw new UnfoldedTranslationError({ modelIds: ['hf:cat/Cat-A',], },);
 ```
 */
export class UnfoldedTranslationError extends Error {
  /**
   Declares this message safe to forward: it is one fixed sentence; the
   models ride beside it as a field and never enter it.
   */
  readonly messageNamesOnly: true = true;

  /**
   Models whose translations arrived unfolded.
   */
  public readonly modelIds: readonly string[];

  /**
   Builds the failure naming the voices.

   @param modelIds - models whose translations arrived unfolded

   @example
   ```ts
   throw new UnfoldedTranslationError({ modelIds: ['hf:cat/Cat-A',], },);
   ```
   */
  public constructor({ modelIds, }: { readonly modelIds: readonly string[]; },) {
    super(
      'a translation reached the slate unfolded, so the checks before it read bytes that will not ship',
    );
    this.name = 'UnfoldedTranslationError';
    this.modelIds = modelIds;
  }
}

/**
 One translation as it would ship, and what folding it replaced.

 @example
 ```ts
 const folded: FoldedTranslation = { translation: 'non-binary', findings: [], };
 ```
 */
export type FoldedTranslation = {
  /**
   The translation with every invisible variant replaced.
   */
  readonly translation: string;

  /**
   One finding per code point folded, naming the model that wrote it.
   */
  readonly findings: readonly string[];
};

/**
 Folds one translation a model wrote, naming the model in each finding.

 @param modelId - model that wrote it, which each finding names

 @param translation - translation as the model wrote it

 @returns The translation as it would ship, with a finding per code point folded

 @example
 ```ts
 const revision = foldTranslation({ modelId: voice.modelId, translation, },);
 ```
 */
export function foldTranslation(
  {
    modelId,
    translation,
  }: {
    readonly modelId: string;
    readonly translation: string;
  },
): FoldedTranslation {
  /**
   The translation as it would ship, and one finding per code point folded.
   */
  const {
    text,
    findings,
  } = foldInvisibleVariants({ text: translation, },);
  return {
    translation: text,
    findings: findings.map(function named(finding,): string {
      return `${finding} (${modelId})`;
    },),
  };
}

/**
 Folds every voice's translation as it enters a lane.

 @param voices - answers as the models wrote them

 @returns Voices as they would ship, with a finding per code point folded

 @example
 ```ts
 const intake = foldTranslatorVoices({ voices: gather.voices, },);
 ```
 */
export function foldTranslatorVoices(
  { voices, }: { readonly voices: readonly HeardVoice<TranslateReportWire>[]; },
): FoldedVoices {
  /**
   Each voice beside its fold.
   */
  const folded = voices.map(function foldOne(voice,): {
    readonly voice: HeardVoice<TranslateReportWire>;
    readonly findings: readonly string[];
  } {
    /**
     The translation as the model wrote it.
     */
    const { translation: written, } = voice.value;

    /**
     The translation as it would ship.
     */
    const fold = foldTranslation({
      modelId: voice.modelId,
      translation: written,
    },);
    return {
      voice: (fold.translation === written)
        ? voice
        : {
          ...voice,
          value: {
            ...voice.value,
            translation: fold.translation,
          },
        },
      findings: fold.findings,
    };
  },);
  return {
    voices: folded.map(function voiceOf({ voice, },): HeardVoice<TranslateReportWire> {
      return voice;
    },),
    findings: folded.flatMap(function findingsOf({ findings, },): readonly string[] {
      return findings;
    },),
  };
}

/**
 Refuses voices whose translations the fold would still change.

 @param voices - voices about to become candidates

 @throws {@link UnfoldedTranslationError} when any translation is unfolded

 @example
 ```ts
 requireFoldedVoices({ voices, },);
 ```
 */
export function requireFoldedVoices(
  { voices, }: { readonly voices: readonly HeardVoice<TranslateReportWire>[]; },
): void {
  /**
   Models whose translations the fold would change.
   */
  const unfolded = voices
    .filter(function changesUnderFold(voice,): boolean {
      /**
       The translation as the voice carries it.
       */
      const { translation, } = voice.value;

      /**
       The same translation as it would ship.
       */
      const { text: folded, } = foldInvisibleVariants({ text: translation, },);
      return folded !== translation;
    },)
    .map(function modelOf(voice,): string {
      return voice.modelId;
    },);
  if (unfolded.length > 0)
    throw new UnfoldedTranslationError({ modelIds: unfolded, },);
}

//endregion Translator answer fold
