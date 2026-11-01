import type { CommunityTerm, } from './community-glossary.ts';

//region Community words
// The community's own words, kept beside `community-glossary.ts`, which
// spreads them with the fandom's (`community-glossary-fandom.ts`) into
// `COMMUNITY_GLOSSARY`, so no file outgrows the line budget. Split out on
// 2026-09-28 when the glossary audit's entries (ledger C3 to C10) would have
// taken `community-glossary.ts` past it.
//
// A WHY IS SHOWN ON EVERY PAGE THAT CARRIES THE TERM, so it states what holds
// on each of them: ledger C6 found 炸柜's why describing one page's passage,
// false on the other, and 跨圈's crediting "the archive" with a rendering
// only one of its three archives writes.

/**
 Adjectives the father's insult 逆子 takes in English.
 */
const UNFILIAL_WORDS = [
  'unfilial',
  'undutiful',
  'disobedient',
  'ungrateful',
  'rebellious',
] as const;

/**
 Nouns that take the son out of 逆子 (ledger C5: "rebellious child" shipped on
 shi_Yumiaoya36 and 37 and passed the floor).
 */
const UNGENDERED_NOUNS = [
  'child',
  'kid',
] as const;

/**
 Forms both spellings of the trans community refuse: crossdressing, which is
 another word (女装) naming another thing, and "across communities", the
 reading shi_Yumiaoya36 shipped (ledger C5).
 */
const TRANS_COMMUNITY_REFUSALS: readonly string[] = [
  ...[
    'crossdressing',
    'cross-dressing',
    'crossdresser',
    'cross-dresser',
  ].flatMap(function withNoun(crossdressing,): readonly string[] {
    return [
      'community',
      'communities',
      'circle',
    ].map(function joined(noun,): string {
      return `${crossdressing} ${noun}`;
    },);
  },),
  'across communities',
  'across different communities',
];

/**
 Renderings both spellings of the trans community take; "trans circle"
 inflects to "trans circles" (ledger C10: both stood, one redundant).
 */
const TRANS_COMMUNITY_RENDERINGS = [
  'trans community',
  'transgender community',
  'trans circle',
] as const;

/**
 Renderings every spelling of the outing takes.
 */
const OUTED_RENDERINGS = [
  'outed',
  'blown out of the closet',
] as const;

/**
 Community words the pinned corpus carries, curated by the owner.
 */
