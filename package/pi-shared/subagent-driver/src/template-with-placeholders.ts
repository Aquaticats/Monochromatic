/**
 Fills a brief template: every `@NAME@` placeholder gets the value its filler names, and a template with a placeholder nobody
 filled is refused rather than written into a Pi run's brief.

 A brief that still reads `@FOCUS@` where its task should be sends an agent to work on a placeholder, so the filling is a check
 rather than a substitution: the placeholders a template carries are read first, and the values must cover every one of them.

 The template's own text is read once, into prose and placeholders, and the filled text is those parts joined with the values.
 The template is never read again, and a value is never read at all: text a value brings is text, even when it is shaped like a
 placeholder. A template is not a value's business, and a path or a commit subject is not a second template.

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
 A template placeholder whose name no value fills.
 */
export class TemplatePlaceholderError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'TemplatePlaceholderError';
}

/**
 One part of a template's own text: prose the template carries, or a placeholder whose value fills it.
 */
type TemplatePart =
  | {
    /**
     The prose this part carries.
     */
    readonly kind: 'text';

    /**
     The text, as the template wrote it.
     */
    readonly text: string;
  }
    | {
      /**
       The kind of part whose value fills it.
       */
      readonly kind: 'placeholder';

      /**
       The placeholder's name, without its at-signs.
       */
      readonly name: string;
    };

/**
 Reads a template's own text into its parts: the prose it carries, and each placeholder that text names, in the order it reads
 them.

 Only the template is read here. A value is never looked in, so text a value brings cannot become a placeholder.

 @param template - the template's text

 @returns the parts, in the order the template gives them

 @example
 ```ts
 const parts = partsOf({ template: 'Read @RULES@ first', },);
 ```
 */
function partsOf({ template, }: { readonly template: string },): readonly TemplatePart[] {
  /**
   The parts read so far.
   */
  const parts: TemplatePart[] = [];
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
      parts.push({
        kind: 'text',
        text: template.slice(from,),
      },);
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
      parts.push({
        kind: 'text',
        text: template.slice(
          from,
          close,
        ),
      },);
      from = close;
      continue;
    }
    parts.push(
      {
        kind: 'text',
        text: template.slice(
          from,
          first,
        ),
      },
      {
        kind: 'placeholder',
        name: candidate,
      },
    );
    from = close + 1;
  }
  return parts.filter(function carriesSomething(part,) {
    return (part.kind === 'placeholder') || (part.text !== '');
  },);
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
  for (const part of partsOf({ template, },)) {
    if (part.kind === 'placeholder') {
      names.add(part.name,);
    }
  }
  return [...names,];
}

/**
 Fills every placeholder a template carries.

 @param template - the template's text

 @param values - the value of each placeholder, by its name

 @param name - what the template is, for a message that names what was not filled

 @returns the filled text

 @throws TemplatePlaceholderError when a placeholder the template carries has no value

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
  for (const part of partsOf({ template, },)) {
    if (part.kind === 'text') {
      pieces.push(part.text,);
      continue;
    }
    /**
     The value this placeholder was given.
     */
    const value = values[part.name];
    if (value === undefined) {
      throw new TemplatePlaceholderError(
        `${name} carries @${part.name}@, and no value fills it. It fills: ${Object.keys(values,)
          .join(', ',)}.`,
      );
    }
    pieces.push(value,);
  }
  return pieces.join('',);
}
