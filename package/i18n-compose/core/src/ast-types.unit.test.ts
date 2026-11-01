/**
 Type-level tests asserting that the AST rejects impossible grammatical
 states. These rely on `expectTypeOf` from the test harness, which
 re-exports `expect-type`.
 
 @module
 */

import {
  describe,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  DeclarativeSentence,
  ImperativeSentence,
  NounPhrase,
  Sentence,
  WhQuestion,
  YesNoQuestion,
} from './ast.ts';

/** Test subject union. */
type S = 'I' | 'you';

/** Test verb union. */
type V = 'see' | 'have';

/** Test noun union. */
type N = 'cat' | 'item';

await describe({
  name: 'ast-types',
  children: [
    //region Imperatives reject subjects

    it({
      name: 'imperative sentence forbids a subject slot at the type level',
      fn: async () => {
        /** Imperative literal; trying to assign a non-undefined subject must fail to type-check. */
        const ok: ImperativeSentence<S, V, N> = {
          kind: 'sentence.imperative',
          predicate: { kind: 'verbPhrase', verb: 'see', },
        };
        expectTypeOf(ok,).toHaveProperty('predicate',);
        /** Subject field is `never` for imperatives. */
        expectTypeOf<ImperativeSentence<S, V, N>['subject']>().toEqualTypeOf<undefined>();
      },
    },),

    //endregion Imperatives reject subjects

    //region Wh-object disallows a top-level object

    it({
      name:
        'wh.object variant has no `object` field; objects must use the noun phrase slot on a vp',
      fn: async () => {
        type WhObject = Extract<WhQuestion<S, V, N>,
          { kind: 'sentence.question.wh.object'; }>;
        /** The variant lacks an `object` field entirely; accessing it must be `never`/unknown. */
        expectTypeOf<keyof WhObject>().toEqualTypeOf<
          'kind' | 'wh' | 'subject' | 'verb' | 'adverbials' | 'tense' | 'terminator'
          >();
      },
    },),

    //endregion Wh-object disallows a top-level object

    //region Yes/no requires a subject

    it({
      name: 'yes/no question requires a subject field',
      fn: async () => {
        type YN = YesNoQuestion<S, V, N>;
        /** Subject is required (not optional). */
        expectTypeOf<YN['subject']>().not.toEqualTypeOf<undefined>();
      },
    },),

    //endregion Yes/no requires a subject

    //region Declarative requires a subject

    it({
      name: 'declarative sentence requires a subject field',
      fn: async () => {
        type D = DeclarativeSentence<S, V, N>;
        expectTypeOf<D['subject']>().not.toEqualTypeOf<undefined>();
      },
    },),

    //endregion Declarative requires a subject

    //region Noun-counted carries count, bare does not

    it({
      name: 'noun.counted variant carries count; noun.bare does not',
      fn: async () => {
        type Counted = Extract<NounPhrase<S, N>, { kind: 'noun.counted'; }>;
        type Bare = Extract<NounPhrase<S, N>, { kind: 'noun.bare'; }>;
        expectTypeOf<Counted['count']>().toBeNumber();
        expectTypeOf<keyof Bare>().toEqualTypeOf<'kind' | 'noun'>();
      },
    },),

    //endregion Noun-counted carries count, bare does not

    //region Sentence union covers all four kinds

    it({
      name: 'Sentence union covers declarative, yes/no, wh, and imperative',
      fn: async () => {
        type Kinds = Sentence<S, V, N>['kind'];
        expectTypeOf<Kinds>().toEqualTypeOf<
          | 'sentence.declarative'
            | 'sentence.question.yesNo'
            | 'sentence.imperative'
            | 'sentence.question.wh.subject'
            | 'sentence.question.wh.object'
            | 'sentence.question.wh.adverbial'
          >();
      },
    },),
    //endregion Sentence union covers all four kinds
  ],
},);
