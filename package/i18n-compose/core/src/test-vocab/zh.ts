/**
 Test-only Chinese vocabulary tables.
 
 @module
 */

import type {
  NounEntry,
  SubjectEntry,
} from '../entries.ts';
import type { ChineseVerbEntry, } from '../locale/zh/index.ts';
import type {
  TestLabel,
  TestNoun,
  TestSubject,
  TestVerb,
} from './types.ts';

/**
 Chinese label table for tests.
 */
export const zhLabels: Record<TestLabel, string> = {
  siteName: 'Aquaticat',
  noResults: '无结果',
  page: '页面',
};

/**
 Chinese subject table for tests.
 */
export const zhSubjects: Record<TestSubject, SubjectEntry> = {
  I: {
    surface: '我',
    possessive: '我的',
    person: 1,
    number: 'singular',
  },
  you: {
    surface: '你',
    possessive: '你的',
    person: 2,
    number: 'singular',
  },
  they: {
    surface: '他们',
    possessive: '他们的',
    person: 3,
    number: 'plural',
  },
  who: {
    surface: '谁',
    possessive: '谁的',
    person: 3,
    number: 'singular',
  },
};

/**
 Chinese noun table for tests.
 */
export const zhNouns: Record<TestNoun, NounEntry> = {
  cat: {
    surface: '猫',
    classifier: '只',
  },
  message: {
    surface: '消息',
    classifier: '条',
  },
  item: {
    surface: '物品',
    classifier: '件',
  },
};

/**
 Chinese verb table for tests.
 */
export const zhVerbs: Record<TestVerb, ChineseVerbEntry> = {
  have: { surface: '有', },
  see: { surface: '看见', },
  delete: { surface: '删除', },
  want: { surface: '想要', },
  save: { surface: '保存', },
};
