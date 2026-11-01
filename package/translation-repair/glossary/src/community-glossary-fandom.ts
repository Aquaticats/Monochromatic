import type { CommunityTerm, } from './community-glossary.ts';

//region Fandom glossary
// CLASS ONE HUNDRED SIXTY (TianqiChen6662, 2026-09-26): a kigurumi performer's
// page shipped 头壳 (the performer's head mask) as "inside her head", the
// wearer's own head, and 阿洛娜 and 亚托莉 as the archive's "Alona and
// Atori"; no candidate on the run wrote the characters' official names, Arona
// of Blue Archive and Atri of ATRI -My Dear Moments-. The archive renders 头壳
// "headpiece" on the same page, so that rendering leads. Kept beside
// `community-glossary.ts`, which spreads these entries into
// `COMMUNITY_GLOSSARY`, so neither file outgrows the line budget. Refused forms
// match in any case at word boundaries (`glossary-match.ts`), and each applies
// only where the source carries its term.

/**
 Fandom words and character names the pinned corpus carries, with the English
 the fandom uses.
 */
export const FANDOM_GLOSSARY: readonly CommunityTerm[] = [
  {
    term: '头壳',
    renderings: [
      'headpiece',
      'kigurumi head',
      'head mask',
      'mask',
    ],
    refusedForms: [
      'inside her head',
      'inside his head',
      'inside their head',
    ],
    // LEDGER C5: the floor refuses "inside her head", the form
    // TianqiChen6662 shipped; a bare "her head" stays with the judges, since
    // a slice that names the head mask can name the wearer's head too ("put
    // the headpiece on her head").
    why: 'a kigurumi performer\'s full head mask; the archive renders it "headpiece", and "her head" or "their head" '
      + 'for it reads as the wearer\'s own head',
  },
  {
    term: '阿洛娜',
    renderings: ['Arona',],
    refusedForms: ['alona',],
    why: 'Arona, the guide character of the game Blue Archive; the official English name, never a transliteration',
  },
  {
    term: '亚托莉',
    renderings: ['Atri',],
    refusedForms: ['atori',],
    why: 'Atri, the robot heroine of ATRI -My Dear Moments-; the official English name, never a transliteration',
  },
  // CLASS ONE HUNDRED SEVENTY-EIGHT (TianqiChen66610, 2026-09-26): the archive
  // glossed the performer's "high-performance robot" image as "a cute
  // character she cosplayed as"; the owner named the character, Atri, whose
  // catchphrase the phrase is.
  {
    term: '高性能机器人',
    renderings: ['high-performance robot',],
    refusedForms: [],
    why: 'Atri\'s own description of herself in ATRI -My Dear Moments- (her catchphrase "Because I\'m '
      + 'high-performance!"); a performer remembered as the "high-performance robot" is remembered as her Atri, so '
      + 'a note naming Atri is the reference, not an addition',
  },
  // CLASS ONE HUNDRED EIGHTY-FOUR (TianqiChen66616, 2026-09-27): 变娃娃 shipped
  // as "in this game of becoming a doll" and 被她治愈 as "those she had
  // healed". Kigurumi players call putting on the costume 变娃 (a Chinese
  // report on the hobby: 偶装玩家会将穿上偶装称作"变娃"); English "doll up"
  // means dressing smartly, so it is refused, every form of it since ledger
  // C5 (the why named "doll up" while the floor refused only "dolled up").
  // 治愈 first led with "comforted"; the owner wrote "I kinda disagree here"
  // (2026-09-27) and gave no reason, so it leads with "healed" and refuses
  // nothing. The reasons this comment once gave as the owner's were the
  // change's own: "healing" is the fandom's English for 治愈系, and the archive
  // reads "those she has healed" (ledger H3). The entry, its guard and the
  // handover also told the owner that the same page writes 安慰 where it means
  // comfort; it writes 安抚, never 安慰 (ledger C6), so no contrast with 安慰
  // stands.
  {
    term: '变娃',
    // A MULTI-WORD RENDERING INFLECTS INSIDE, where the ending the matcher
    // reads never reaches (ledger C3: "becoming the doll" named a departure),
    // so each verb form is its own rendering.
    renderings: [
      'put on the kigurumi',
      'puts on the kigurumi',
      'putting on the kigurumi',
      'in kigurumi',
      'became the doll',
      'become the doll',
      'becoming the doll',
    ],
    refusedForms: [
      'doll up',
      'dolls up',
      'dolled up',
      'dolling up',
    ],
    // LEDGER C6: the why once said the character "then stays silent", which
    // the doll's own quoted speech on the page contradicts.
    why: 'kigurumi players\' word for putting on the costume (head, bodysuit) and becoming the character; not a '
      + 'game, and never "doll up", which in English means dressing smartly',
  },
  {
    term: '治愈',
    // Each rendering inflects (ledger C3: "to cure" and "Healing views" named
    // departures), so "heal", "soothe" and "cure" carry every form; "healed"
    // leads by the owner's ruling. LEDGER C6: the why once named one page's
    // sentence ("the people she healed") on the three pages that carry 治愈.
    renderings: [
      'healed',
      'healing',
      'heal',
      'soothe',
      'cure',
    ],
    refusedForms: [],
    why: 'of people or feelings, the healing a person, a place or a work gives (治愈系, "healing" in the fan sense), '
      + 'so English says it healed or soothed them rather than flattening it into "comforted"; "cured" only where '
      + 'the passage speaks of an illness',
  },
];

//endregion Fandom glossary
