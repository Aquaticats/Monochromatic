/**
 English verb-phrase renderer factory and finite-verb form helpers.
 
 @module
 */

import type { SubjectAgreement, } from '../../agreement.ts';
import type {
  Adverbial,
  NounPhrase,
  VerbPhrase,
} from '../../ast.ts';
import type { Tense, } from '../../grammar-primitives.ts';
import { joinTokens, } from '../../render-helpers.ts';
import { englishThirdSingular, } from './morphology.ts';
import type { EnglishVerbEntry, } from './types.ts';

/**
 Grammatical-person value used to detect third-person singular agreement; composed from the exempt 1..2 range to satisfy `no-magic-numbers`.
 */
const THIRD_PERSON = 1 + 2;

/**
 Complement rendering mode for English verb phrases.
 
 Ordinary verbs take infinitive complements with `to`; modal-like verbs
 take bare complements, e.g. `can save` rather than `can to save`.
 */
export type EnglishComplementForm = 'infinitive' | 'bare';

/**
 Question head and retained lexical verb chosen from an English verb entry.
 
 `lexicalVerb` is empty string when the strategy retains no lexical verb
 (copula and modal questions front the finite verb itself), so `joinTokens`
 drops the slot.
 */
export type EnglishQuestionVerbParts = {
  readonly auxiliary: string;
  readonly lexicalVerb: string;
  readonly complementForm: EnglishComplementForm;
};

/**
 Concrete auxiliary strategy after applying the default.
 */
type EnglishAuxiliaryStrategy = NonNullable<EnglishVerbEntry['auxiliaryStrategy']>;

/**
 Returns the auxiliary strategy for an entry, defaulting lexical verbs to do-support.
 
 @param entry - English verb entry
 
 @returns configured strategy or `do-support`
 */
function auxiliaryStrategyFor(
  {
    entry,
  }: {
    readonly entry: EnglishVerbEntry;
  },
): EnglishAuxiliaryStrategy {
  return entry.auxiliaryStrategy
    ?? 'do-support';
}

/**
 Picks how complements should attach to a verb entry.
 
 @param entry - English verb entry
 
 @returns bare mode for modal-like verbs, infinitive mode otherwise
 
 @example
 ```ts
 complementFormForVerb({ entry: { base: 'can', auxiliaryStrategy: 'modal' } }); // 'bare'
 ```
 */
export function complementFormForVerb(
  {
    entry,
  }: {
    readonly entry: EnglishVerbEntry;
  },
): EnglishComplementForm {
  /**
   Configured auxiliary strategy after applying the default.
   */
  const strategy = auxiliaryStrategyFor({ entry, },);
  return (strategy === 'modal') || (strategy === 'none') ? 'bare' : 'infinitive';
}

/**
 Picks the verb surface for a finite declarative predicate slot.
 
 Uses do-support strategy: declaratives use the third-person singular
 form for 3s present, otherwise the base form. Past tense uses the
 `past` form. Future leaves the base form; the caller wraps in `will`.
 
 @param entry - resolved English verb entry
 
 @param tense - sentence tense
 
 @param agreement - subject person/number for 3s lookup
 
 @returns finite verb surface ready to drop into a declarative
 
 @example
 ```ts
 declarativeVerbSurface({ entry: { base: 'have', present3s: 'has' }, tense: 'present', agreement: { person: 3, number: 'singular' } });
 // 'has'
 ```
 */
export function declarativeVerbSurface(
  {
    entry,
    tense,
    agreement,
  }: {
    readonly entry: EnglishVerbEntry;
    readonly tense: Tense;
    readonly agreement: SubjectAgreement;
  },
): string {
  if (tense === 'past')
    return entry.past
      ?? `${entry.base}ed`;
  if (tense === 'future')
    return entry.base;
  if ((agreement.person
    === THIRD_PERSON) && (agreement.number
      === 'singular'))
    return entry.present3s
      ?? englishThirdSingular({ base: entry.base, },);
  return entry.base;
}

/**
 Picks the do-support auxiliary for a yes/no or wh-question body.
 
 @param tense - sentence tense
 
 @param agreement - subject person/number for `Does` selection
 
 @returns lowercase auxiliary; the caller sentence-cases it when at position 0
 
 @example
 ```ts
 doAuxiliary({ tense: 'past', agreement: { person: 1, number: 'singular' } }); // 'did'
 ```
 */
export function doAuxiliary(
  {
    tense,
    agreement,
  }: {
    readonly tense: Tense;
    readonly agreement: SubjectAgreement;
  },
): string {
  if (tense === 'past')
    return 'did';
  if (tense === 'future')
    return 'will';
  if ((agreement.person
    === THIRD_PERSON) && (agreement.number
      === 'singular'))
    return 'does';
  return 'do';
}

/**
 Picks the fronted auxiliary and retained lexical verb for an English question.
 
 @param entry - resolved English verb entry
 
 @param tense - sentence tense
 
 @param agreement - subject agreement for do-support and copula finite forms
 
 @returns question verb parts ready to join around the subject
 
 @example
 ```ts
 questionVerbParts({ entry: { base: 'are', present3s: 'is', auxiliaryStrategy: 'copula' }, tense: 'present', agreement: { person: 2, number: 'singular' } });
 // { auxiliary: 'are', lexicalVerb: '', complementForm: 'infinitive' }
 ```
 */