export const COMMUNITY_WORD_GLOSSARY: readonly CommunityTerm[] = [
  {
    term: '自切',
    renderings: ['self-surgery',],
    refusedForms: [],
    why: 'the community\'s word for gender-affirming surgery performed on oneself; the archive renders it '
      + '"attempted self-surgery", and "self-harm" or "cutting" misreads it',
    // LEDGER C10: each of these writes 自切's two characters inside another
    // word (each cutting, cutting in person, cutting alone).
    enclosingWords: [
      '各自切',
      '亲自切',
      '独自切',
    ],
  },
  {
    term: '超天酱',
    renderings: [
      'KAngel',
      'Needy Streamer Overload',
    ],
    refusedForms: [],
    why: 'the community\'s nickname for KAngel, the streamer character of the game Needy Streamer Overload; '
      + 'the archive names the character or the game, never a transliteration',
  },
  {
    // CLASS SEVENTY-TWO (mikaela_khara, 2026-09-19). The archive rendered 炸柜
    // as "tried coming out", one pass shipped "got blown out of the closet"
    // and the next "came out", a judge calling the community reading risky
    // and the literal one a display cabinet; nothing on any sheet said which.
    term: '炸柜',
    renderings: OUTED_RENDERINGS,
    refusedForms: [],
    why: 'the community\'s word for being outed against one\'s will, the closet blowing up, not for coming out; '
      + '"came out" or "tried coming out" reads the outing as her choice',
  },
  {
    // LEDGER C7 (the glossary audit of 2026-09-27): 炸柜's other spelling, on
    // Anilovr, had no entry; its archive renders it "outed".
    term: '爆柜',
    renderings: OUTED_RENDERINGS,
    refusedForms: [],
    why: '炸柜 in another spelling: being outed against one\'s will, the closet blowing up, not coming out; '
      + '"came out" reads the outing as her choice',
  },
  {
    // CLASS ONE HUNDRED EIGHTY-TWO (TianqiChen66614, 2026-09-27). 炸柜
    // written out, the closet door blown open; the 炸柜 entry never matched
    // it. The page shipped "blocked again and again, each time the closet
    // door blew open" for 因为柜门炸开屡屡受阻, keeping the figure literal and
    // reading the cause as a repeated event. One page in the pin carries it.
    term: '柜门炸开',
    renderings: OUTED_RENDERINGS,
    refusedForms: [],
    why: '炸柜 written out, the closet door blown open: being outed against one\'s will, not coming out; the figure '
      + 'is the closet\'s, so English says she was outed or blown out of the closet, never that a door blew open',
  },
  {
    // CLASS ONE HUNDRED NINETEEN (shi_Yumiaoya19, 2026-09-24). 小药娘 and
    // 药娘 shipped in Han, the judges following the archive translator's
    // comment that the word needs no translation, where two earlier runs
    // wrote "little HRT girl" and "little yaoniang". The owner answered on
    // 2026-09-24: the word is disrespectful, used neutrally by only some of
    // the community, and that neutrality does not carry into English, so
    // the page says "trans woman" or "trans girl". The owner added the same
    // day that the term is not kept even where the existing translation keeps
    // it, because the term itself can read as derogatory. 小药娘 carries
    // 药娘, so one entry covers both. Ledger C5: "HRT girl", which
    // shi_Yumiaoya17 shipped, and the other word-for-word forms passed the
    // floor until 2026-09-28.
    term: '药娘',
    renderings: [
      'trans girl',
      'trans woman',
      'trans women',
      'transgender girl',
      'transgender woman',
      'transgender women',
    ],
    refusedForms: [
      'yaoniang',
      'yao niang',
      'xiaoyaoniang',
      'xiao yaoniang',
      'xiao yao niang',
      'hrt girl',
      'medicine girl',
      'medication girl',
      'hormone girl',
      'pill girl',
      'drug girl',
    ],
    why: 'a disrespectful word for trans women on hormone therapy that some of the community use neutrally; '
      + 'the term itself can read as derogatory and the neutrality does not carry into English, so the page '
      + 'says "trans girl" or "trans woman", never the Han, a pinyin form or a word-for-word "HRT girl" or '
      + '"medicine girl", even where the existing translation or a translator\'s note on the page keeps it',
    // OWNER, 2026-09-27: mikaela_khara's registered company carries the word
    // in its name, and a company's proper name keeps its own form.
    properNameContexts: [
      '小药娘网络科技',
      '以小药娘做字号',
    ],
  },
  {
    // CLASS ONE HUNDRED FIFTY-ONE (shi_Yumiaoya36 and 37, 2026-09-26). The
    // father's insult 「逆子」 shipped as "rebellious child" on both runs and
    // in Han on shi_Yumiaoya8. The word is "unfilial son": on a trans
    // woman's memorial it is her father calling her his son, and "child"
    // takes the misgendering out of the insult the page reports. No archive
    // renders the passage (shi_Yumiaoya's archive is partial), so nothing on
    // any sheet said which. Ledger C5: the why refused "child" and the floor
    // did not, and the replay of 2026-09-28 found "unfilial child",
    // "ungrateful child" or "rebellious child" on thirteen settled
    // shi_Yumiaoya pages.
    term: '逆子',
    renderings: UNFILIAL_WORDS.map(function son(adjective,): string {
      return `${adjective} son`;
    },),
    refusedForms: UNFILIAL_WORDS.flatMap(function ungendered(adjective,): readonly string[] {
      return UNGENDERED_NOUNS.map(function joined(noun,): string {
        return `${adjective} ${noun}`;
      },);
    },),
    why: 'a parent\'s insult, "unfilial son"; said by a father of his trans daughter it calls her his son, '
      + 'so the page keeps "son" inside the quoted insult: "child" or "kid" drops the misgendering the '
      + 'insult carries, and the Han is never left',
  },
  {
    // CLASS ONE HUNDRED FIFTY-FOUR (shi_Yumiaoya38, 2026-09-26). 跨圈 appears
    // four times on the page; three shipped "the trans community" and one
    // "the crossdressing community", where shi_Yumiaoya37 had read it "across
    // different communities". The word is short for 跨性别圈子; shihai4h's
    // archive renders it "the Trans Community" and XingZ60's "the
    // transgender community". Crossdressing is another word (女装) and names
    // another thing, so writing it for 跨圈 tells the reader the page's trans
    // women are crossdressers.
    term: '跨圈',
    renderings: TRANS_COMMUNITY_RENDERINGS,
    refusedForms: TRANS_COMMUNITY_REFUSALS,
    why: 'short for 跨性别圈子, the trans community, as the archives that render it write; it never means '
      + 'crossdressing (女装) or "across communities"',
  },
  {
    // LEDGER C7 (the glossary audit of 2026-09-27): the word written out, on
    // GLaDOSister, had no entry; its archive renders it "the trans
    // community". 跨圈 never matched it, since 性别 stands between.
    term: '跨性别圈',
    renderings: TRANS_COMMUNITY_RENDERINGS,
    refusedForms: TRANS_COMMUNITY_REFUSALS,
    why: 'the trans community (跨圈 written out), as the archive writes; it never means crossdressing (女装) or '
      + '"across communities"',
  },
];

//endregion Community words
