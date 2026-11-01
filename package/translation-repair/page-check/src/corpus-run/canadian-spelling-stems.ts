//region Canadian spelling stems
// Stems whose inflections Canadian English spells one way (ledger K3: the
// first list named "honor" and "honored" but not "honoring", "summarise" not
// at all). `canadian-spelling-words.ts` crosses each stem with its endings.
// Each ending is listed only where it keeps the Canadian letter: Canadian
// writes "honourable" but "honorary", "humourless" but "humorous" and
// "humoral", "colourful" but "coloration", "vigour" but "vigorous", so "-ary",
// "-ous", "-al", "-ation" and "-ist" never join an "-our" stem, and
// `canadian-spelling.unit.test.ts` holds those words unchanged. Sources: the
// house policy's own list (McGill's language guidelines, after the Canadian
// Press Stylebook and the Canadian Oxford: "labour but laborious", "honour,
// honourable, honoured but honorary", "criticize, organize, capitalize").

/**
 What an "-or" stem reads up to its "r": "colo" for color, colour.
 */
export const OUR_STEMS: readonly string[] = [
  'ardo',
  'armo',
  'behavio',
  'cando',
  'clamo',
  'colo',
  'demeano',
  'endeavo',
  'favo',
  'fervo',
  'flavo',
  'glamo',
  'harbo',
  'hono',
  'humo',
  'labo',
  'misdemeano',
  'neighbo',
  'odo',
  'parlo',
  'ranco',
  'rigo',
  'rumo',
  'savio',
  'savo',
  'splendo',
  'tumo',
  'valo',
  'vapo',
  'vigo',
];

/**
 Endings after an "-our" stem's "r" that keep the "u" in Canadian English.
 */
export const OUR_ENDINGS: readonly string[] = [
  '',
  's',
  'ed',
  'ing',
  'ful',
  'fully',
  'less',
  'able',
  'ably',
  'er',
  'ers',
  'ite',
  'ites',
  'itism',
  'hood',
  'hoods',
  'ly',
];

/**
 What a British "-ise" verb reads up to its "is": "organ" for organise,
 organize. Only verbs whose "-ise" is the suffix Canadian writes "-ize";
 "advise", "exercise", "promise", "surprise", "compromise" and the other
 words whose "-ise" is their root are not stems here.
 */
export const IZE_STEMS: readonly string[] = [
  'agon',
  'antagon',
  'apolog',
  'author',
  'capital',
  'categor',
  'character',
  'civil',
  'critic',
  'dehuman',
  'depatholog',
  'desensit',
  'destabil',
  'dramat',
  'emphas',
  'euthan',
  'familiar',
  'fantas',
  'femin',
  'final',
  'general',
  'global',
  'harmon',
  'hospital',
  'human',
  'hypothes',
  'ideal',
  'idol',
  'internal',
  'jeopard',
  'legal',
  'legitim',
  'local',
  'marginal',
  'maxim',
  'medical',
  'memor',
  'minim',
  'mobil',
  'modern',
  'moral',
  'neutral',
  'normal',
  'optim',
  'organ',
  'ostrac',
  'patholog',
  'patron',
  'personal',
  'polar',
  'popular',
  'priorit',
  'public',
  'rational',
  'real',
  'recogn',
  'reorgan',
  'retraumat',
  'revolution',
  'romantic',
  'satir',
  'scrutin',
  'sensit',
  'sexual',
  'social',
  'special',
  'stabil',
  'standard',
  'steril',
  'stigmat',
  'subsid',
  'summar',
  'symbol',
  'sympath',
  'synchron',
  'theor',
  'traumat',
  'trivial',
  'util',
  'vandal',
  'victim',
  'visual',
  'vocal',
  'western',
];

/**
 Endings after an "-ise" stem's "is" that Canadian English writes with "iz".
 The bare stem plus "is" is never one: "emphasis" and "hypothesis" are nouns.
 */
export const IZE_ENDINGS: readonly string[] = [
  'e',
  'ed',
  'es',
  'ing',
  'ation',
  'ations',
  'ational',
  'er',
  'ers',
  'able',
];

//endregion Canadian spelling stems
