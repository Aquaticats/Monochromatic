/**
 Model family keys and age keys for the Qwen Token Plan individual facts
 fallback: one id normalizes to the family its siblings share
 (`qwen3.8-max`, `qwen3.7-max`, `qwen4-max-preview` all reach `qwen-max`), and
 one id parses to the age key that orders it against those siblings. The walk
 is an index scan over hyphen-delimited tokens (no regular expressions), one
 linear pass per id. @module
 */

//region Constants

/**
 Smallest digit count a trailing date-suffix token carries, per the `-\d{3,4}`
 suffix shape `doc/planning/qwen-token-plan-individual-provider.md` names for
 snapshot ids (`deepseek-v4.1-flash-1017`, `deepseek-v4-pro-0813`).
 */
const DATE_SUFFIX_DIGIT_COUNT_MIN = 3;

/**
 Largest digit count a trailing date-suffix token carries, per the same
 `-\d{3,4}` suffix shape.
 */
const DATE_SUFFIX_DIGIT_COUNT_MAX = 4;

/**
 Token that marks a preview id (`qwen4-max-preview`); it names a release
 channel, not a family branch, so the family key drops it.
 */
const PREVIEW_TOKEN = 'preview';

/**
 Token joiner the ids and family keys share.
 */
const TOKEN_JOINER = '-';

//endregion Constants

//region Token scanning

/**
 Whether one character is an ASCII digit: the only digit form these ids use,
 so version and date runs parse without numeric coercion.

  @param character - character read out of an id token

  @returns `true` when the character is `0` to `9`
  */
function isDigit(character: string): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Whether one token is entirely ASCII digits: the shape of a date-suffix token
 and of a bare version component. Character scan instead of numeric coercion,
 which would accept `0x` spellings as numbers.

  @param token - hyphen-delimited id token

  @returns `true` when every character is a digit and the token is not empty
  */
function isDigitRun(token: string): boolean {
  if (token.length === 0) {
    return false;
  }
  for (let index = 0; index < token.length; index += 1) {
    if (!isDigit(token.charAt(index))) {
      return false;
    }
  }
  return true;
}

/**
 One token with its version runs removed, plus the components those runs
 carried: a version run is an optional `v`, a digit run, and any `.` digit run
 after it (`v4.1`, `3.8`, `1017`), which is exactly the `v?\d+(\.\d+)*` shape
 the family rule names. Scanned left to right in one pass.

  @param token - hyphen-delimited id token

  @returns the token text left after removal, and the version components found
  */
function scanVersionRuns({
  token,
}: {
  readonly token: string;
}): {
  readonly text: string;
  readonly versionComponents: readonly number[];
} {
  /**
   Remaining text and collected components, mutated in place as the scan
   advances since the exported shapes are readonly.
   */
  const scanned: {
    text: string;
    versionComponents: number[];
  } = {
    text: '',
    versionComponents: [],
  };
  for (let index = 0; index < token.length; ) {
    /**
     Character at the scan position.
     */
    const character = token.charAt(index);
    /**
     Character after the scan position, which a `v` prefix needs beside it.
     */
    const after = token.charAt(index + 1);
    if ((!isDigit(character)) && (!((character === 'v') && isDigit(after)))) {
      scanned.text += character;
      index += 1;
      continue;
    }
    if (character === 'v') {
      index += 1;
    }
    /**
     Digit text of one version component, accumulated before parsing.
     */
    let componentDigits = '';
    while ((index < token.length) && isDigit(token.charAt(index))) {
      componentDigits += token.charAt(index);
      index += 1;
    }
    scanned.versionComponents
      .push(Number(componentDigits));
    while ((index < token.length)
      && (token.charAt(index) === '.')
      && isDigit(token.charAt(index + 1))) {
      index += 1;
      componentDigits = '';
      while ((index < token.length) && isDigit(token.charAt(index))) {
        componentDigits += token.charAt(index);
        index += 1;
      }
      scanned.versionComponents
        .push(Number(componentDigits));
    }
  }
  return scanned;
}

/**
 Id tokens after the family-irrelevant ones drop: the preview channel marker
 and a trailing date-suffix token (`-1017`, `-0813`) stay out of the family key
 and out of the version components, and the suffix number travels separately as
 the snapshot component.

  @param id - model id to tokenize

  @returns the family tokens, and the snapshot component the suffix carried
  */
function familyTokensOf({
  id,
}: {
  readonly id: string;
}): {
  readonly tokens: readonly string[];
  readonly snapshotComponent: number;
} {
  /**
   Lowercased id tokens, which the family rule works on.
   */
  const loweredTokens = id.toLowerCase()
    .split(TOKEN_JOINER);
  /**
   Family tokens kept so far, without the preview marker.
   */
  const kept: string[] = [];
  for (const token of loweredTokens) {
    if ((token.length > 0) && (token !== PREVIEW_TOKEN)) {
      kept.push(token);
    }
  }
  /**
   Last token, which carries the date suffix when the id names one.
   */
  const lastToken = kept.at(-1);
  if ((lastToken !== undefined)
    && isDigitRun(lastToken)
    && (lastToken.length >= DATE_SUFFIX_DIGIT_COUNT_MIN)
    && (lastToken.length <= DATE_SUFFIX_DIGIT_COUNT_MAX)) {
    return {
      tokens: kept.slice(
        0,
        -1
      ),
      snapshotComponent: Number(lastToken),
    };
  }
  return {
    tokens: kept,
    snapshotComponent: 0,
  };
}

