import {
  at,
} from './range.ts';
import {
  inTemplateFlagsOf,
  lineTableOf,
} from './indent-lines.ts';
import {
  anchorLineIndexOf,
  bracketBaseLineIndexOf,
  closeValueOfOpenBracket,
  isCloseBracketToken,
  isOpenBracketToken,
  isSuffixKeywordToken,
} from './indent-routes.ts';
import {
  collectIndentNodeRecords,
  collectTemplateSpans,
  type ExpectationNode,
  type ExpectationToken,
  type IndentNodeRecord,
  tokenOwnerRecordIndexes,
} from './indent-tree.ts';

//region Expectation model
// Public result shapes for the per-line indentation model.

/**
 Which branch of the indentation model assigned an expectation.
 */
export type IndentExpectationRoute = 'ancestor' | 'closer' | 'suffix-keyword' | 'unmatched';

/**
 The indentation model's assignment for one checked line.
 */
export type LineIndentExpectation = {
  /**
   Zero-based line index, matching `SourceCode` line numbering minus one.
   */
  readonly lineIndex: number;
  /**
   Offset where the line begins.
   */
  readonly lineStart: number;
  /**
   Offset where the line's leading whitespace ends.
   */
  readonly contentStart: number;
  /**
   Leading-whitespace width found on the line.
   */
  readonly actualWidth: number;
  /**
   Leading-whitespace width the model assigns, built on earlier lines' model
   widths so one autofix pass reaches the fixed point.
   */
  readonly expectedWidth: number;
  /**
   Model branch that produced {@link LineIndentExpectation.expectedWidth}.
   */
  readonly route: IndentExpectationRoute;
};

/**
 Parameters for {@link computeIndentExpectations}.
 */
export type ComputeIndentExpectationsParams = {
  readonly sourceText: string;
  readonly root: ExpectationNode;
  readonly tokens: readonly ExpectationToken[];
  readonly visitorKeys: Readonly<Record<string, readonly string[]>>;
};
//endregion Expectation model

//region Bracket frames
// Open-bracket stack shared by the line loop; closers read the pre-line state.

/**
 One open bracket with the line its closer returns to.
 */
export type BracketFrame = {
  /**
   Close bracket text that pops this frame.
   */
  readonly closeValue: string;
  /**
   Line index whose leading width the closer restores.
   */
  readonly baseLine: number;
};

/**
 Parameters for {@link closeBaseLineOf}.
 */
export type CloseBaseLineOfParams = {
  readonly openFrames: readonly BracketFrame[];
  readonly closeValue: string;
};

/**
 Finds the base line a line-leading closer aligns with.
 
 The search takes the innermost open frame whose close text matches, so a
 closer follows its own bracket even while outer frames stay open.
 
 @param openFrames - bracket frames still open before the checked line
 
 @param closeValue - close bracket text the checked line starts with
 
 @returns base line index, or `-1` when no open frame matches the closer
 
 @example
 ```ts
 closeBaseLineOf({ openFrames, closeValue: '}' }); // opener's base line
 ```
 */
export function closeBaseLineOf({
  openFrames,
  closeValue,
}: CloseBaseLineOfParams,): number {
  for (
    let frameIndex = openFrames.length - 1;
    frameIndex >= 0;
    frameIndex -= 1
  ) {
    /**
     Frame at the probed index; the walk starts at the innermost open frame.
     */
    const frame = at({
      arr: openFrames,
      index: frameIndex,
    },);
    if (frame.closeValue === closeValue)
      return frame.baseLine;
  }
  return -1;
}
//endregion Bracket frames

//region Line model
// Per-line route resolution over the record, token, and line tables.

/**
 Parameters for {@link assignLineExpectation}.
 */
export type AssignLineExpectationParams = {
  readonly token: ExpectationToken;
  readonly width: number;
  readonly lineIndex: number;
  readonly lineStarts: readonly number[];
  readonly records: readonly IndentNodeRecord[];
  readonly ownerRecordIndexes: readonly number[];
  readonly firstTokenIndex: number;
  readonly resolvedWidths: readonly number[];
  readonly openFrames: readonly BracketFrame[];
};

/**
 Expected width for one checked line together with the route that produced it.
 */
export type LineIndentAssignment = {
  /**
   Leading-whitespace width the model assigns.
   */
  readonly expectedWidth: number;
  /**
   Model branch behind {@link LineIndentAssignment.expectedWidth}.
   */
  readonly route: IndentExpectationRoute;
};

