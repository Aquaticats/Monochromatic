/**
 Readers of workflow and action files written in block style, shared by the tests that check them.

 They follow indentation instead of parsing YAML, because this package has no YAML parser and the files are written in
 one regular style.
 */

/**
 Indentation of a line: the count of leading spaces.

 @param line - one line of a workflow file

 @returns how many spaces start the line

 @example
 ```ts
 indentOf('    - name: a');
 ```
 */
export function indentOf(line: string): number {
  return line.length
    - line.trimStart()
      .length;
}

/**
 Take the lines below a line that belong to it: the ones before the first non-blank line indented less than a bound.

 @param lines - all lines of the file

 @param at - index of the line whose block is wanted

 @param stopsAt - the shallowest indentation a line of the block may have; a non-blank line shallower than this ends it

 @returns the lines of the block

 @example
 ```ts
 blockBelow({ lines: ['a:', '  b', 'c:'], at: 0, stopsAt: 1 });
 ```
 */
export function blockBelow({
  lines,
  at,
  stopsAt,
}: {
  readonly lines: readonly string[];
  readonly at: number;
  readonly stopsAt: number;
}): readonly string[] {
  /**
   Lines after the key.
   */
  const body = lines.slice(at + 1);
  /**
   Index of the first line outside the block.
   */
  const end = body.findIndex(function isOutside(line) {
    return (line.trim() !== '') && (indentOf(line) < stopsAt);
  });
  return end === (-1) ? body : body.slice(
    0,
    end
  );
}

/**
 Collect the scripts of the `run:` steps of a workflow file, each as its non-blank lines.

 @param lines - workflow file lines

 @returns one entry per `run:` key, holding the script's lines

 @example
 ```ts
 runScripts(['- run: |', '    echo a']);
 ```
 */
export function runScripts(lines: readonly string[]): readonly (readonly string[])[] {
  return lines.flatMap(function scriptAt(
    line,
    index
  ): (readonly string[])[] {
    /**
     The line without its indentation.
     */
    const trimmed = line.trim();
    if (!trimmed.startsWith('run:'))
      return [];
    /**
     Text after the key on the same line.
     */
    const inline = trimmed.slice('run:'.length)
      .trim();
    if ((inline !== '') && (inline !== '|'))
      return [[inline]];
    return [
      blockBelow({
        lines,
        at: index,
        stopsAt: indentOf(line) + 1
      })
        .map(function trim(entry) {
          return entry.trim();
        })
        .filter(function nonBlank(entry) {
          return entry !== '';
        }),
    ];
  });
}

/**
 One job of a workflow: its id and its lines, without the id line.
 */
export type JobBlock = {
  readonly id: string;
  readonly lines: readonly string[];
};

/**
 Split a workflow file into the jobs under its `jobs:` key.

 @param lines - workflow file lines

 @returns every job in file order

 @example
 ```ts
 jobBlocks(['jobs:', '  a:', '    runs-on: x', '  b:', '    runs-on: y']);
 ```
 */
export function jobBlocks(lines: readonly string[]): readonly JobBlock[] {
  /**
   Index of the `jobs:` key.
   */
  const jobsAt = lines.indexOf('jobs:');
  if (jobsAt === (-1))
    return [];
  /**
   Lines of the jobs section.
   */
  const section = blockBelow({
    lines,
    at: jobsAt,
    stopsAt: 1
  });
  return section.flatMap(function jobAt(
    line,
    index
  ): JobBlock[] {
    if ((indentOf(line) !== 2)
      || (!line.trim()
        .endsWith(':'))
      || line.trim()
        .startsWith('#'))
      return [];
    return [
      {
        id: line.trim()
          .slice(
          0,
          -1
          ),
        lines: blockBelow({
          lines: section,
          at: index,
          stopsAt: 3
        }),
      },
    ];
  });
}

/**
 Split the lines of a job into its steps: each starts at a `- ` at the indentation of the first one.

 @param job - lines of one job

 @returns the lines of each step, in order; empty for a job without a `steps:` key

 @example
 ```ts
 stepBlocks(['    steps:', '    - name: a', '      run: x']);
 ```
 */
export function stepBlocks(job: readonly string[]): readonly (readonly string[])[] {
  /**
   Index of the `steps:` key.
   */
  const stepsAt = job.findIndex(function isSteps(line) {
    return line.trim() === 'steps:';
  });
  if (stepsAt === (-1))
    return [];
  /**
   Lines of the steps section.
   */
  const section = blockBelow({
    lines: job,
    at: stepsAt,
    stopsAt: indentOf(job[stepsAt] ?? '')
  });
  /**
   Indentation of the dash of each step.
   */
  const dashIndent = indentOf(section.find(function isDash(line) {
    return line.trim()
      .startsWith('- ');
  }) ?? '');
  return section.reduce<string[][]>(
    function place(
      steps,
      line
    ) {
      if ((indentOf(line) === dashIndent)
        && line.trim()
          .startsWith('- '))
        steps.push([line]);
      else
        steps.at(-1)
          ?.push(line);
      return steps;
    },
    [],
  );
}
