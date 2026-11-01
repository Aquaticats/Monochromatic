import type { CommunityTerm, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Medical renderings
// CLASS ONE HUNDRED SIXTY-TWO (TianqiChen6663, 2026-09-26): 身患II型糖尿病
// shipped as the archive's "type II diabetes". The Roman numeral is a dated
// form; current English, Diabetes Canada's among it, writes "type 2
// diabetes". The pinned corpus carries the term on one entry, in the Latin
// capitals II. Kept beside `rendering-glossary.ts`, which spreads these
// entries into `RENDERING_GLOSSARY`, so neither file outgrows the line budget.

/**
 Medical terms the pinned corpus carries whose English the page writes in a
 dated form, with the form current English uses.
 */
export const MEDICAL_GLOSSARY: readonly CommunityTerm[] = [
  {
    term: 'II型糖尿病',
    // LEDGER R15: "type II diabetic" and "diabetes type II" once passed; a
    // hyphen folds to a space, so "type-II" needs no form of its own.
    renderings: [
      'type 2 diabetes',
      'type 2 diabetic',
    ],
    refusedForms: [
      'type ii diabetes',
      'type ii diabetic',
      'diabetes type ii',
      'diabetes, type ii',
    ],
    why: 'the common adult-onset diabetes; current English writes the type with an Arabic numeral, "type 2 diabetes", '
      + 'never the dated "type II"',
  },
  // CLASS ONE HUNDRED EIGHTY (TianqiChen66613, 2026-09-27): 激素一点一点进入
  // 她的身体 shipped as "the medication entered her system", hiding the
  // hormones the passage is about. Seeded without a refused form: the census
  // of the pin finds 激素 in seven paragraphs on five entries, and one of them
  // (shi_Yumiaoya) also writes 药物, where "medication" renders that word
  // (ledger R16, measured 2026-09-28: this comment once counted two, the
  // second a 药 inside 药娘).
  {
    term: '激素',
    renderings: [
      'hormones',
      'hormone',
      'estrogen',
    ],
    refusedForms: [],
    why: 'hormones, on these pages nearly always the hormones of transition (雌激素 is estrogen); "medication" hides '
      + 'what the passage is about',
  },
];

//endregion Medical renderings