/**
 Resolves the model route for a line-leading token and its expected width.
 
 Closers restore their bracket's base line, with no matching bracket keeping
 the line's width since no base exists. Suffix keywords align with their
 statement line. Every other token anchors at the nearest earlier-line AST
 ancestor's line plus two columns, or zero columns at source root.
 
 @param token - line-leading token whose route is being resolved
 
 @param width - leading-whitespace width the line currently carries
 
 @param lineIndex - zero-based index of the checked line
 
 @param lineStarts - ascending offset where each source line begins
 
 @param records - pre-order AST records the anchors are searched in
 
 @param ownerRecordIndexes - owning record index per token, `-1` when none
 
 @param firstTokenIndex - index in `tokens` of the line's leading token
 
 @param resolvedWidths - model widths already settled for earlier lines
 
 @param openFrames - bracket frames still open before this line's tokens
 
 @returns expected width and route for the line
 
 @example
 ```ts
 const assignment = assignLineExpectation({ ...params, token, width: 4 });
 ```
 */
export function assignLineExpectation({
  token,
  width,
  lineIndex,
  lineStarts,
  records,
  ownerRecordIndexes,
  firstTokenIndex,
  resolvedWidths,
  openFrames,
}: AssignLineExpectationParams,): LineIndentAssignment {
  if (isCloseBracketToken(token,)) {
    /**
     Base line this closer restores, or `-1` when no open frame matches it.
     */
    const baseLine = closeBaseLineOf({
      openFrames,
      closeValue: token.value,
    },);
    if (baseLine === (-1)) {
      return {
        expectedWidth: width,
        route: 'unmatched',
      };
    }
    return {
      expectedWidth: at({
        arr: resolvedWidths,
        index: baseLine,
      },),
      route: 'closer',
    };
  }
  /**
   Record owning the line-leading token, `-1` when no record contains it.
   */
  const ownerRecordIndex = at({
    arr: ownerRecordIndexes,
    index: firstTokenIndex,
  },);
  /**
   Node kind of the owning record; empty when the token has no owner, so
   suffix keywords never match a record-less token.
   */
  const ownerType = (ownerRecordIndex >= 0)
    ? at({
      arr: records,
      index: ownerRecordIndex,
    },)
      .node
      .type
    : '';
  /**
   Anchor line for the token, or `-1` for content rooted at the source top.
   */
  const anchorLine = anchorLineIndexOf({
    records,
    recordIndex: ownerRecordIndex,
    lineIndex,
    lineStarts,
  },);
  /**
   Model width of the anchor line; source-rooted content anchors at zero.
   */
  const anchorWidth = (anchorLine === (-1))
    ? 0
    : at({
      arr: resolvedWidths,
      index: anchorLine,
    },);
  if (isSuffixKeywordToken({
    token,
    ownerType,
  },)) {
    return {
      expectedWidth: anchorWidth,
      route: 'suffix-keyword',
    };
  }
  return {
    expectedWidth: (anchorLine === (-1))
      ? 0
      : anchorWidth + 2,
    route: 'ancestor',
  };
}

/**
 Computes the indentation model's expected leading width for every checked
 line of a file.
 
 Blank lines, comment-led lines (no token at the line's content start), and
 template-literal interior lines stay exempt. Checked lines take one of four
 routes: a closer's bracket base, a suffix keyword's statement line, the
 nearest earlier-line AST ancestor's line plus two columns, or source root at
 zero columns. Expected widths build on earlier lines' expected widths, so one
 autofix pass moves the whole file to the model's fixed point.
 
 @param sourceText - full file text the line table is built from
 
 @param root - AST root whose records the routes search
 
 @param tokens - every token of the file, in source order
 
 @param visitorKeys - visitor-key map naming each node kind's child properties
 
 @returns one entry per checked line, in line order
 
 @example
 ```ts
 const expectations = computeIndentExpectations({
   sourceText,
   root,
   tokens,
   visitorKeys,
 });
 ```
 */
