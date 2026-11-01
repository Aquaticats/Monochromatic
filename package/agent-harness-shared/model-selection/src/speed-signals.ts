/**
 Model-name speed-signal scoring helpers.
 
 @module
 */

import type { ModelPricing, } from './types.ts';

//region Score constants

/**
 Score for models whose name says highspeed or high-speed.
 */
const HIGH_SPEED_SCORE = 100;

/**
 Score for models whose name says fast.
 */
const FAST_SCORE = 90;

/**
 Score for OpenAI Luna models.
 */
const LUNA_SCORE = 85;

/**
 Score for models whose name says flash.
 */
const FLASH_SCORE = 80;

/**
 Score for models whose name says spark.
 */
const SPARK_SCORE = FLASH_SCORE;

/**
 Score for OpenAI Terra models.
 */
const TERRA_SCORE = 75;

/**
 Score for models whose name says turbo.
 */
const TURBO_SCORE = 70;

/**
 Score for models whose name says nano.
 */
const NANO_SCORE = 60;

/**
 Score for models whose name says mini.
 */
const MINI_SCORE = 50;

/**
 Score for models whose name says haiku.
 */
const HAIKU_SCORE = 40;

/**
 Score for models whose name says lite or light.
 */
const LITE_SCORE = 30;

/**
 Score for models with no recognized speed signal.
 */
const NO_SPEED_SCORE = 0;

//endregion Score constants

//region Public API

/**
 Score model id and display name for explicit speed words.
 
 @param model - model identity to score
 
 @returns speed score, where higher means faster by local name heuristic
 
 @example
 ```typescript
 scoreModelSpeed({ id: 'kimi-k2.7-code-highspeed', name: 'Kimi highspeed' });
 ```
 */
export function scoreModelSpeed(
  model: Pick<ModelPricing, 'id' | 'name'>,
): number {
  /**
   Token and compact forms derived from model id and display name.
   */
  const signals = modelSpeedSignals(model,);
  if (signals
    .compact
    .includes('highspeed',))
    return HIGH_SPEED_SCORE;

  return signals
    .tokens
    .reduce(
    function maxSignalScore(
      currentScore,
      signal,
    ) {
      return Math.max(
        currentScore,
        speedSignalScore(signal,),
      );
    },
    NO_SPEED_SCORE,
    );
}

//endregion Public API

//region Signal extraction

/**
 Token and compact string forms used by speed scoring.
 */
type ModelSpeedSignals = {
  /**
   Separator-delimited model id and name tokens.
   */
  readonly tokens: readonly string[];
  /**
   Lowercase model id and name with separators removed.
   */
  readonly compact: string;
};

/**
 Build speed-scoring signals from model identity fields.
 
 @param model - model identity to inspect
 
 @returns separator-delimited tokens and compact form
 */
function modelSpeedSignals(
  model: Pick<ModelPricing, 'id' | 'name'>,
): ModelSpeedSignals {
  /**
   Model text fields that may carry provider speed words.
   */
  const values = [
    model.id,
    model.name,
  ];
  /**
   Tokens accumulated across every model text field.
   */
  const tokens = values.flatMap(function tokensFromValue(value,) {
    return splitSpeedTokens(value,);
  },);
  /**
   Compact form that catches compound strings like `high-speed` and `highspeed`.
   */
  const compact = values
    .map(function compactValue(value,) {
      return compactSpeedText(value,);
    },)
    .join('',);
  return {
    tokens,
    compact,
  };
}

/**
 Score one normalized speed signal token.
 
 @param signal - lowercase model-name token
 
 @returns speed score for the token
 */
function speedSignalScore(
  signal: string,
): number {
  if (signal === 'highspeed')
    return HIGH_SPEED_SCORE;
  if (signal === 'fast')
    return FAST_SCORE;
  if (signal === 'luna')
    return LUNA_SCORE;
  if (signal === 'flash')
    return FLASH_SCORE;
  if (signal === 'spark')
    return SPARK_SCORE;
  if (signal === 'terra')
    return TERRA_SCORE;
  if (signal === 'turbo')
    return TURBO_SCORE;
  if (signal === 'nano')
    return NANO_SCORE;
  if (signal === 'mini')
    return MINI_SCORE;
  if (signal === 'haiku')
    return HAIKU_SCORE;
  if ((signal === 'lite') || (signal === 'light'))
    return LITE_SCORE;
  return NO_SPEED_SCORE;
}

/**
 Split model text into lowercase speed-signal tokens.
 
 @param value - model id or display name
 
 @returns lowercase tokens separated by model-id punctuation or whitespace
 */
function splitSpeedTokens(
  value: string,
): string[] {
  /**
   Completed lowercase tokens.
   */
  const tokens: string[] = [];
  /**
   Characters collected for current token.
   */
  let currentTokenCharacters: string[] = [];
  for (const character of value.toLowerCase()) {
    if (isSpeedTokenSeparator(character,)) {
      if (currentTokenCharacters.length
        > 0) {
        tokens.push(currentTokenCharacters.join('',),);
        currentTokenCharacters = [];
      }
      continue;
    }
    currentTokenCharacters.push(character,);
  }
  if (currentTokenCharacters.length
    > 0)
    tokens.push(currentTokenCharacters.join('',),);
  return tokens;
}

/**
 Remove separators and lowercase model text for compound speed-word checks.
 
 @param value - model id or display name
 
 @returns compact lowercase text
 */
function compactSpeedText(
  value: string,
): string {
  /**
   Characters kept after dropping separators.
   */
  const characters: string[] = [];
  for (const character of value.toLowerCase()) {
    if (!isSpeedTokenSeparator(character,))
      characters.push(character,);
  }
  return characters.join('',);
}

/**
 Check whether a character separates model-name tokens.
 
 @param character - single character from model id or name
 
 @returns whether character is a token separator
 */
function isSpeedTokenSeparator(
  character: string,
): boolean {
  return (character === '.')
    || (character === '_')
    || (character === '-')
    || (character === '/')
    || (character === ':')
    || isAsciiWhitespace(character,);
}

/**
 Check whether a character is ASCII whitespace.
 
 @param character - character to check
 
 @returns whether character is whitespace used by model ids
 */
function isAsciiWhitespace(
  character: string,
): boolean {
  return (character === ' ')
    || (character === '\t')
    || (character === '\n')
    || (character === '\r')
    || (character === '\f')
    || (character === '\v');
}

//endregion Signal extraction
