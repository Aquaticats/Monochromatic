/**
 Fills a brief template: every `@NAME@` placeholder gets the value its filler names, and a template with a placeholder nobody
 filled is refused rather than written into a Pi run's brief.

 A brief that still reads `@FOCUS@` where its task should be sends an agent to work on a placeholder, so the filling is a check
 rather than a substitution: the placeholders a template carries are read first, the values must cover every one of them, and
 nothing may remain afterwards.

 The text is read by scanning for its at-signs, one pass over the template, because the grammar is one a person reads: two
 at-signs around a name of capitals, digits and underscores, and nothing else is a placeholder.

 @module
 */

/**
 The letters a placeholder's name is made of, capital only.
 */
const UPPERCASE = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';

/**
 The digits a placeholder's name may carry after its first letter.
 */
const DIGITS = '0123456789';

/**
 Says whether text between two at-signs is a placeholder's name: capital letters, digits and underscores, starting with a letter.

 @param candidate - the text between two at-signs

 @returns whether it names a placeholder

 @example
 ```ts
 const isAName = isPlaceholderName('FOCUS');
 ```
 */
function isPlaceholderName(candidate: string,): boolean {
  if (candidate === '') {
    return false;
  }
  /**
   The placeholder's first character, which must be a capital letter.
   */
  const first = candidate.slice(
    0,
    1,
  );
  if (!UPPERCASE.includes(first,)) {
    return false;
  }
  for (let index = 1; index < candidate.length; index += 1) {
    /**
     The character at this place.
     */
    const character = candidate[index] ?? '';
    if (UPPERCASE.includes(character,)) {
      continue;
    }
    if (DIGITS.includes(character,)) {
      continue;
    }
    if (character === '_') {
      continue;
    }
    return false;
  }
  return true;
}

/**
 A template placeholder that a filler did not fill, or a placeholder left behind in a filled brief.
 */
export class TemplatePlaceholderError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'TemplatePlaceholderError';
}

/**
 Reads the placeholders one template carries, each once, in the order they appear.

 @param template - the template's text

 @returns the placeholders' names, without their at-signs

 @example
 ```ts
 const names = placeholdersIn({ template: 'a @FOCUS@ and a @COMMIT@', },);
 ```
 */
export function placeholdersIn({ template, }: { readonly template: string },): readonly string[] {
  /**
   The names read so far, each once, in the order the template reads them.
   */
  const names = new Set<string>();
  for (let index = template.indexOf('@',); index !== (-1);) {
    /**
     The at-sign that would close that placeholder.
     */
    const close = template.indexOf(
      '@',
      index + 1,
    );
    if (close === (-1)) {
      break;
    }
    /**
     The text between the two at-signs.
     */
    const candidate = template.slice(
      index + 1,
      close,
    );
    if (isPlaceholderName(candidate,)) {
      names.add(candidate,);
    }
    index = template.indexOf(
      '@',
      close + 1,
    );
  }
  return [...names,];
}

/**
 Fills every placeholder a template carries.

 @param template - the template's text

 @param values - the value of each placeholder, by its name

 @param name - what the template is, for a message that names what was not filled

 @returns the filled text

 @throws TemplatePlaceholderError when a placeholder has no value, or one is still there after filling

 @example
 ```ts
 const brief = filledTemplate({ template: 'a @FOCUS@', values: { FOCUS: 'data loss', }, name: 'the brief', },);
 ```
 */
export function filledTemplate({
  template,
  values,
  name,
}: {
  readonly template: string;
  readonly values: Readonly<Record<string, string>>;
  readonly name: string;
}): string {
  /**
   The pieces of the filled text, in the order the template gives them.
   */
  const pieces: string[] = [];
  for (let from = 0; from < template.length;) {
    /**
     The first at-sign at or after where the reading stands.
     */
    const first = template.indexOf(
      '@',
      from,
    );
    /**
     The at-sign that would close that placeholder, or minus one when none follows.
     */
    const close = (first === (-1))
      ? (-1)
      : template.indexOf(
        '@',
        first + 1,
      );
    if ((first === (-1)) || (close === (-1))) {
      pieces.push(template.slice(from,),);
      break;
    }
    /**
     The text between the two at-signs.
     */
    const candidate = template.slice(
      first + 1,
      close,
    );
    if (!isPlaceholderName(candidate,)) {
      pieces.push(template.slice(
        from,
        close,
      ),);
      from = close;
      continue;
    }
    /**
     The value this placeholder was given.
     */
    const value = values[candidate];
    if (value === undefined) {
      throw new TemplatePlaceholderError(
        `${name} carries @${candidate}@, and no value fills it. It fills: ${Object.keys(values,)
          .join(', ',)}.`,
      );
    }
    pieces.push(
      template.slice(
        from,
        first,
      ),
      value,
    );
    from = close + 1;
  }
  /**
   The filled text.
   */
  const filled = pieces.join('',);
  /**
   The placeholders still there, which a filled brief must not carry into a run.
   */
  const left = placeholdersIn({ template: filled, },);
  if (left.length > 0) {
    throw new TemplatePlaceholderError(
      `${name} still carries ${left.map(function atSigns(placeholder,) {
        return `@${placeholder}@`;
      },)
        .join(', ',)} after filling, so a run would read a placeholder as its task.`,
    );
  }
  return filled;
}
