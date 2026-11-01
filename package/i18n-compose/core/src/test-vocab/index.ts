/**
 Test-only vocabulary barrel.
 
 Re-exports the per-locale tables and union types used by `*.unit.test.ts`
 files in the package. Not exported from the package root.
 
 @module
 */

export type {
  TestLabel,
  TestNoun,
  TestSubject,
  TestVerb,
} from './types.ts';

export {
  enLabels,
  enNouns,
  enSubjects,
  enVerbs,
} from './en.ts';

export {
  zhLabels,
  zhNouns,
  zhSubjects,
  zhVerbs,
} from './zh.ts';

export {
  caLabels,
  caNouns,
  caSubjects,
  caVerbs,
} from './ca.ts';
