import { isWhitespace, } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

/**
 Parameters for {@link isOnlyWhitespaceOrSeparator}.
 */
export type IsOnlyWhitespaceOrSeparatorParams = {
  /**
   Source slice between two syntax items.
   */
  readonly text: string;
  /**
   Separator character allowed alongside whitespace.
   */
  readonly separator: ',' | ';';
};

/**
 Returns true when `text` contains only ASCII whitespace and one separator.
 
 Empty strings return true. The scan is linear and intentionally tiny because
 autofix safety checks use it on source slices that may contain comments.
 
 @param params - source slice and allowed separator
 
 @returns whether the slice is safe to replace verbatim
 
 @example
 ```ts
 isOnlyWhitespaceOrSeparator({ text: ', ', separator: ',' });
 ```
 */
export function isOnlyWhitespaceOrSeparator(
  params: IsOnlyWhitespaceOrSeparatorParams,
): boolean {
  /**
   Source slice and allowed separator.
   */
  const {
    text,
    separator,
  } = params;
  for (const char of text) {
    if ((!isWhitespace(char,)) && (char !== separator))
      return false;
  }
  return true;
}
