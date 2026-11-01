/**
 Grammar AST shared across every locale renderer.
 
 Every node in this module is a tagged variant; impossible grammatical
 combinations are unrepresentable in TypeScript. Renderers dispatch on
 the `kind` discriminant and never inspect the absence of a field as
 a signal.
 
 @module
 */

import type {
  Capitalization,
  Tense,
  VerbFragmentForm,
} from './grammar-primitives.ts';

//region Subject references

/**
 Reference to a subject in a sentence.
 
 `subject.key` resolves through the locale's subject vocabulary so the
 renderer can read person/number/gender for verb agreement. `subject.externalName`
 carries opaque caller text such as a user-typed name; the library may
 position the text grammatically but never translates it.
 */
export type SubjectRef<S extends string,> =
  | {
    readonly kind: 'subject.key';
    readonly subject: S;
  }
    | {
      readonly kind: 'subject.externalName';
      readonly text: string;
    };

/**
 Possessor in a `noun.possessed` phrase.
 
 Same dichotomy as {@link SubjectRef}: a `possessor.subject` resolves
 through the locale's subject vocabulary so the renderer can read the
 possessive surface, while `possessor.externalName` is opaque text.
 */
export type Possessor<S extends string,> =
  | {
    readonly kind: 'possessor.subject';
    readonly subject: S;
  }
    | {
      readonly kind: 'possessor.externalName';
      readonly text: string;
    };

//endregion Subject references

//region External text leaves

/**
 Opaque caller-supplied text placed verbatim into the rendered output.
 
 Use for item titles, names, preformatted times, or any payload the
 library must not translate or parse. Escaping for HTML insertion remains
 the caller's responsibility; the library treats the text as a leaf token.
 */
export type ExternalText = {
  readonly kind: 'externalText';
  readonly text: string;
};

//endregion External text leaves

//region Noun phrases

/**
 Noun phrase variants.
 
 Article intent is carried by the variant, not inferred from the noun
 entry. `noun.bare` renders without any article; `noun.counted` carries
 a numeric `count`, triggers plural/classifier logic, and is rejected for
 nouns marked `countability: 'mass'`; `noun.definite` and `noun.indefinite`
 consult the noun's article table; `noun.possessed` pairs the noun with
 a possessor; `noun.externalText` is an opaque leaf.
 */
export type NounPhrase<S extends string, N extends string,> =
  | {
    readonly kind: 'noun.bare';
    readonly noun: N;
  }
    | {
      readonly kind: 'noun.counted';
      readonly count: number;
      readonly noun: N;
    }
    | {
      readonly kind: 'noun.definite';
      readonly noun: N;
    }
    | {
      readonly kind: 'noun.indefinite';
      readonly noun: N;
    }
    | {
      readonly kind: 'noun.possessed';
      readonly possessor: Possessor<S>;
      readonly noun: N;
    }
    | {
      readonly kind: 'noun.externalText';
      readonly text: string;
    };

//endregion Noun phrases

//region Adverbials

/**
 Adverbial relation attached to a verb phrase.
 
 `adverbial.location` covers `at/in/to/from` plus a place noun phrase;
 `adverbial.time` covers `at/before/after` plus a time noun phrase or
 opaque external text (preformatted timestamps).
 
 The relation is normalized; the locale renderer chooses how to surface
 it (English preposition vs. Chinese coverb vs. Catalan preposition).
 */
export type Adverbial<S extends string, N extends string,> =
  | {
    readonly kind: 'adverbial.location';
    readonly relation: 'at' | 'in' | 'to' | 'from';
    readonly place: NounPhrase<S, N>;
  }
    | {
      readonly kind: 'adverbial.time';
      readonly relation: 'at' | 'before' | 'after';
      readonly time: NounPhrase<S, N> | ExternalText;
    };

//endregion Adverbials

//region Verb phrases

/**
 Non-finite complement attached to a verb phrase.
 
 Currently only the infinitive complement is modeled; gerund and
 participle complements can be added as additional discriminants when
 a consumer needs them. Modeled as a discriminated union for
 forward-compatibility.
 */
export type NonFiniteComplement<S extends string, V extends string, N extends string,> = {
  readonly kind: 'complement.infinitive';
  readonly phrase: VerbPhrase<S, V, N>;
};

/**
 Verb phrase: a verb plus optional object, non-finite complement, and
 adverbials.
 
 The verb identifier is a consumer-supplied key; the locale's verb
 vocabulary resolves it to its finite/non-finite surface forms. The
 object is itself a noun phrase, allowing nested grammar without
 domain-specific message methods.
 */
export type VerbPhrase<S extends string, V extends string, N extends string,> = {
  readonly kind: 'verbPhrase';
  readonly verb: V;
  readonly object?: NounPhrase<S, N>;
  readonly complement?: NonFiniteComplement<S, V, N>;
  readonly adverbials?: readonly Adverbial<S, N>[];
};

