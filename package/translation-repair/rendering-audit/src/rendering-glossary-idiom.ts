import type { CommunityTerm, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Idiom renderings
// CLASS ONE HUNDRED THIRTY (shi_Yumiaoya31, 2026-09-25): 三剑客汽车节目
// shipped as "the Three Musketeers car show", 燃油车 as "a fuel-powered car",
// 喘不过气 as "left her breathless", 密密麻麻的伤痕 as "densely packed scars",
// 志愿填写 as "her application preferences", 命运的齿轮 as "the gears of fate",
// 相关医院 as "the relevant hospitals", 巨大的影响 as "an enormous influence on
// her death" and 贴贴计划 as "her cuddling plan". Every term appears in the
// pinned corpus on that entry alone. Kept beside `rendering-glossary.ts`, which
// spreads these entries into `RENDERING_GLOSSARY`, so neither file outgrows the
// line budget.
//
// THE GLOSSARY AUDIT OF 2026-09-27 took out 相关医院 and 巨大的影响, two
// ordinary words joined by one sentence (the lesson, a filler word English
// does without, is the idiomatic English rule of `english-usage-policy.ts`),
// and dropped the refused forms of 三剑客, 喘不过气 and 密密麻麻 that are the
// right English for the same word elsewhere: any famous trio is "the Three
// Musketeers", a runner is "out of breath", writing is "densely packed".
/**
 Idioms and set phrases the pinned corpus carries whose word-for-word
 rendering reads badly in English, with the English the page uses.
 */
export const IDIOM_GLOSSARY: readonly CommunityTerm[] = [
  {
    term: '三剑客',
    // LEDGER R13: "the Top Gear trio" was one page's rendering among the
    // word's; "trio" carries it.
    renderings: [
      'trio',
      'the Three Musketeers',
    ],
    refusedForms: [
      'three swordsmen',
      'sanjianke',
    ],
    why: 'a famous trio, after the Three Musketeers; English names a trio by its own name where it has one (the '
      + 'presenters of Top Gear are "the Top Gear trio") and otherwise says "trio" or "the Three Musketeers"',
  },
  {
    term: '燃油车',
    // LEDGER R2: "fossil-fuel car" is sound English for the car being phased
    // out, and the refused "fuel car" stands inside it; as a rendering it
    // excuses what it overlaps.
    renderings: [
      'gas-powered car',
      'gasoline car',
      'gas car',
      'fossil-fuel car',
      'fossil-fuel vehicle',
    ],
    refusedForms: [
      'fuel-powered car',
      'fuel powered car',
      'fuel car',
      'fuel vehicle',
      'fuel-burning car',
      // Class one hundred thirty-two: "petrol" is British; the page is Canadian.
      'petrol car',
      'petrol-powered',
    ],
    why: 'a car with a gasoline engine, the kind being phased out; Canadian English says "a gas-powered car" or "a '
      + 'fossil-fuel car", never "a fuel-powered car" or the British "petrol car"',
  },
  {
    term: '喘不过气',
    renderings: [
      'could barely breathe',
      'suffocating',
      'overwhelmed',
      'out of breath',
    ],
    refusedForms: [],
    why: 'unable to catch one\'s breath; said of pressure or grief it means smothered, and English says the pressure '
      + 'was suffocating or she could barely breathe under it, since "breathless" reads as excited; said of running it '
      + 'is "out of breath"',
  },
  {
    term: '密密麻麻',
    renderings: [
      'covered in',
      'a mass of',
      'thick with',
      'densely packed',
    ],
    refusedForms: [],
    why: 'so many that they crowd together; English takes the noun\'s own idiom, an arm "covered in" scars or a page '
      + '"densely packed" with writing, never "densely packed scars"',
  },
  {
    term: '志愿填写',
    renderings: [
      'university applications',
      'choice of universities',
      'university choices',
    ],
    refusedForms: [
      'application preferences',
      'preference form',
      'volunteer form',
      'wish form',
    ],
    // LEDGER R9 and R13: the why once wrote "gaokao", which 高考 refuses, and
    // named one page's mistake on the applications.
    why: 'listing the universities a college entrance examination candidate applies to; English says university '
      + 'applications or choices, never "application preferences"',
  },
  {
    term: '命运的齿轮',
    renderings: [
      'wheels of fate',
      'wheel of fate',
    ],
    refusedForms: [
      'gears of fate',
      'gear of fate',
      'cogs of fate',
      'gears of destiny',
    ],
    why: 'the moment a life\'s course is set; the English idiom is "the wheels of fate began to turn", never "gears"',
  },
  {
    term: '贴贴计划',
    renderings: [
      'cuddle meetups',
      'cuddle tour',
      'meetups to hug',
    ],
    refusedForms: [
      'cuddling plan',
      'cuddle plan',
      'sticking plan',
      'tietie',
    ],
    why: 'her project of meeting community friends across the country for hugs; the page names the meetups, never a '
      + '"cuddling plan"',
  },
  // CLASS ONE HUNDRED FIFTY-SEVEN (aiyysk1, 2026-09-26): 樱奈以生命相逼 shipped as
  // "Sakurana threatened her life", which reads as a threat against the other
  // girl; the friend staked her own life ("If I find out, I'll kill myself").
  // The archive wrote "threatened her in return". The pinned corpus carries
  // the idiom once, on aiyysk.
  {
    term: '以生命相逼',
    renderings: [
      'threatened to take her own life',
      'threatened to kill herself',
      'used her own life as leverage',
    ],
    // LEDGER R15: the present tense and the other pronouns once passed.
    refusedForms: [
      'threatened',
      'threatens',
      'threatening',
    ].flatMap(function againstLife(verb,): readonly string[] {
      return [
        'her',
        'his',
        'their',
      ].map(function joined(pronoun,): string {
        return `${verb} ${pronoun} life`;
      },);
    },),
    why: 'someone who stakes their own life to stop another, threatening to kill themselves; "threatened her life" '
      + 'says the other person\'s life was threatened',
  },
  // TianqiChen66621 (2026-09-28) and TianqiChen66622 (2026-10-11) both shipped
  // the farewell glow's 像朗照溪水的月色扑面而来 as moonlight sweeping "toward
  // her" or "over her face", where the glow she cast comes at everyone she
  // knew (owner, 2026-10-11: "Add a dictionary entry"). The pinned corpus
  // carries the idiom on four pages, and on three of them (MocaKawai,
  // lin10104, republic_o85611) pressure or loneliness washes over the person
  // herself, so no rendering is refused: the target is whoever faces it.
  {
    term: '扑面而来',
    renderings: [
      'washed over',
      'came rushing at',
      'swept over',
      'came crashing down on',
      'met everyone',
    ],
    refusedForms: [],
    why: 'coming straight at whoever faces it: a feeling or pressure that washes over the person feeling it, or, when '
      + 'it describes what someone gives off, the people receiving it rather than the giver',
  },
];

//endregion Idiom renderings
