/**
 What one finished run's report reads as: its folder, why it ended, how many turns it took, and the head of the final message it
 ended with, cut to a width and a line count so a report of several runs stays readable.

 The report is read from the summary alone. `lastAssistantText` is cut to 900 characters when the summary is written, which is
 enough to read the head of a report; a whole report a run wrote to its own file is read from that file.

 @module
 */

import { isRecord, } from './read-pi-fields.ts';

/**
 How many lines of a run's final message are printed when a caller names no count. A report of several runs is read on one
 screen, and the head names what the run did and what it could not finish.
 */
export const REPORT_LINES = 30;

/**
 How wide a line of a run's final message is printed, when a caller names no width. A terminal holds about this much, and a
 message's own line breaks are kept.
 */
export const REPORT_WIDTH = 420;

/**
 How many more lines a run's final message held than were printed, as the line that says so.
 */
export const REPORT_MORE_LINES_SUFFIX = 'more lines';

/**
 Why one run ended, and what its summary says of it.
 */
export type SummaryOfARun = {
  /**
   Why the run ended, as the driver writes it.
   */
  readonly endReason?: string;

  /**
   How many model turns the run started.
   */
  readonly turns?: number;

  /**
   The last assistant message, as the summary cut it.
   */
  readonly lastAssistantText?: string;
};

/**
 Reads the text of a run's final message, cut to the width a line prints.

 @param text - the message

 @param width - how much of each line to keep

 @returns the message's lines, blank ones left out, each cut to that width

 @example
 ```ts
 const lines = reportLinesIn({ text: 'a\\n\\nb', width: 420, },);
 ```
 */
export function reportLinesIn({
  text,
  width,
}: {
  readonly text: string;
  readonly width: number;
},): readonly string[] {
  return text.split('\n',)
    .filter(function isNotBlank(line,) {
      return line.trim() !== '';
    },)
    .map(function cutTo(line,) {
      return (line.length <= width) ? line : line.slice(
        0,
        width,
      );
    },);
}

/**
 Reads one finished run's report.

 @param runFolder - the run folder's name, which the report names

 @param summary - what the run's summary says

 @param lines - how many lines of the final message to print

 @param width - how much of each line to print

 @returns the lines to print, the run's own line first

 @example
 ```ts
 const lines = reportLinesOf({ runFolder: 'copy-route-1', summary: { endReason: 'idle', turns: 40, lastAssistantText: 'done' }, lines: 30, width: 420, },);
 ```
 */
export function reportLinesOf({
  runFolder,
  summary,
  lines = REPORT_LINES,
  width = REPORT_WIDTH,
}: {
  readonly runFolder: string;
  readonly summary: SummaryOfARun;
  readonly lines?: number;
  readonly width?: number;
},): readonly string[] {
  /**
   The message's lines, as they are printed.
   */
  const report = reportLinesIn({
    text: summary.lastAssistantText ?? '',
    width,
  },);
  return [
    `=== ${runFolder}: ${summary.endReason ?? 'no-end-reason'}, ${String(summary.turns ?? 0,)} turns`,
    ...report.slice(
      0,
      lines,
    ),
    ...((report.length <= lines)
      ? []
      : [`[${String(report.length - lines,)} ${REPORT_MORE_LINES_SUFFIX}]`,]),
  ];
}

/**
 Reads the line that says an agent has no finished run.

 @param agent - the agent's name

 @returns the line to print

 @example
 ```ts
 const line = noFinishedRunLineOf('copy-route');
 ```
 */
export function noFinishedRunLineOf(agent: string,): string {
  return `=== ${agent}: no finished run`;
}

/**
 Reads what a summary says about its run, keeping the fields a report prints.

 A summary that holds no JSON object, or a field that is not the shape the driver writes, is left out rather than printed as
 `undefined`: a report of a run whose summary another tool touched still reads.

 @param shown - the summary as it was written

 @returns why the run ended, how many turns it took, and its last message

 @example
 ```ts
 const summary = summaryIn({ shown: '{"endReason":"idle","turns":40,"lastAssistantText":"done"}' },);
 ```
 */
export function summaryIn({ shown, }: { readonly shown: string },): SummaryOfARun {
  /**
   The summary as it was parsed.
   */
  const parsed: unknown = JSON.parse(shown,);
  if (!isRecord(parsed,)) {
    return {};
  }
  return {
    ...(((typeof parsed.endReason) === 'string') ? { endReason: parsed.endReason, } : {}),
    ...(((typeof parsed.turns) === 'number') ? { turns: parsed.turns, } : {}),
    ...(((typeof parsed.lastAssistantText) === 'string') ? { lastAssistantText: parsed.lastAssistantText, } : {}),
  };
}