//endregion Verb phrases

//region Sentences

/**
 Declarative sentence: subject + predicate.
 
 Tense defaults to `present` when omitted; the renderer applies the
 tense to the predicate verb using the locale's tense strategy.
 */
export type DeclarativeSentence<S extends string, V extends string, N extends string,> = {
  readonly kind: 'sentence.declarative';
  readonly subject: SubjectRef<S>;
  readonly predicate: VerbPhrase<S, V, N>;
  readonly tense?: Tense;
  readonly terminator?: '.';
};

/**
 Yes/no question: same shape as a declarative but rendered with
 do-support, copula inversion, or a question particle depending on
 the locale.
 */
export type YesNoQuestion<S extends string, V extends string, N extends string,> = {
  readonly kind: 'sentence.question.yesNo';
  readonly subject: SubjectRef<S>;
  readonly predicate: VerbPhrase<S, V, N>;
  readonly tense?: Tense;
  readonly terminator?: '?';
};

/**
 Imperative sentence: predicate only, no subject slot.
 
 The `subject?: never` field guarantees TypeScript rejects attempts to
 attach a subject to an imperative; the locale renderer uses the
 imperative surface of the verb entry.
 */
export type ImperativeSentence<S extends string, V extends string, N extends string,> = {
  readonly kind: 'sentence.imperative';
  readonly predicate: VerbPhrase<S, V, N>;
  readonly subject?: never;
  readonly terminator?: '.' | '!';
};

/**
 Wh-question variants, one per slot the wh-word can occupy.
 
 Splitting by slot lets English front the wh-word and lets Chinese
 keep the wh-word in the occupied slot; a single flat shape with a
 top-level `whWord` field would force the renderer to guess.
 
 - `wh.subject` asks who performs the predicate (`Who saw it?`).
 - `wh.object` asks what the verb acts on (`What do I see?`); the verb
   carries adverbials but not an object.
 - `wh.adverbial` asks where/when/why/how about a full subject+predicate.
 */
export type WhQuestion<S extends string, V extends string, N extends string,> =
  | {
    readonly kind: 'sentence.question.wh.subject';
    readonly wh: 'who';
    readonly predicate: VerbPhrase<S, V, N>;
    readonly tense?: Tense;
    readonly terminator?: '?';
  }
    | {
      readonly kind: 'sentence.question.wh.object';
      readonly wh: 'what';
      readonly subject: SubjectRef<S>;
      readonly verb: V;
      readonly adverbials?: readonly Adverbial<S, N>[];
      readonly tense?: Tense;
      readonly terminator?: '?';
    }
    | {
      readonly kind: 'sentence.question.wh.adverbial';
      readonly wh: 'where' | 'when' | 'why' | 'how';
      readonly subject: SubjectRef<S>;
      readonly predicate: VerbPhrase<S, V, N>;
      readonly tense?: Tense;
      readonly terminator?: '?';
    };

/**
 Full sentence: declarative, yes/no question, wh-question, or imperative.
 */
export type Sentence<S extends string, V extends string, N extends string,> =
  | DeclarativeSentence<S, V, N>
    | YesNoQuestion<S, V, N>
    | WhQuestion<S, V, N>
    | ImperativeSentence<S, V, N>;

//endregion Sentences

//region Fragments

/**
 Sub-sentence fragment part inside a `fragment.sequence`.
 
 Labels in this position must come from the consumer's `Label` union;
 the locale spec resolves the key against its label vocabulary.
 */
export type FragmentPart<Label extends string, S extends string, N extends string,> =
  | {
    readonly kind: 'part.label';
    readonly label: Label;
  }
    | {
      readonly kind: 'part.nounPhrase';
      readonly phrase: NounPhrase<S, N>;
    }
    | {
      readonly kind: 'part.externalText';
      readonly text: string;
    };

/**
 UI text that is not a complete sentence.
 
 Avoid faking subjectless declaratives just to use the sentence
 machinery: fragments render without a sentence terminator and without
 subject-verb agreement, so a noun-phrase fragment is just the noun
 phrase, and a verb-phrase fragment uses one of the {@link VerbFragmentForm}
 non-finite forms.
 
 `fragment.sequence` composes labels, noun phrases, and external text
 with a single trailing space between parts.
 */
export type Fragment<Label extends string, S extends string, V extends string,
  N extends string,> =
  | {
    readonly kind: 'fragment.nounPhrase';
    readonly phrase: NounPhrase<S, N>;
    readonly capitalization?: Capitalization;
  }
    | {
      readonly kind: 'fragment.verbPhrase';
      readonly phrase: VerbPhrase<S, V, N>;
      readonly form: VerbFragmentForm;
      readonly capitalization?: Capitalization;
    }
    | {
      readonly kind: 'fragment.sequence';
      readonly parts: readonly FragmentPart<Label, S, N>[];
      readonly capitalization?: Capitalization;
    };

//endregion Fragments
