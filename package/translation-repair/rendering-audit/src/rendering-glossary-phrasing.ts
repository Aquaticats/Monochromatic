import type { CommunityTerm, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Phrasing renderings
// CLASS ONE HUNDRED TWENTY-NINE (shi_Yumiaoya30, 2026-09-25): 摆烂 shipped as
// "turned to one of giving up" and 万千世界 as "this myriad world". lxy writes
// 偶尔摆烂 for taking it easy. Kept beside `rendering-glossary.ts`, which
// spreads these entries into `RENDERING_GLOSSARY`, so neither file outgrows
// the line budget.
//
// THE GLOSSARY AUDIT OF 2026-09-27 took out the entries here that were keyed
// on one sentence's construction rather than on a word (原因是多方面的, 特例,
// 陷入癫狂, 环境的问题, kigurumi的记忆结束); their lessons are the idiomatic
// English, grammatical English and kept-subject rules of
// `english-usage-policy.ts`, on every sheet. 交往 stays as a word but refuses
// nothing, since "dated" is its English wherever the passage speaks of
// romance; the pin carries it four times, all on aiyysk, never of romance, so
// "dated" is not among its renderings (ledger R11).

/**
 Phrasings the pinned corpus carries whose word-for-word rendering reads
 badly in English, with the English the page uses.
 */
export const PHRASING_GLOSSARY: readonly CommunityTerm[] = [
  {
    term: '摆烂',
    // LEDGER R6: the why once prescribed one page's sentence ("she stopped
    // trying"), a mistranslation of lxy's 偶尔摆烂, taking it easy now and
    // then; the renderings now lead with the plain verb either way.
    renderings: [
      'gave up',
      'stopped trying',
      'took it easy',
      'let things slide',
    ],
    refusedForms: [
      'one of giving up',
      'state of giving up',
      'giving-up state',
      'slacked off',
      'slacking off',
      'slack off',
      'bailan',
    ],
    why: 'internet slang for no longer making an effort, or, said lightly, for taking it easy; English says she gave '
      + 'up, stopped trying or took it easy, as the passage means, never an attitude "of giving up" and never the '
      + 'English slang "slacked off"',
  },
  {
    term: '万千世界',
    renderings: [
      'this vast world',
      'the wide world',
      'the whole wide world',
    ],
    refusedForms: [
      'myriad world',
      'ten thousand worlds',
      'thousands of worlds',
    ],
    why: 'the world in all its variety; "myriad" before a singular noun is not English, so the page says "this vast world"',
  },
  // CLASS ONE HUNDRED FIFTY-EIGHT (aiyysk2, 2026-09-26): 没和 MTF 交往过 shipped
  // twice as "I'd never dated a trans woman", a romance the original never
  // states; the next paragraph calls it "my first time ever talking with a
  // trans woman" and the archive wrote "never actually talked to". The audit
  // of 2026-09-27 dropped the refused dating forms: 交往 does mean dating
  // where the passage speaks of romance, so the why carries the condition and
  // the judges read it.
  {
    term: '交往',
    renderings: [
      'talked to',
      'interacted with',
      'spent time with',
      'got to know',
    ],
    refusedForms: [],
    why: 'keeping company with people and getting to know them; it means dating only where the passage speaks of '
      + 'romance, so elsewhere English says "talked to" or "spent time with", and "dated" there invents a romance',
  },
  // CLASS ONE HUNDRED FIFTY-NINE (aiyysk2, 2026-09-26): 工程机 shipped
  // "engineering phone" three times (the archive too), where the phone world
  // says "prototype" or "engineering sample". The pinned corpus carries 工程机
  // three times, all on aiyysk.
  {
    term: '工程机',
    renderings: [
      'prototype phone',
      'engineering sample',
      'prototype',
    ],
    refusedForms: [
      'engineering phone',
      'engineering machine',
      'engineering device',
    ],
    why: 'a pre-release phone made for testing; the phone world calls it a "prototype" or "engineering sample", never an '
      + '"engineering phone"',
  },
  // CLASS ONE HUNDRED EIGHTY (TianqiChen66613, 2026-09-27): 亲友都没有忘记 shipped
  // as the archive's "close friends never forgot", dropping the family. The
  // pin carries 亲友 in two paragraphs on two entries, neither writing another
  // word for friends beside it, so "close friends" is refused outright.
  {
    term: '亲友',
    renderings: [
      'friends and family',
      'family and friends',
      'loved ones',
    ],
    refusedForms: ['close friends',],
    why: 'relatives and friends together; "close friends" drops the family the word names',
  },
];

//endregion Phrasing renderings
