import type { RootContent, } from 'mdast';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type DeepReadonlyData,
  MdxParseError,
  parseSliceBody,
  requireMdxRefusal,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { IncumbentKind, } from './translate-absence.ts';

/**
 Module diagnostics expose counts and parser status, never source wording.
 */
const l = contextRoot({ tag: 'source-break-display', },);

/**
 Parser-proven Markdown break span in canonical source coordinates.
 */
type BreakSpan = {
  readonly start: number;
  readonly end: number;
};

/**
 Reads Markdown break spans in source order without interpreting code or attributes.

 @param text - exact canonical slice used to establish offsets

 @returns Non-overlapping spans, retaining the original line endings

 THE ENGINE'S STACK EXHAUSTION IS READ AS THE REFUSAL ONLY WHERE THE PARSE ENDS, inside `parseSliceBody`. The walk of the
 parsed tree (the loop over `pending`) is no part of the parse: an exhaustion it raises is no fact about the text and
 propagates.

 @throws {@link MdxParseError} when the shared slice grammar refuses this input, or the parse exhausted the stack

 @example
 ```ts
 const spans = sourceBreakSpans({ text: sourceText });
 ```
 */
function sourceBreakSpans({ text, }: { readonly text: string; },): readonly BreakSpan[] {
  /**
   Masks preserve offsets just as they do for structural admission.
   */
  const { root, } = parseSliceBody({ text, },);
  /**
   Reversed work stack preserves document order through nested containers.
   */
  const pending: DeepReadonlyData<RootContent>[] = [...root.children,].toReversed();
  /**
   Only real break nodes can license a presentation replacement.
   */
  const spans: BreakSpan[] = [];
  while (pending.length > 0) {
    /**
     Parsed node, whose position is supplied by the source parser.
     */
    const node = nonNullishOrThrow(pending.pop(),);
    if (node.type === 'break') {
      spans.push({
        start: nonNullishOrThrow(node.position
          ?.start
          .offset,),
        end: nonNullishOrThrow(node.position
          ?.end
          .offset,),
      },);
    }
    if ('children' in node)
      pending.push(...[...node.children,].toReversed(),);
  }
  return spans;
}

/**
 Makes source-only Markdown break syntax visible to a whole-slice translator.
 Canonical source and its offsets remain untouched outside this presentation.
 Explicit absent-incumbent provenance is required in addition to empty archive
 wording; omission of that fact conservatively preserves the original view.

 @internal

 @param sourceText - canonical original, never modified in place

 @param archiveText - actual archive wording, not a generated standing

 @param incumbentKind - caller-established existence of an archival rendering

 @param syntax - metadata remains outside this Markdown presentation

 @returns Source view with parsed hard breaks spelled visibly as intrinsic br

 @example
 ```ts
 const view = sourceBreakDisplay({ sourceText, archiveText: '', incumbentKind: 'absent' });
 ```
 */
export function sourceBreakDisplay(
  {
    sourceText,
    archiveText,
    incumbentKind,
    syntax,
  }: {
    readonly sourceText: string;
    readonly archiveText: string;
    readonly incumbentKind: IncumbentKind;
    readonly syntax?: SliceSyntax;
  },
): string {
  if (incumbentKind !== 'absent')
    return sourceText;
  if (archiveText !== '')
    return sourceText;
  if (syntax === 'front-matter')
    return sourceText;
  /**
   This function's diagnostics distinguish a display change from a source edit.
   */
  const dl = tagged({
    tag: sourceBreakDisplay.name,
    l,
  },);
  try {
    /**
     Source-order spans retain every character outside authored break syntax.
     */
    const spans = sourceBreakSpans({ text: sourceText, },);
    if (spans.length === 0)
      return sourceText;
    /**
     Disjoint slices are joined once rather than repeatedly copying the source.
     */
    const parts: string[] = [];
    /**
     First canonical character not yet copied to the display view.
     */
    let cursor = 0;
    for (const span of spans) {
      /**
       Parsed break positions include their LF, CRLF or CR line ending.
       */
      const endingWidth = sourceText.slice(
        span.end - 2,
        span.end,
      ) === '\r\n' ? 2 : 1;
      parts.push(
        sourceText.slice(
          cursor,
          span.start,
        ),
        '<br/>',
        sourceText.slice(
          span.end - endingWidth,
          span.end,
        ),
      );
      cursor = span.end;
    }
    parts.push(sourceText.slice(cursor,),);
    dl.info(`writer-source-break-display: ${String(spans.length,)} explicit breaks made visible; canonical source unchanged`,);
    return parts.join('',);
  }
  catch (error) {
    /**
     The grammar's refusal.
     */
    const refusal = requireMdxRefusal({ error, },);
    dl.warn(`writer-source-break-display: kept original presentation because ${refusal.message}`,);
    return sourceText;
  }
}
