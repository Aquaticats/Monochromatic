import { isWhitespaceChar, } from '@monochromatic-dev/oxlint-plugin-shared/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Comment,
  Context,
  CreateOnceRule,
  Fix,
  Fixer,
  VisitorWithHooks,
} from '@oxlint/plugins';

/**
 Policy controlling whether TSDoc body lines carry a canonical asterisk prefix.
 */
type AsteriskPrefixMode = 'always' | 'never';

/**
 Source-backed line from a TSDoc comment body.
 */
type CommentBodyLine = {
  /**
   Zero-based physical line offset from comment opener.
   */
  readonly lineOffset: number;

  /**
   Absolute source offset where line begins.
   */
  readonly sourceStart: number;

  /**
   Line text without terminating line-feed character.
   */
  readonly text: string;
};

/**
 Inputs for reporting one TSDoc body-line prefix violation.
 */
type PrefixViolationParams = {
  /**
   Comment whose location anchors diagnostic line.
   */
  readonly comment: Comment;

  /**
   Rule context receiving diagnostic.
   */
  readonly context: Context;

  /**
   Analyzed physical body line.
   */
  readonly line: CommentBodyLine;

  /**
   Character offset of prefix or insertion point within line.
   */
  readonly prefixIndex: number;
};

/**
 Configuration error retained as runtime defense behind Oxlint's option schema.
 */
class AsteriskPrefixConfigurationError extends Error {
  /**
   Builds error for mode rejected by public rule contract.

   @param mode - invalid configured mode
   */
  constructor(mode: unknown,) {
    super(`stylistic/require-asterisk-prefix requires exactly one mode, "always" or "never"; received ${String(mode,)}.`,);
    this.name = 'AsteriskPrefixConfigurationError';
  }
}

/**
 Checks whether comment uses TSDoc block opener `/**`.

 @param comment - Oxlint comment candidate

 @returns whether candidate is TSDoc block

 @example
 ```ts
 isTsdocBlock(comment);
 ```
 */
function isTsdocBlock(comment: ForeignBorrowed<Comment>,): boolean {
  return (comment.type === 'Block')
    && comment.value
      .startsWith('*',);
}

/**
 Removes carriage return retained after splitting CRLF text on line feeds.

 @param line - raw comment line

 @returns line text without trailing carriage return

 @example
 ```ts
 lineContent(' text\r');
 ```
 */
function lineContent(line: string,): string {
  return line.endsWith('\r',)
    ? line.slice(
      0,
      -1,
    )
    : line;
}

/**
 Maps comment value lines to absolute source offsets.

 @param comment - TSDoc comment to split

 @returns body lines in source order

 @example
 ```ts
 commentBodyLines(comment);
 ```
 */
function commentBodyLines(comment: ForeignBorrowed<Comment>,): readonly CommentBodyLine[] {
  /**
   Source-backed lines in comment order, built here so the cursor never escapes this helper.
   */
  const lines: CommentBodyLine[] = [];
  /**
   Absolute start of current comment-value line; starts after opening `/*` and advances past each line feed.
   */
  let sourceStart = comment.range[0] + 2;
  /**
   Comment value split at line feeds, each line paired with its offset from the opener's line.
   */
  const numberedLines = comment.value
    .split('\n',)
    .entries();
  for (const [lineOffset, text,] of numberedLines) {
    lines.push({
      lineOffset,
      sourceStart,
      text,
    },);
    sourceStart += text.length + 1;
  }
  return lines;
}

/**
 Checks whether line starts with canonical marker `*`, followed by whitespace or line end.

 @param content - physical line without CR terminator

 @param prefixIndex - first non-whitespace character offset

 @returns whether canonical marker starts at offset

 @example
 ```ts
 hasCanonicalPrefix({ content: ' * text', prefixIndex: 1 });
 ```
 */
function hasCanonicalPrefix({
  content,
  prefixIndex,
}: {
  /**
   Physical line without CR terminator.
   */
  readonly content: string;

  /**
   First non-whitespace character offset.
   */
  readonly prefixIndex: number;
},): boolean {
  if (content.charAt(prefixIndex,) !== '*')
    return false;
  /**
   Character after marker, empty at line end.
   */
  const following = content.charAt(prefixIndex + 1,);
  return (following.length === 0) || isWhitespaceChar(following,);
}

/**
 Reads explicitly required prefix mode after schema validation.

 @param context - rule context carrying configured options

 @returns validated mode

 @throws {@link AsteriskPrefixConfigurationError} when host bypasses schema validation

 @example
 ```ts
 const mode = configuredMode(context);
 ```
 */
function configuredMode(context: ForeignBorrowed<Context>,): AsteriskPrefixMode {
  /**
   First and only public rule option.
   */
  const [mode,] = context.options;
  if ((mode === 'always') || (mode === 'never'))
    return mode;
  throw new AsteriskPrefixConfigurationError(mode,);
}

/**
 Reports and removes canonical prefix from one line.

 @param params - diagnostic location and source line

 @mutates params - emits Oxlint diagnostic through params.context

 @example
 ```ts
 reportUnexpectedPrefix({ context, comment, line, prefixIndex });
 ```
 */
function reportUnexpectedPrefix(params: ForeignBorrowed<PrefixViolationParams>,): void {
  /**
   Diagnostic and fix inputs.
   */
  const {
    comment,
    context,
    line,
    prefixIndex,
  } = params;
  /**
   Line text without CR terminator.
   */
  const content = lineContent(line.text,);
  /**
   Character after marker, used to remove one canonical separator.
   */
  const following = content.charAt(prefixIndex + 1,);
  /**
   Exclusive end of prefix removal; line-ending CR is never consumed.
   */
  const removeEnd = line.sourceStart
    + prefixIndex
    + ((following.length > 0) ? 2 : 1);
  context.report({
    loc: {
      start: {
        line: comment.loc
          .start
          .line
          + line.lineOffset,
        column: prefixIndex,
      },
    },
    messageId: 'unexpected',
    fix(fixer: ForeignBorrowed<Fixer>,): Fix {
      return fixer.removeRange([
        line.sourceStart + prefixIndex,
        removeEnd,
      ],);
    },
  },);
}

