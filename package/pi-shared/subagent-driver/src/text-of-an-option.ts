/**
 Reading the value of one option out of text another program wrote, such as a unit's own command line or a list of run folder
 names, without a regular expression.

 Every reader here scans the text once, from left to right, so a line of any length costs one pass and no backtracking.

 @module
 */

/**
 The characters that end an option's value where systemd prints a command line, and where a shell would end a word.
 */
const VALUE_END_CHARACTERS: ReadonlySet<string> = new Set<string>([
  ' ',
  '\t',
  '\n',
  ';',
  '"',
  "'",
]);

/**
 Says where an option's value ends.

 @param text - the text the value starts in

 @returns the index of the first character that ends a value, or the text's length

 @example
 ```ts
 const end = endOfValueIn('neuralwatt ; }',);
 ```
 */
function endOfValueIn(text: string,): number {
  for (let index = 0; index < text.length; index += 1) {
    if (VALUE_END_CHARACTERS.has(text[index] ?? '',)) {
      return index;
    }
  }
  return text.length;
}

/**
 Reads the value of one option out of text, where the option may be written with an equals sign or with a space after it.

 @param text - the text to read

 @param option - the option's name, without an equals sign, such as `--provider`

 @returns the value, or empty text when the text does not carry that option

 @example
 ```ts
 const provider = valueOfOption({ text: '{ argv[]=node /p.ts --provider=neuralwatt --model=deepseek-v4.1-flash ; }', option: '--provider', },);
 ```
 */
export function valueOfOption({
  text,
  option,
}: {
  readonly text: string;
  readonly option: string;
}): string {
  /**
   Where the option is written.
   */
  const at = text.indexOf(option,);
  if (at === (-1)) {
    return '';
  }
  /**
   What follows the option.
   */
  const rest = text.slice(at + option.length,);
  if ((!rest.startsWith('=',)) && (!rest.startsWith(' ',))) {
    // A longer option that starts with this one, such as `--provider-token`, is a different option.
    return '';
  }
  /**
   The value's text.
   */
  const from = rest.slice(1,);
  return from.slice(
    0,
    endOfValueIn(from,),
  );
}

/**
 Says whether text is digits only, which is what a run folder's number is written as.

 @param text - the text to read

 @returns whether every character is a digit and there is at least one

 @example
 ```ts
 const isNumber = isDigitsOnly('42',);
 ```
 */
export function isDigitsOnly(text: string,): boolean {
  if (text.length === 0) {
    return false;
  }
  for (let index = 0; index < text.length; index += 1) {
    /**
     The character at this position.
     */
    const character = text.charAt(index,);
    if ((character < '0') || (character > '9')) {
      return false;
    }
  }
  return true;
}

/**
 Reads the first word of a line, which is the unit's name in what `systemctl list-units` prints.

 @param line - one line of that listing

 @returns the first word, or empty text for a line that is blank or only spaces

 @example
 ```ts
 const unit = firstWordOf('pi-a.service loaded active running',);
 ```
 */
export function firstWordOf(line: string,): string {
  return line.trim()
    .split(' ',)
    .find(function isAWord(word,) {
      return word !== '';
    },)
    ?? '';
}
