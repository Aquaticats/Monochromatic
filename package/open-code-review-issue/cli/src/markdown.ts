/**
 GitHub Flavored Markdown boundary encoding.
 
 @module
 */

/**
 Escapes path text at final GitHub Markdown interpolation boundary.
 
 @param text - Untrusted source path displayed as ordinary inline text.
 
 @returns Markdown text that renders path characters literally.
 
 @example
 ```ts
 escapeMarkdownInline('src/[id].ts'); // 'src/\\[id\\].ts'
 ```
 */
export function escapeMarkdownInline(text: string,): string {
  return text
    .replaceAll(
    '\\',
    String.raw`\\`,
    )
    .replaceAll(
    '`',
    '\\`',
    )
    .replaceAll(
    '*',
    String.raw`\*`,
    )
    .replaceAll(
    '_',
    String.raw`\_`,
    )
    .replaceAll(
    '[',
    String.raw`\[`,
    )
    .replaceAll(
    ']',
    String.raw`\]`,
    )
    .replaceAll(
    '<',
    String.raw`\<`,
    )
    .replaceAll(
    '>',
    String.raw`\>`,
    );
}

/**
 Adds four spaces to every source line for GFM indented code blocks.
 
 @param code - OCR source text retained byte-for-byte after line prefixing.
 
 @returns Indented code block content including terminal blank source line.
 
 @example
 ```ts
 indentCode('a\nb'); // '    a\n    b'
 ```
 */
export function indentCode(code: string,): string {
  return code.split('\n',)
    .map(function indentLine(line,): string {
      return `    ${line}`;
    },)
    .join('\n',);
}
