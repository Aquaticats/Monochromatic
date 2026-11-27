/**
 What the container's own answers say, read as text.

 The install proof asks its machine for three kinds of answer and judges each one on this host: the status line a background
 commit writes when it ends, the list of units a manager left failed, and the journal lines of a wrapper unit. Every reader
 here takes that text and returns a value, so a unit test holds each judgement without a container.
 */

/**
 The text a background commit's own status line begins with, as the shell line of `commit` writes it.
 */
const statusPrefix = 'status=';

/**
 The first character of a decimal digit, which every character of a status is compared against.
 */
const digitFirst = '0';

/**
 The last character of a decimal digit.
 */
const digitLast = '9';

/**
 The marker a wrapper service writes into the journal when it starts under a release.
 */
const startMarker = 'started release=';

/**
 The marker systemd writes when a start failed at the EXEC step, which is the step a program the install removed breaks.
 */
const execFailureMarker = '203/EXEC';

/**
 The marker systemd writes when a unit reached its start limit.
 */
const startLimitMarker = 'start-limit-hit';

/**
 Whether one character is a decimal digit.

 @param character - the character to judge, which is one character of the text under test

 @returns true for `0` through `9`
 */
function isDigit({ character, }: { readonly character: string; }): boolean {
  return (character >= digitFirst) && (character <= digitLast);
}

/**
 Whether one text is an integer the way a status line writes one: an optional minus and at least one decimal digit.

 @param text - the text after the status prefix

 @returns true when the whole text is such an integer

 @example
 ```ts
 isSignedInteger({ text: '-1' }); // true
 ```
 */
function isSignedInteger({ text, }: { readonly text: string; }): boolean {
  /**
   The text without its sign, which is all that may be digits.
   */
  const body = text.startsWith('-',) ? text.slice(1,) : text;
  if (body === '') return false;
  for (const character of body) {
    if (!isDigit({ character, })) return false;
  }
  return true;
}

/**
 The status a background commit ended with, read from the line it wrote for itself.

 A line is the status line only when it is a whole line of the text: `status=` and an integer, with nothing after it. A text
 that holds no such line, or a line whose number is malformed, answers the status of a commit that did not land.

 @param printed - what the machine printed when the proof read the commit's output file

 @returns the status the commit wrote, or one when no line carries a status

 @example
 ```ts
 statusOf({ printed: 'committed\nstatus=0\n' }); // 0
 ```
 */
export function statusOf({ printed, }: { readonly printed: string; }): number {
  /**
   The line the commit wrote for itself, or nothing when it wrote none.
   */
  const line = printed
    .split('\n',)
    .find(function beginsWithTheStatusPrefix(candidate) {
      return candidate.startsWith(statusPrefix,);
    },);
  if (line === undefined) return 1;
  /**
   The text after the prefix, which is the whole number or not a status at all.
   */
  const text = line.slice(statusPrefix.length,);
  if (!isSignedInteger({ text, })) return 1;
  return Number(text,);
}

/**
 The lines of a listing that hold something, each without the space around it.

 @param listed - what the machine printed, one entry per line

 @returns every line that is not empty after trimming, in the order it was listed

 @example
 ```ts
 nonEmptyLines({ listed: 'a.service\n\nb.service\n' }); // ['a.service', 'b.service']
 ```
 */
export function nonEmptyLines({ listed, }: { readonly listed: string; }): readonly string[] {
  return listed
    .split('\n',)
    .map(function withoutSurroundingSpace(line) {
      return line.trim();
    },)
    .filter(function holdsSomething(line) {
      return line !== '';
    },);
}

/**
 How many lines a journal read gave, counted by what each one says.
 */
export type JournalCounts = {
  /**
   How many starts of a wrapper service this run made.
   */
  readonly starts: number;
  /**
   How many starts the manager failed at the EXEC step.
   */
  readonly execFailures: number;
  /**
   How many wrapper units reached the start limit.
   */
  readonly limits: number;
};

/**
 How many of some lines carry one marker.

 @param lines - the lines under test

 @param needle - the text that marks a line

 @returns how many lines hold the needle
 */
function countHolding({
  lines,
  needle,
}: {
  readonly lines: readonly string[];
  readonly needle: string;
}): number {
  return lines
    .filter(function holdsTheNeedle(line) {
      return line.includes(needle,);
    },)
    .length;
}

/**
 What a wrapper unit's journal says this run made, counted by the marker on each line.

 @param journal - the journal lines the proof read, or an empty text when it read none

 @returns the counts of starts, EXEC failures and start-limit hits

 @example
 ```ts
 countsOfJournalLines({ journal: 'started release=0.6.1\n' }); // { starts: 1, execFailures: 0, limits: 0 }
 ```
 */
export function countsOfJournalLines({ journal, }: { readonly journal: string; }): JournalCounts {
  /**
   The journal's lines, each judged on its own.
   */
  const lines = journal.split('\n',);
  return {
    starts: countHolding({
      lines,
      needle: startMarker,
    },),
    execFailures: countHolding({
      lines,
      needle: execFailureMarker,
    },),
    limits: countHolding({
      lines,
      needle: startLimitMarker,
    },),
  };
}
