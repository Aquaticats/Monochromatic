//region Present-tense vocabulary
// The word lists the present-tense rules compare against. Each list names the words one
// rule reads, so a rule and its vocabulary can be read apart from the scanning code. Plain
// lists of literals, so this file holds data and no logic.

/**
 Words that end in -s without being a present verb of the subject.
 */
export const SKIPPED_S_WORDS: readonly string[] = [
  'was',
  'has',
  'does',
  'is',
  'this',
  'its',
  'his',
  'hers',
  'ours',
  'yours',
  'us',
  'as',
  'thus',
  'across',
  'always',
  'towards',
  'besides',
  'perhaps',
  'sometimes',
  'plus',
  'unless',
  'less',
  'yes',
  'things',
  'days',
  'years',
  'times',
  'lots',
  'mrs',
  'miss',
  'gets',
];

/**
 Bare present verbs that a pronoun followed by them narrates as the person's act or feeling.
 */
export const BARE_VERBS: readonly string[] = [
  'live',
  'bear',
  'know',
  'feel',
  'love',
  'want',
  'miss',
  'remember',
  'have',
  'keep',
  'stay',
  'look',
  'smile',
  'laugh',
  'care',
  'need',
  'think',
  'believe',
  'hope',
  'wish',
  'try',
  'hold',
  'carry',
  'accept',
  'forgive',
  'fight',
  'face',
  'lead',
  'play',
  'sing',
  'dream',
  'wake',
  'rest',
];

/**
 Auxiliaries and copulas that, after a subject, mark a present state.
 */
export const PRESENT_AUXILIARIES: readonly string[] = [
  'is',
  'are',
  'am',
  'has',
  'does',
  'do',
];

/**
 Auxiliaries that, placed directly before a subject, make the sentence an inversion.
 */
export const INVERTING_AUXILIARIES: readonly string[] = [
  'did',
  'does',
  'do',
  'would',
  'will',
  'can',
  'could',
  'should',
  'must',
  'may',
  'might',
  'had',
];

/**
 Past participles that, after a contracted subject, make the contraction a past state.
 */
export const PAST_PARTICIPLES: readonly string[] = [
  'been',
  'had',
  'got',
  'gone',
  'done',
  'left',
];

/**
 Pronouns the pronoun rules read as the subject.
 */
export const PRONOUNS: readonly string[] = [
  'she',
  'he',
  'they',
];

//endregion Present-tense vocabulary
