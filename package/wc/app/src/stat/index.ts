/**
 Public entry point for the text-statistics module: {@link analyzeText}
 and {@link computeFrequency}, plus {@link splitWords} (the tokenizer
 {@link computeFrequency} consumes) and their result types.
 */
export { analyzeText, } from './analyze.ts';
export { computeFrequency, } from './frequency.ts';
export { splitWords, } from './tokenize.ts';
export type {
  FrequencyEntry,
  TextStats,
} from './types.ts';
