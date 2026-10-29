/**
 Date formatting template.
 
 Renders a `<time>` element with a human-readable date string
 and machine-readable `datetime` attribute.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 Renders a date as a `<time>` element with locale-formatted text.
 
 @param date - date to render
 
 @param lang - language code for locale-appropriate formatting
 
 @returns HTML string for the `<time>` element
 
 @example
 ```ts
 prettyDate({ date: new Date('2022-07-01'), lang: 'en' })
 // '<time datetime="...">Jul 01, 2022</time>'
 ```
 */
export function prettyDate(
  {
    date,
    lang,
  }: {
    readonly date: Date;
    readonly lang: string;
  },
): string {
  /**
   Locale-formatted display text rendered inside the `<time>` element.
   */
  const formatted = date.toLocaleDateString(
    lang,
    {
      year: 'numeric',
      month: 'short',
      day: '2-digit',
    },
  );

  return h({
    tag: 'time',
    attrs: { datetime: date.toISOString(), },
    text: formatted,
  },);
}