export function computeIndentExpectations({
  sourceText,
  root,
  tokens,
  visitorKeys,
}: ComputeIndentExpectationsParams,): readonly LineIndentExpectation[] {
  /**
   Pre-order AST records; every route walks this table instead of the tree.
   */
  const records = collectIndentNodeRecords({
    root,
    visitorKeys,
  },);
  /**
   Source spans of template literals, whose interior lines stay exempt.
   */
  const templateSpans = collectTemplateSpans(records,);
  /**
   Owning record index per token, with declaration body wrappers collapsed.
   */
  const ownerRecordIndexes = tokenOwnerRecordIndexes({
    records,
    tokens,
  },);
  /**
   Per-line leading-whitespace facts, aligned by line index.
   */
  const {
    lineStarts,
    widths,
    contentStarts,
    blank,
  } = lineTableOf(sourceText,);
  /**
   Per-line flags marking lines whose content starts inside a template.
   */
  const templateLines = inTemplateFlagsOf({
    templateSpans,
    contentStarts,
  },);
  /**
   Expectation per checked line, appended in line order.
   */
  const expectations: LineIndentExpectation[] = [];
  /**
   Model width resolved per earlier line; checked lines contribute their
   expected width, so later anchors cascade within one pass.
   */
  const resolvedWidths: number[] = [];
  /**
   Open bracket frames before the line's tokens run; closers read these and
   the bracket-effect pass below extends them.
   */
  const openFrames: BracketFrame[] = [];
  /**
   First token index not yet assigned to a line; the loop advances it past
   every token starting on the line it just placed.
   */
  let lineTokenStart = 0;
  for (
    let lineIndex = 0;
    lineIndex < lineStarts.length;
    lineIndex += 1
  ) {
    /**
     Offset where this line begins.
     */
    const lineStart = at({
      arr: lineStarts,
      index: lineIndex,
    },);
    /**
     Offset where this line ends: the next line's start, or the file length
     for the last line.
     */
    const lineEnd = ((lineIndex + 1) < lineStarts.length)
      ? at({
        arr: lineStarts,
        index: lineIndex + 1,
      },)
      : sourceText.length;
    /**
     Token range starting on this line: `[lineTokenStart, lineTokenEnd)`.
     */
    let lineTokenEnd = lineTokenStart;
    while (
      (lineTokenEnd < tokens.length)
        && (at({
          arr: tokens,
          index: lineTokenEnd,
        },)
          .start
          < lineEnd)
    ) {
      lineTokenEnd += 1;
    }
    /**
     Leading-whitespace width found on this line.
     */
    const width = at({
      arr: widths,
      index: lineIndex,
    },);
    /**
     Offset where this line's content begins, past its leading whitespace.
     */
    const contentStart = at({
      arr: contentStarts,
      index: lineIndex,
    },);
    /**
     Whether this line's first token starts exactly at its content offset;
     comment-led lines fail this and stay exempt.
     */
    const lineStartsToken = (lineTokenStart < lineTokenEnd)
      && (at({
        arr: tokens,
        index: lineTokenStart,
      },)
        .start
        === contentStart);
    if (
      at({
        arr: blank,
        index: lineIndex,
      },)
        || (!lineStartsToken)
        || at({
          arr: templateLines,
          index: lineIndex,
        },)
    ) {
      resolvedWidths.push(width,);
    } else {
      /**
       Line-leading token this line's expectation is resolved for.
       */
      const token = at({
        arr: tokens,
        index: lineTokenStart,
      },);
      /**
       Model width for this line and the branch that produced it.
       */
      const assignment = assignLineExpectation({
        token,
        width,
        lineIndex,
        lineStarts,
        records,
        ownerRecordIndexes,
        firstTokenIndex: lineTokenStart,
        resolvedWidths,
        openFrames,
      },);
      resolvedWidths.push(assignment.expectedWidth,);
      expectations.push({
        lineIndex,
        lineStart,
        contentStart,
        actualWidth: width,
        expectedWidth: assignment.expectedWidth,
        route: assignment.route,
      },);
    }
    for (
      let tokenIndex = lineTokenStart;
      tokenIndex < lineTokenEnd;
      tokenIndex += 1
    ) {
      /**
       Token whose bracket effect extends or unwinds the open-frame stack.
       */
      const token = at({
        arr: tokens,
        index: tokenIndex,
      },);
      if (isOpenBracketToken(token,)) {
        openFrames.push({
          closeValue: closeValueOfOpenBracket(token.value,),
          baseLine: bracketBaseLineIndexOf({
            sourceText,
            records,
            ownerRecordIndex: at({
              arr: ownerRecordIndexes,
              index: tokenIndex,
            },),
            openValue: token.value,
            openOffset: token.start,
            openLine: lineIndex,
            lineStarts,
          },),
        },);
        continue;
      }
      if (isCloseBracketToken(token,)) {
        for (
          let frameIndex = openFrames.length - 1;
          frameIndex >= 0;
          frameIndex -= 1
        ) {
          if (at({
            arr: openFrames,
            index: frameIndex,
          },)
            .closeValue
            === token.value) {
            openFrames.length = frameIndex;
            break;
          }
        }
      }
    }
    lineTokenStart = lineTokenEnd;
  }
  return expectations;
}
//endregion Line model
