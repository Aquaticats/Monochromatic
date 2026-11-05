//region Read task usage declarations without interpreting their help text

/**
 One flag or positional argument declared by a task.
 */
export type UsageOption = {
  readonly kind: 'flag' | 'arg';
  readonly name: string;
  readonly valueName?: string;
};

/**
 Reads the first word, stopping flag names before their value placeholder.

 @param body - Quoted declaration text.
 *
 @param kind - Flag names and positional placeholders have different delimiters.
 *
 @returns Name word without trimming declaration-leading whitespace.
 */
function declarationWord({
  body,
  kind,
}: {
  readonly body: string;
  readonly kind: UsageOption['kind'];
},): string {
  for (let end = 0; end < body.length; end += 1) {
    /**
     Whitespace terminates every name; a flag value starts at its opening angle bracket.
     */
    const character = body[end];
    if ((character?.trim() === '') || ((kind === 'flag') && (character === '<')))
      return body.slice(
        0,
        end,
      );
  }
  return body;
}

/**
 Removes flag dashes and positional argument brackets.

 @param word - First declaration word.
 *
 @returns Environment-compatible argument name.
 */
function declarationName(word: string,): string {
  for (let start = 0; start <= word.length; start += 1) {
    if (word[start] === '-')
      continue;
    /**
     Positional placeholders use angle or square brackets around their names.
     */
    const unprefixed = word.slice(start,);
    /**
     Only boundary brackets are syntax; interior characters remain literal.
     */
    const unwrapped = unprefixed.startsWith('[') || unprefixed.startsWith('<')
      ? unprefixed.slice(1,) : unprefixed;
    return unwrapped.endsWith(']') || unwrapped.endsWith('>') ? unwrapped.slice(
      0,
      -1,
    ) : unwrapped;
  }
  return '';
}

/**
 Parses quoted flag and argument declarations in one forward scan.

 @param usage - Usage string from the task definition; absent declarations produce no options.
 *
 @returns Options in declaration order.
 *
 @example
 ```ts
 const options = parseUsageSpec('flag "--mode <name>"\narg "<file>"');
 ```
 */
export function parseUsageSpec(usage = '',): readonly UsageOption[] {
  /**
   Each complete declaration contributes one descriptor.
   */
  const options: UsageOption[] = [];
  for (let cursor = 0; cursor < usage.length;) {
    /**
     Quoted help strings are scanned once and ignored unless preceded by a declaration keyword.
     */
    const opening = usage.indexOf(
      '"',
      cursor,
    );
    if (opening === -1)
      break;
    /**
     An unterminated string contributes no declaration, matching the supported usage grammar.
     */
    const closing = usage.indexOf(
      '"',
      opening + 1,
    );
    if (closing === -1)
      break;
    /**
     Disjoint prefixes make scanning linear even when help text contains many quoted strings.
     */
    const prefix = usage.slice(
      cursor,
      opening,
    );
    /**
     Keywords must have whitespace before their quoted body.
     */
    const keyword = prefix.trimEnd();
    /**
     Quoted body is examined only after the scan cursor has advanced past it.
     */
    const body = usage.slice(
      opening + 1,
      closing,
    );
    cursor = closing + 1;
    if ((prefix.length === keyword.length) || ((!keyword.endsWith('flag')) && (!keyword.endsWith('arg'))))
      continue;
    /**
     A positional argument can contain its required/optional brackets in the name word.
     */
    const kind = keyword.endsWith('flag') ? 'flag' : 'arg';
    /**
     Empty name declarations were ignored by the previous parser.
     */
    const name = declarationName(declarationWord({
      body,
      kind,
    },),);
    if (name.length === 0)
      continue;
    /**
     Value placeholders have a nonempty name between angle brackets.
     */
    const valueStart = body.indexOf('<',);
    /**
     Searching only the remaining body avoids rescanning a prefix.
     */
    const valueEnd = valueStart === -1 ? -1 : body.indexOf(
      '>',
      valueStart + 1,
    );
    options.push((valueStart >= 0) && (valueEnd > valueStart + 1)
      ? {
        kind,
        name,
        valueName: body.slice(valueStart + 1, valueEnd,),
      }
      : {
        kind,
        name,
      });
  }
  return options;
}

//endregion Read task usage declarations without interpreting their help text
