/**
 Test-only vocabulary union types.
 
 Not exported from the package root. Consumers must supply their own
 vocabulary; the library never bundles application-level subjects,
 nouns, or verbs.
 
 @module
 */

/**
 Test label vocabulary union.
 */
export type TestLabel = 'siteName' | 'noResults' | 'page';

/**
 Test subject vocabulary union.
 */
export type TestSubject = 'I' | 'you' | 'they' | 'who';

/**
 Test noun vocabulary union.
 */
export type TestNoun = 'cat' | 'message' | 'item';

/**
 Test verb vocabulary union.
 */
export type TestVerb = 'have' | 'see' | 'delete' | 'want' | 'save';
