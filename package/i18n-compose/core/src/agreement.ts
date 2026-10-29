/**
 Subject-agreement metadata extraction shared by every locale.
 
 The renderer-internal verb-form helpers consume `{ person, number }`
 rather than a {@link SubjectRef}, so a wh-subject question can pass
 `{ person: 3, number: 'singular' }` without having to synthesize an
 unsafe `'who' as Subject` cast.
 
 @module
 */

import type { SubjectRef, } from './ast.ts';
import type { SubjectEntry, } from './entries.ts';
import type {
  GrammaticalNumber,
  Person,
} from './grammar-primitives.ts';

/**
 Person + number pair used by verb-agreement lookups.
 
 Decoupled from {@link SubjectRef} so wh-subject and other synthesized
 subjects can supply agreement explicitly without polluting the
 consumer's subject vocabulary.
 */
export type SubjectAgreement = {
  readonly person: Person;
  readonly number: GrammaticalNumber;
};

/**
 Agreement used for `subject.externalName` references; opaque names default to third-person singular.
 */
const EXTERNAL_NAME_AGREEMENT: SubjectAgreement = {
  person: 3,
  number: 'singular',
};

/**
 Resolves the agreement metadata for a subject reference.
 
 @param ref - subject reference from the AST
 
 @param subjects - locale's subject vocabulary table
 
 @returns person/number for verb agreement
 
 @example
 ```ts
 subjectAgreement({ ref: { kind: 'subject.key', subject: 'I' }, subjects });
 // { person: 1, number: 'singular' }
 ```
 */
export function subjectAgreement<S extends string,>(
  {
    ref,
    subjects,
  }: {
    readonly ref: SubjectRef<S>;
    readonly subjects: Readonly<Record<S, SubjectEntry>>;
  },
): SubjectAgreement {
  if (ref.kind
    === 'subject.key') {
    /**
     Resolved subject entry from the locale vocabulary.
     */
    const meta = subjects[ref.subject];
    return {
      person: meta.person,
      number: meta.number,
    };
  }
  return EXTERNAL_NAME_AGREEMENT;
}

/**
 Returns the rendered surface for a subject reference.
 
 @param ref - subject reference from the AST
 
 @param subjects - locale's subject vocabulary table
 
 @returns surface form to drop into the rendered sentence
 
 @example
 ```ts
 subjectSurface({ ref: { kind: 'subject.key', subject: 'I' }, subjects });
 // 'I'
 ```
 */
export function subjectSurface<S extends string,>(
  {
    ref,
    subjects,
  }: {
    readonly ref: SubjectRef<S>;
    readonly subjects: Readonly<Record<S, SubjectEntry>>;
  },
): string {
  return ref.kind
    === 'subject.key'
    ? subjects[ref.subject]
      .surface
    : ref.text;
}

/**
 Agreement constant used by wh-subject renderers: English `who`, Catalan `Qui`, Chinese `谁`.
 */
export const WH_SUBJECT_AGREEMENT: SubjectAgreement = {
  person: 3,
  number: 'singular',
};
