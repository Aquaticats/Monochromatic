import { DOUBLED_L_PAIRS, } from './canadian-spelling-pairs-doubled-l.ts';
import { WORD_PAIRS, } from './canadian-spelling-pairs-word.ts';

//region Canadian spelling pairs
// Words outside the stem families, each in lower case with its Canadian
// spelling. Every one has no second sense a respelling could damage: "meter" (a
// device), "check" (to verify), "tire" (to weary), "license" and "practise"
// (verbs), "analyses" and "paralyses" (plural nouns) are left to the judges.
// Where Canadian sources disagree the house policy's source decides: McGill's
// language guidelines write "counsellor, not counselor", "enrolment, not
// enrollment" and "program, not programme"; they are silent on "fulfil" and
// "skilful", which the Ryerson (Toronto Metropolitan) guide writes "fulfill"
// and "skillful", so those stand as written. "instalment" rests on the Ryerson
// guide alone.
//
// The doubled-l pairs and the pairs no ending pattern covers live in
// `canadian-spelling-pairs-doubled-l.ts` and `canadian-spelling-pairs-word.ts`,
// spread here in their places, so no file outgrows the line budget.

/**
 American or British spelling, lower case, to the Canadian one, which is lower
 case too except "ID". A map, as every table keyed by text is (ledger B77):
 the words it is looked up by are the page's own.
 */
export const CANADIAN_PAIRS: ReadonlyMap<string, string> = new Map([
  // -re
  [
    'center',
    'centre',
  ],
  [
    'centers',
    'centres',
  ],
  [
    'centered',
    'centred',
  ],
  [
    'theater',
    'theatre',
  ],
  [
    'theaters',
    'theatres',
  ],
  [
    'fiber',
    'fibre',
  ],
  [
    'fibers',
    'fibres',
  ],
  [
    'liter',
    'litre',
  ],
  [
    'liters',
    'litres',
  ],
  [
    'kilometer',
    'kilometre',
  ],
  [
    'kilometers',
    'kilometres',
  ],
  [
    'centimeter',
    'centimetre',
  ],
  [
    'centimeters',
    'centimetres',
  ],
  [
    'millimeter',
    'millimetre',
  ],
  [
    'millimeters',
    'millimetres',
  ],
  [
    'somber',
    'sombre',
  ],
  [
    'caliber',
    'calibre',
  ],
  [
    'saber',
    'sabre',
  ],
  [
    'specter',
    'spectre',
  ],
  [
    'luster',
    'lustre',
  ],
  [
    'meager',
    'meagre',
  ],
  [
    'maneuver',
    'manoeuvre',
  ],
  [
    'maneuvers',
    'manoeuvres',
  ],
  [
    'maneuvered',
    'manoeuvred',
  ],
  [
    'ocher',
    'ochre',
  ],
  [
    'sepulcher',
    'sepulchre',
  ],
  // A doubled l before a suffix, from `canadian-spelling-pairs-doubled-l.ts`.
  ...DOUBLED_L_PAIRS,
  // -yse
  [
    'analyse',
    'analyze',
  ],
  [
    'analysed',
    'analyzed',
  ],
  [
    'analysing',
    'analyzing',
  ],
  [
    'paralyse',
    'paralyze',
  ],
  [
    'paralysed',
    'paralyzed',
  ],
  [
    'paralysing',
    'paralyzing',
  ],
  // Forms no ending pattern covers, from `canadian-spelling-pairs-word.ts`.
  ...WORD_PAIRS,
],);

//endregion Canadian spelling pairs
