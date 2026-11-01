/**
 English regular morphology fallbacks.
 
 @module
 */

/**
 Vowels used by regular English spelling heuristics.
 */
const ENGLISH_VOWELS: ReadonlySet<string> = new Set([
  'a',
  'e',
  'i',
  'o',
  'u',
],);

/**
 Final consonants that do not double in consonant-vowel-consonant gerunds.
 */
const CVC_FINAL_CONSONANT_EXCEPTIONS: ReadonlySet<string> = new Set([
  'w',
  'x',
  'y',
],);

/**
 Minimum base length that can contain final consonant-vowel-consonant spelling.
 */
const MINIMUM_CVC_BASE_LENGTH = 1 + 2;

/**
 Offset from end for initial consonant in final consonant-vowel-consonant spelling.
 */
const CVC_INITIAL_CONSONANT_OFFSET = 1 + 2;

/**
 Named-parameter object for English morphology helpers.
 */
type EnglishMorphologyOptions = {
  /**
   Verb base form used to derive regular fallback spelling.
   */
  readonly base: string;
};

/**
 Checks whether one English spelling character is treated as vowel.
 
 @param options - character wrapped for named-parameter calls
 
 @returns whether character is one of `a`, `e`, `i`, `o`, or `u`
 */
function isEnglishVowel(
  options: {
    readonly character: string;
  },
): boolean {
  /**
   Raw character for case-insensitive spelling checks.
   */
  const { character: rawCharacter, } = options;
  /**
   Lowercase character for case-insensitive spelling checks.
   */
  const character = rawCharacter.toLowerCase();
  return ENGLISH_VOWELS.has(character,);
}

/**
 Checks whether one English spelling character is treated as consonant.
 
 @param options - character wrapped for named-parameter calls
 
 @returns whether character is non-empty and not in {@link ENGLISH_VOWELS}
 */
function isEnglishConsonant(
  options: {
    readonly character: string;
  },
): boolean {
  /**
   Character inspected for consonant status.
   */
  const { character, } = options;
  return (character.length > 0) && (!isEnglishVowel(options,));
}

/**
 Reads required character from a string by offset from end.
 
 @param options - text and offset wrapped for named-parameter calls
 
 @returns character at requested offset from end
 
 @throws when offset falls outside text
 */
function requiredCharacterFromEnd(
  options: {
    readonly text: string;
    readonly offset: number;
  },
): string {
  /**
   Text and offset for end-relative lookup.
   */
  const {
    text,
    offset,
  } = options;
  /**
   Character at requested offset from end.
   */
  const character = text.at(-offset,);
  if (character === undefined)
    throw new Error('Cannot read required character from string.',);
  return character;
}

/**
 Checks whether base ends in consonant-vowel-consonant spelling that doubles for gerunds.
 
 @param options - verb base wrapped for named-parameter calls
 
 @returns whether regular gerund should double final consonant
 */
function isFinalConsonantDoublingCandidate(
  options: EnglishMorphologyOptions,
): boolean {
  /**
   Verb base form inspected for final spelling pattern.
   */
  const { base, } = options;
  if (base.length < MINIMUM_CVC_BASE_LENGTH)
    return false;
  /**
   Final consonant candidate in lower case.
   */
  const finalCharacter = requiredCharacterFromEnd({
    text: base,
    offset: 1,
  },)
    .toLowerCase();
  if (CVC_FINAL_CONSONANT_EXCEPTIONS.has(finalCharacter,))
    return false;
  /**
   Middle vowel candidate in final consonant-vowel-consonant spelling.
   */
  const vowelCharacter = requiredCharacterFromEnd({
    text: base,
    offset: 2,
  },);
  /**
   Initial consonant candidate in final consonant-vowel-consonant spelling.
   */
  const initialConsonant = requiredCharacterFromEnd({
    text: base,
    offset: CVC_INITIAL_CONSONANT_OFFSET,
  },);
  return isEnglishConsonant({ character: initialConsonant, },)
    && isEnglishVowel({ character: vowelCharacter, },)
    && isEnglishConsonant({ character: finalCharacter, },);
}

/**
 Derives regular English gerund spelling from base form.
 
 Irregular verbs and heuristic misses remain caller-overridable through explicit
 entry fields. `be` is a known two-letter irregular that derives to `bing`, so
 vocabulary should supply `gerund: 'being'` for that entry.
 
 @param options - verb base wrapped for named-parameter calls
 
 @returns gerund fallback for regular English spelling
 
 @example
 ```ts
 englishGerund({ base: 'die' }); // 'dying'
 englishGerund({ base: 'run' }); // 'running'
 englishGerund({ base: 'save' }); // 'saving'
 ```
 */
export function englishGerund(options: EnglishMorphologyOptions,): string {
  /**
   Verb base form used for derived spelling.
   */
  const { base, } = options;
  /**
   Lowercase base for suffix checks that should be case-insensitive.
   */
  const lowerBase = base.toLowerCase();
  if (lowerBase.endsWith('ie',))
    return `${base.slice(
      0,
      -2,
    )}ying`;
  if (isFinalConsonantDoublingCandidate({ base, },)) {
    /**
     Final consonant duplicated for consonant-vowel-consonant gerunds.
     */
    const finalCharacter = requiredCharacterFromEnd({
      text: base,
      offset: 1,
    },);
    return `${base}${finalCharacter}ing`;
  }
  /**
   Whether final silent `e` should be dropped before `ing`.
   */
  const hasSilentFinalE = lowerBase.endsWith('e',)
    && (!lowerBase.endsWith('ee',))
    && (!lowerBase.endsWith('oe',))
    && (!lowerBase.endsWith('ye',));
  if (hasSilentFinalE)
    return `${base.slice(
      0,
      -1,
    )}ing`;
  return `${base}ing`;
}

/**
 Derives regular English third-person singular present spelling from base form.
 
 Irregular verbs remain caller-overridable through explicit entry fields. This
 helper never doubles final consonants because `-s` spelling does not use that
 gerund rule.
 
 @param options - verb base wrapped for named-parameter calls
 
 @returns third-person singular present fallback for regular English spelling
 
 @example
 ```ts
 englishThirdSingular({ base: 'watch' }); // 'watches'
 englishThirdSingular({ base: 'try' }); // 'tries'
 englishThirdSingular({ base: 'save' }); // 'saves'
 ```
 */
export function englishThirdSingular(options: EnglishMorphologyOptions,): string {
  /**
   Verb base form used for derived spelling.
   */
  const { base, } = options;
  /**
   Lowercase base for suffix checks that should be case-insensitive.
   */
  const lowerBase = base.toLowerCase();
  if (lowerBase.endsWith('s',)
    || lowerBase.endsWith('x',)
    || lowerBase.endsWith('z',)
    || lowerBase.endsWith('ch',)
    || lowerBase.endsWith('sh',))
    return `${base}es`;
  /**
   Whether base has enough characters to inspect consonant before final `y`.
   */
  const hasInspectableFinalY = lowerBase.endsWith('y',) && (base.length >= 2);
  if (hasInspectableFinalY) {
    /**
     Character before final `y`, used to decide `ies` versus `s`.
     */
    const previousCharacter = requiredCharacterFromEnd({
      text: base,
      offset: 2,
    },);
    if (isEnglishConsonant({ character: previousCharacter, },))
      return `${base.slice(
        0,
        -1,
      )}ies`;
  }
  if (lowerBase.endsWith('o',))
    return `${base}es`;
  return `${base}s`;
}
