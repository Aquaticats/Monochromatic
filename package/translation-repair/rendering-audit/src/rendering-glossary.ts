import {
  type CommunityTerm,
  communityTermLines,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import { IDIOM_GLOSSARY, } from './rendering-glossary-idiom.ts';
import { MEDICAL_GLOSSARY, } from './rendering-glossary-medical.ts';
import { PHRASING_GLOSSARY, } from './rendering-glossary-phrasing.ts';
import { WORDING_GLOSSARY, } from './rendering-glossary-wording.ts';

//region Rendering glossary
// CLASS ONE HUNDRED TWENTY-THREE (shi_Yumiaoya23, 2026-09-25). The page
// shipped 师范学院 as "a normal college" and 觉醒了学霸属性 as "awakened her
// top-student trait": word-for-word renderings an English reader stumbles on,
// chosen by judges who had nothing telling them the ordinary English. The
// owner answered on 2026-09-25 that anything which can be translated better
// should be. This glossary holds ordinary Chinese words, not the community's
// own (`community-glossary.ts`), with the English the page uses and the
// calques a candidate may not write. It shares the community glossary's
// machinery: the words an entry's source carries reach every sheet's identity
// context (`renderingTermLines`), and the source-carry floor refuses a
// candidate that keeps one in Han or writes a refused form
// (`translate-source-carry.ts`). An entry is added whenever a read finds a
// rendering that can be better; an official name that carries a word (北京师范
// 大学, "Beijing Normal University") is never entered, only the generic word.

/**
 Slides onto a tier, each form 滑档 refuses (ledger R4): the verbs a run
 wrote for it, with or without "down", before "into" or "to" and a tier.
 Every variant the settled shi_Yumiaoya pages shipped is among them
 ("slipped down into the second tier", "slid down to a second-tier").
 */
const TIER_SLIDES: readonly string[] = [
  'slid',
  'slipped',
  'slide',
  'slides',
  'slip',
  'slips',
  'sliding',
  'slipping',
].flatMap(function slidesOf(verb,): readonly string[] {
  return [
    verb,
    `${verb} down`,
  ].flatMap(function linksOf(slide,): readonly string[] {
    return [
      'into',
      'to',
    ].flatMap(function tiersOf(link,): readonly string[] {
      return [
        'a second tier',
        'the second tier',
        'second tier',
        'a lower tier',
        'the lower tier',
        'lower tier',
        'a third tier',
        'the third tier',
        'third tier',
      ].map(function joined(tier,): string {
        return `${slide} ${link} ${tier}`;
      },);
    },);
  },);
},);

/**
 Ordinary words the pinned corpus carries whose word-for-word rendering reads
 badly in English, with the English the page uses.
 */
export const RENDERING_GLOSSARY: readonly CommunityTerm[] = [
  {
    term: '师范学院',
    renderings: [
      'teachers\' college',
      'teachers college',
      'teacher-training college',
    ],
    refusedForms: [
      'normal college',
    ],
    // LEDGER R8: the why once promised that "normal college" stays inside an
    // institution's official English name, which the floor never excepted.
    // The pin carries 师范学院 once, generically (shi_Yumiaoya); a page that
    // names an institution takes a `properNameContexts` entry, as 药娘 does.
    why: 'a college that trains teachers; "normal college" is the dated English name, which today\'s readers do '
      + 'not recognize',
  },
  {
    term: '师范学校',
    renderings: [
      'teacher-training school',
      'teachers\' school',
      'teachers\' college',
    ],
    refusedForms: [
      'normal school',
    ],
    why: 'a school that trains teachers; "normal school" is the dated English name, which today\'s readers do not '
      + 'recognize',
  },
  {
    term: '学霸',
    renderings: [
      'top student',
      'star student',
      'straight-A',
    ],
    refusedForms: [
      'top-student trait',
      'top student trait',
      'academic tyrant',
      'study tyrant',
      'xueba',
    ],
    // LEDGER R13: the why once taught one page's sentence (觉醒了学霸属性); the
    // stock-phrase lesson is the idiomatic English rule's, and the refused
    // forms keep its calque out.
    why: 'a student who excels academically; English says top student or star student, never "academic tyrant"',
  },
  // CLASS ONE HUNDRED TWENTY-FIVE (shi_Yumiaoya25, 2026-09-25): 滑档二本
  // shipped as "slid down into a second-tier admission slot". Each form
  // appears once in the pinned corpus. The class seeded 用这种方式 too; the
  // glossary audit of 2026-09-27 took it out, a construction rather than a
  // word, and the idiomatic English rule of `english-usage-policy.ts` states
  // its lesson.
  {
    term: '二本',
    renderings: [
      'second-tier university',
      'second-tier college',
      'second-tier school',
    ],
    refusedForms: [
      'admission slot',
      'second batch',
      'erben',
    ],
    why: 'the second tier of Chinese universities, admitted by college entrance examination score after the first '
      + 'tier (一本); the page names the kind of university, never the admission batch or slot',
  },
  {
    term: '滑档',
    renderings: [
      'missed her chosen schools',
      'fell through to',
      'ended up at',
    ],
    // LEDGER R4: the bare motion verbs ("slid into", "slid to") refused tears
    // that slid down a face and a mood slipped into, so each refused slide
    // now lands on a tier.
    refusedForms: [
      ...TIER_SLIDES,
      'slipped a file',
      'sliding file',
      'huadang',
    ],
    why: 'an applicant whose college entrance examination score met none of the schools applied to and who was placed '
      + 'at a lower tier; English says she missed her chosen schools and ended up at a lower-tier one, never that she '
      + 'slid or slipped to a tier',
  },
  // CLASS ONE HUNDRED TWENTY-SIX (shi_Yumiaoya27, 2026-09-25): 年级组长
  // shipped as "the grade leader", 未成年药娘 as "a minor trans girl", 骨灰骰子
  // as "ash dice", 同居者 as "cohabitants" and 精神霸凌 as plain "bullied".
  {
    term: '年级组长',
    renderings: [
      'grade coordinator',
      'head of the grade',
      'grade director',
    ],
    refusedForms: [
      'grade leader',
      'head of year',
      'year head',
      'grade group leader',
      'grade team leader',
    ],
    why: 'the teacher in charge of a whole grade; "grade leader" is a calque English readers do not know, and '
      + '"head of year" is British where a Canadian school says grade',
  },
  {
    term: '未成年',
    renderings: [
      'underage',
      'under eighteen',
      'a minor',
    ],
    refusedForms: [
      'minor trans',
      'minor transgender',
      'minor girl',
      'minor boy',
    ],
    why: 'under the age of majority; English says "underage" before a noun and "a minor" only as a noun, since "a '
      + 'minor trans girl" reads as an unimportant one',
  },
  {
    term: '骨灰骰子',
    renderings: [
      'dice made from the ashes',
      'memorial dice made from the ashes',
    ],
    refusedForms: [
      'ash dice',
      'ashes dice',
    ],
    why: 'dice made with a person\'s cremated ashes as a keepsake; "ash dice" reads as dice made of ash from anything',
  },
  {
    term: '同居者',
    renderings: [
      'housemate',
      'roommate',
    ],
    refusedForms: [
      'cohabitant',
      'cohabiter',
    ],
    why: 'someone she shared a home with; "cohabitant" says a romantic partner, which the original does not',
  },
  {
    term: '精神霸凌',
    renderings: [
      'psychologically bullied',
      'emotionally bullied',
      'psychological bullying',
    ],
    refusedForms: [
      'spiritually bullied',
      'spiritually bullying',
      'spiritually bully',
      'spiritual bullying',
      'spiritual bully',
      'spiritual bullies',
    ],
    why: 'bullying by words and pressure rather than by force; the page keeps the adjective ("psychologically '
      + 'bullied"), never "spiritual"',
  },
  // CLASS ONE HUNDRED TWENTY-SEVEN (owner, 2026-09-25: "overdosing", "sailor
  // uniform", "National College Entrance Examination"), answering the three
  // renderings the shi_Yumiaoya27 read left as written. OD once carried a
  // leading space so it would not match the MOD entry s5ehfr9 writes, on the
  // claim that every OD in the pinned corpus stands after one; that was false
  // (hulicaijia opens a paragraph with OD, XingZ60 writes a lowercase od four
  // times), and XingZ6010 and XingZ6014 shipped "I hate od". Word boundaries
  // (`glossary-match.ts`, class one hundred eighty-six) keep MOD out instead.
  {
    term: 'OD',
    renderings: [
      'overdosing',
      'overdose',
      'overdosed',
    ],
    refusedForms: [],
    why: 'the community\'s shorthand for taking medication far past the dose; the page spells it out, never "OD" '
      + 'or "ODing"',
  },
  {
    term: 'jk 裙',
    renderings: [
      'sailor uniform',
      'sailor-uniform skirt',
    ],
    refusedForms: [
      'jk skirt',
      'jk-skirt',
      'jk uniform',
      'jk-style',
      'jk dress',
    ],
    why: 'the Japanese schoolgirl uniform the community calls JK; the page says "sailor uniform", never "JK skirt"',
  },
  {
    term: '高考',
    renderings: [
      'National College Entrance Examination',
      'college entrance examination',
      'college entrance exam',
    ],
    refusedForms: [
      'gaokao',
    ],
    why: 'China\'s national university entrance examination; the page names it in English, never "Gaokao"',
  },
  // CLASS ONE HUNDRED TWENTY-NINE (shi_Yumiaoya30, 2026-09-25): phrasings,
  // kept in `rendering-glossary-phrasing.ts`.
  ...PHRASING_GLOSSARY,
  // CLASS ONE HUNDRED THIRTY (shi_Yumiaoya31, 2026-09-25): idioms and set
  // phrases, kept in `rendering-glossary-idiom.ts`.
  ...IDIOM_GLOSSARY,
  // CLASS ONE HUNDRED THIRTY-ONE (shi_Yumiaoya32, 2026-09-25): wording in
  // accounts of events, kept in `rendering-glossary-wording.ts`.
  ...WORDING_GLOSSARY,
  // CLASS ONE HUNDRED SIXTY-ONE (TianqiChen6662, 2026-09-26) seeded 化作 to
  // refuse the doubled preposition "turned into in"; ledger R5 (2026-09-28)
  // took it out, since the refusal also struck sound English ("what the
  // kitten turned into in spring"), and the grammatical English rule of
  // `english-usage-policy.ts` states the lesson on every sheet.
  // CLASS ONE HUNDRED SIXTY-TWO (TianqiChen6663, 2026-09-26): medical terms,
  // kept in `rendering-glossary-medical.ts`.
  ...MEDICAL_GLOSSARY,
  // CLASS ONE HUNDRED SIXTY-SEVEN (TianqiChen6665, 2026-09-26) seeded one
  // card-game line here; the glossary audit of 2026-09-27 took it out for the
  // game jargon rule of `english-usage-policy.ts`.
];

/**
 Identity-context lines for the ordinary words an entry's source carries,
 under a heading that does not call them the community's.

 @param text - whole original document

 @returns Heading and one line per word present, empty when none is

 @example
 ```ts
 const lines = renderingTermLines({ text: sourceText, },);
 ```
 */
export function renderingTermLines(
  { text, }: { readonly text: string; },
): readonly string[] {
  return communityTermLines({
    text,
    glossary: RENDERING_GLOSSARY,
    heading: 'RENDERINGS for ordinary words this entry carries: write the English meaning, not a word-for-word calque '
      + '(a rendering may inflect):',
  },);
}

/**
 Identity-context lines for both glossaries: the community's words an entry
 carries (the owner's decision of 2026-09-09), then the ordinary words whose
 calque reads badly.

 @param text - whole original document

 @returns Both glossaries' headed lines, each empty when the entry carries
 none of its words

 @example
 ```ts
 const lines = glossaryTermLines({ text: sourceText, },);
 ```
 */
export function glossaryTermLines(
  { text, }: { readonly text: string; },
): readonly string[] {
  return [
    ...communityTermLines({ text, },),
    ...renderingTermLines({ text, },),
  ];
}

//endregion Rendering glossary