/**
 Reports and inserts canonical prefix on one line.

 @param params - diagnostic location and source line

 @mutates params - emits Oxlint diagnostic through params.context

 @example
 ```ts
 reportMissingPrefix({ context, comment, line, prefixIndex });
 ```
 */
function reportMissingPrefix(params: ForeignBorrowed<PrefixViolationParams>,): void {
  /**
   Diagnostic and fix inputs.
   */
  const {
    comment,
    context,
    line,
    prefixIndex,
  } = params;
  /**
   Body content without line-ending CR.
   */
  const content = lineContent(line.text,);
  /**
   Canonical asterisk column, one column after comment's slash.
   */
  const canonicalIndex = comment.loc
    .start
    .column
    + 1;
  /**
   Existing source position where insertion can begin without deleting content.
   */
  const insertionIndex = Math.min(
    prefixIndex,
    canonicalIndex,
  );
  /**
   Separator after inserted marker, omitted for blank body line.
   */
  const separator = content.trim()
    .length
    === 0 ? '' : ' ';
  /**
   Missing indentation plus marker and optional content separator.
   */
  const insertionText = `${' '.repeat(
    canonicalIndex - insertionIndex,
  )}*${separator}`;
  context.report({
    loc: {
      start: {
        line: comment.loc
          .start
          .line
          + line.lineOffset,
        column: insertionIndex,
      },
    },
    messageId: 'missing',
    fix(fixer: ForeignBorrowed<Fixer>,): Fix {
      return fixer.insertTextBeforeRange(
        [
          line.sourceStart + insertionIndex,
          line.sourceStart + insertionIndex,
        ],
        insertionText,
      );
    },
  },);
}

/**
 Applies selected prefix policy to every physical body line in one TSDoc block.

 @param context - rule context receiving diagnostics

 @param comment - TSDoc block to inspect

 @param mode - selected prefix policy

 @mutates context - emits one diagnostic per offending line

 @example
 ```ts
 checkComment({ context, comment, mode: 'never' });
 ```
 */
function checkComment({
  context,
  comment,
  mode,
}: {
  /**
   Rule context receiving diagnostics.
   */
  readonly context: Context;

  /**
   TSDoc block to inspect.
   */
  readonly comment: Comment;

  /**
   Selected prefix policy.
   */
  readonly mode: AsteriskPrefixMode;
},): void {
  /**
   Source-backed comment lines.
   */
  const lines = commentBodyLines(comment,);
  lines.forEach(function checkBodyLine(line,): void {
    if (line.lineOffset === 0)
      return;
    /**
     Body content without line-ending CR.
     */
    const content = lineContent(line.text,);
    /**
     Whether final value line is only indentation before closing delimiter.
     */
    const isClosingLine = (line.lineOffset === (lines.length - 1))
      && (content.trim()
        .length
        === 0);
    if (isClosingLine)
      return;
    /**
     First non-whitespace character, where canonical marker may appear.
     */
    const prefixIndex = content.length
      - content.trimStart()
        .length;
    /**
     Whether line carries canonical marker rather than literal leading asterisk content.
     */
    const hasPrefix = hasCanonicalPrefix({
      content,
      prefixIndex,
    },);
    if ((mode === 'never') && hasPrefix) {
      reportUnexpectedPrefix({
        comment,
        context,
        line,
        prefixIndex,
      },);
      return;
    }
    if ((mode === 'always') && (!hasPrefix)) {
      reportMissingPrefix({
        comment,
        context,
        line,
        prefixIndex,
      },);
    }
  },);
}

/**
 Enforces configured asterisk-prefix policy on every TSDoc block comment.

 `always` requires canonical `* ` body markers. `never` forbids those markers
 while preserving literal-leading content such as `**Note**` or `*through*`.
 Both modes include blank body lines and provide bidirectional autofixes.

 @example
 ```json
 {
   "stylistic/require-asterisk-prefix": ["warn", "never"]
 }
 ```
 */
export const requireAsteriskPrefix: CreateOnceRule = {
  meta: {
    type: 'layout',
    fixable: 'code',
    schema: {
      type: 'array',
      minItems: 1,
      maxItems: 1,
      items: [
        {
          type: 'string',
          enum: [
            'always',
            'never',
          ],
        },
      ],
    },
    docs: {
      description: 'Require or forbid canonical asterisk prefixes on TSDoc body lines.',
      recommended: true,
    },
    messages: {
      missing: 'TSDoc body line requires an asterisk prefix in always mode.',
      unexpected: 'TSDoc body line cannot have an asterisk prefix in never mode.',
    },
  },
  /**
   Handles effectful plugin callback.

   @param context - Foreign callback value carrying diagnostic capability.

   @mutates context - Emits Oxlint diagnostics through foreign rule context.

   @example
   ```ts
   createOnce(context);
   ```
   */
  createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
    return {
      Program(): void {
        /**
         Explicit prefix mode read after Oxlint installs current file's options.
         */
        const mode = configuredMode(context,);
        context.sourceCode
          .getAllComments()
          .filter(isTsdocBlock,)
          .forEach(function checkTsdocComment(comment,): void {
            checkComment({
              context,
              comment,
              mode,
            },);
          },);
      },
    };
  },
};