//endregion Token scanning

//region Public API

/**
 Age key one model id earns: the listing's `created` stamp when the listing
 carried one, the parsed version components otherwise, and the date-suffix
 number as the snapshot component. Two ids order by `created` when both carry
 one, by version components when either lacks it, then by snapshot number.
 */
export type QwenTokenPlanModelAgeKey = {
  readonly created?: number;
  readonly versionComponents: readonly number[];
  readonly snapshotComponent: number;
};

/**
 Family key one model id normalizes to: lowercased, minus the preview channel
 marker, minus a trailing date suffix, minus version runs. `qwen4-max-preview`,
 `qwen3.8-max`, and `qwen3.7-max` share `qwen-max`;
 `deepseek-v4.1-flash-1017`, `deepseek-v4.1-flash`, and `deepseek-v4-flash-0731`
 share `deepseek-flash`. An id whose tokens are all version runs keeps its
 lowercased id instead of collapsing to the empty key.

  @param id - model id from the listing

  @returns family key shared by the id's siblings

  @example
  ```ts
  qwenTokenPlanModelFamilyKey('qwen3.8-max'); // 'qwen-max'
  ```
  */
export function qwenTokenPlanModelFamilyKey(id: string): string {
  /**
   Family tokens and snapshot component the id's shape discloses.
   */
  const { tokens, } = familyTokensOf({ id, });
  /**
   Family tokens with their version runs stripped.
   */
  const stripped: string[] = [];
  for (const token of tokens) {
    /**
     Token text left after its version runs drop.
     */
    const { text, } = scanVersionRuns({ token, });
    if (text.length > 0) {
      stripped.push(text);
    }
  }
  if (stripped.length === 0) {
    return id.toLowerCase();
  }
  return stripped.join(TOKEN_JOINER);
}

/**
 Age key one model id earns for sibling ordering.

  @param id - model id from the listing

  @param created - listing `created` stamp when the entry carried one

  @returns age key ordered by `created`, parsed version, then snapshot number

  @example
  ```ts
  qwenTokenPlanModelAgeKey({ id: 'deepseek-v4.1-flash-1017', created: 1 });
  // { created: 1, versionComponents: [4, 1], snapshotComponent: 1017 }
  ```
  */
export function qwenTokenPlanModelAgeKey({
  id,
  created,
}: {
  readonly id: string;
  readonly created?: number;
}): QwenTokenPlanModelAgeKey {
  /**
   Family tokens and snapshot component the id's shape discloses.
   */
  const {
    tokens,
    snapshotComponent,
  } = familyTokensOf({ id, });
  /**
   Version components collected across the family tokens in id order.
   */
  const versionComponents: number[] = [];
  for (const token of tokens) {
    /**
     Version components this token's runs carried.
     */
    const { versionComponents: tokenComponents, } = scanVersionRuns({ token, });
    versionComponents.push(...tokenComponents);
  }
  return {
    ...(created === undefined ? {} : { created, }),
    versionComponents,
    snapshotComponent,
  };
}

/**
 Version component at one index, which exists because callers only read the
 shared prefix of two tuples.

  @param components - version components one age key carries

  @param index - position inside the shared prefix

  @returns the component at that position
  */
function versionComponentAt({
  components,
  index,
}: {
  readonly components: readonly number[];
  readonly index: number;
}): number {
  /**
   Component at the position, which the shared-prefix contract guarantees.
   */
  const component = components[index];
  if (component === undefined) {
    throw new Error(
      `model version component ${String(index)} missing from [${components.join(', ')}]`,
    );
  }
  return component;
}

/**
 Compare two age keys: negative when the left id is older, positive when it is
 newer, zero when the shapes cannot tell them apart. `created` decides when
 both listings carried it; parsed versions decide otherwise; the snapshot
 number decides last.

  @param left - age key of the first id

  @param right - age key of the second id

  @returns signed age difference

  @example
  ```ts
  compareQwenTokenPlanModelAgeKeys({
    left: qwenTokenPlanModelAgeKey({ id: 'widget4-max' }),
    right: qwenTokenPlanModelAgeKey({ id: 'widget3-max' }),
  }); // positive: the left id is newer
  ```
  */
export function compareQwenTokenPlanModelAgeKeys({
  left,
  right,
}: {
  readonly left: QwenTokenPlanModelAgeKey;
  readonly right: QwenTokenPlanModelAgeKey;
}): number {
  if ((left.created !== undefined) && (right.created !== undefined)
    && (left.created !== right.created)) {
    return left.created - right.created;
  }
  /**
   Version positions both tuples carry.
   */
  const sharedLength = Math.min(
    left.versionComponents
      .length,
    right.versionComponents
      .length,
  );
  for (let index = 0; index < sharedLength; index += 1) {
    /**
     Signed difference between the components at this position.
     */
    const difference = versionComponentAt({
      components: left.versionComponents,
      index,
    }) - versionComponentAt({
      components: right.versionComponents,
      index,
    });
    if (difference !== 0) {
      return difference;
    }
  }
  if (left.versionComponents
    .length
    !== right.versionComponents
      .length) {
    return left.versionComponents
      .length
      - right.versionComponents
        .length;
  }
  return left.snapshotComponent - right.snapshotComponent;
}

//endregion Public API