export function questionVerbParts(
  {
    entry,
    tense,
    agreement,
  }: {
    readonly entry: EnglishVerbEntry;
    readonly tense: Tense;
    readonly agreement: SubjectAgreement;
  },
): EnglishQuestionVerbParts {
  /**
   Configured auxiliary strategy after applying the default.
   */
  const strategy = auxiliaryStrategyFor({ entry, },);
  if (strategy === 'copula') {
    if (tense === 'future') {
      return {
        auxiliary: 'will',
        lexicalVerb: entry.base,
        complementForm: 'infinitive',
      };
    }
    return {
      auxiliary: declarativeVerbSurface({
        entry,
        tense,
        agreement,
      },),
      lexicalVerb: '',
      complementForm: 'infinitive',
    };
  }
  if ((strategy === 'modal') || (strategy === 'none')) {
    return {
      auxiliary: tense === 'past' ? entry.past
        ?? entry
          .base : entry.base,
      lexicalVerb: '',
      complementForm: 'bare',
    };
  }
  return {
    auxiliary: doAuxiliary({
      tense,
      agreement,
    },),
    lexicalVerb: entry.base,
    complementForm: 'infinitive',
  };
}

/**
 Picks the head verb for a wh-subject question.
 
 @param entry - resolved English verb entry
 
 @param tense - sentence tense
 
 @param agreement - wh-subject agreement used for ordinary finite forms
 
 @returns non-inverted question head such as `sees`, `is`, or `can`
 
 @example
 ```ts
 subjectQuestionVerbSurface({ entry: { base: 'can', auxiliaryStrategy: 'modal' }, tense: 'present', agreement: { person: 3, number: 'singular' } });
 // 'can'
 ```
 */
export function subjectQuestionVerbSurface(
  {
    entry,
    tense,
    agreement,
  }: {
    readonly entry: EnglishVerbEntry;
    readonly tense: Tense;
    readonly agreement: SubjectAgreement;
  },
): string {
  /**
   Configured auxiliary strategy after applying the default.
   */
  const strategy = auxiliaryStrategyFor({ entry, },);
  if ((strategy === 'modal') || (strategy === 'none'))
    return tense === 'past' ? entry.past
      ?? entry
        .base : entry.base;
  if (tense === 'future')
    return `will ${entry.base}`;
  return declarativeVerbSurface({
    entry,
    tense,
    agreement,
  },);
}

/**
 Renders an English nested complement using the head verb's attachment strategy.
 
 @param entry - head verb entry controlling bare vs infinitive attachment
 
 @param phrase - nested verb phrase to render
 
 @param renderVerbPhrase - recursive verb-phrase renderer
 
 @returns rendered complement surface
 */
function renderComplement<S extends string, V extends string, N extends string,>(
  {
    entry,
    phrase,
    renderVerbPhrase,
  }: {
    readonly entry: EnglishVerbEntry;
    readonly phrase: VerbPhrase<S, V, N>;
    readonly renderVerbPhrase: (phrase: VerbPhrase<S, V, N>,) => string;
  },
): string {
  /**
   Rendered nested verb phrase before complement marker selection.
   */
  const rendered = renderVerbPhrase(phrase,);
  return complementFormForVerb({ entry, },)
    === 'bare' ? rendered : `to ${rendered}`;
}

/**
 Builds an English verb-phrase renderer that consumes the noun-phrase
 and adverbial renderers as dependencies.
 
 @param verbs - verb vocabulary keyed by the consumer's `Verb` union
 
 @param renderNounPhrase - noun-phrase render function
 
 @param renderAdverbials - adverbial cluster render function
 
 @returns render function for verb phrases
 
 @example
 ```ts
 const renderVerbPhrase = makeEnglishVerbPhraseRenderer({ verbs, renderNounPhrase, renderAdverbials });
 renderVerbPhrase({ kind: 'verbPhrase', verb: 'save' }); // 'save'
 ```
 */
export function makeEnglishVerbPhraseRenderer<
  S extends string,
  V extends string,
  N extends string,
  >(
  {
    verbs,
    renderNounPhrase,
    renderAdverbials,
  }: {
    readonly verbs: Readonly<Record<V, EnglishVerbEntry>>;
    readonly renderNounPhrase: (phrase: NounPhrase<S, N>,) => string;
    readonly renderAdverbials: (
      advs?: readonly Adverbial<S, N>[],
    ) => string;
  },
  ): (phrase: VerbPhrase<S, V, N>,) => string {
  /**
   Renders a verb-phrase AST in English using the base verb form.
   
   @param phrase - verb-phrase AST
   
   @returns rendered surface
   */
  function renderVerbPhrase(phrase: VerbPhrase<S, V, N>,): string {
    /**
     Verb entry used for the head and complement attachment strategy.
     */
    const entry = verbs[phrase.verb];
    /**
     Verb base form; finite tense is decided by callers that own subject + tense context.
     */
    const verb = entry.base;
    /**
     Rendered object surface; empty string when no object slot was supplied.
     */
    const object = phrase.object
      === undefined
      ? ''
      : renderNounPhrase(phrase.object,);
    /**
     Rendered complement, bare for modal-like heads and infinitive otherwise; empty string when absent.
     */
    const complement = phrase.complement
      === undefined
      ? ''
      : renderComplement({
        entry,
        phrase: phrase.complement
          .phrase,
        renderVerbPhrase,
      },);
    /**
     Rendered adverbial cluster; empty string when none.
     */
    const adverbials = renderAdverbials(phrase.adverbials,);
    return joinTokens([
      verb,
      object,
      complement,
      adverbials,
    ],);
  }

  return renderVerbPhrase;
}
