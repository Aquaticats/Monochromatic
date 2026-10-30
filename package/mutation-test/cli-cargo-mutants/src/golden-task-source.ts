import { readFile } from 'node:fs/promises';

/**
 The `run` setting of a mise task and how the task runs it.
 */
export type TaskSource = {
  readonly source: string;
  readonly inline: boolean;
};

/**
 Triple quote that opens and closes a TOML literal multi-line string.
 */
const literalQuotes = "'''";

/**
 `shell` setting that makes a task run its `run` text as TypeScript source.
 */
const inlineShellSetting = 'shell = "node --input-type=module-typescript -e"';

/**
 Read the `run` text of a mise task.

 @param miseFile - path of the `mise.toml`
 
 @param task - task name
 
 @returns the text, unescaped from a TOML literal multi-line string or a single-line basic string, and whether the task
 runs it as TypeScript source
 
 @example
 ```ts
 await taskSource({ miseFile, task: 'mutation:processors' });
 ```
 */
export async function taskSource({
  miseFile,
  task,
}: {
  readonly miseFile: string;
  readonly task: string;
}): Promise<TaskSource> {
  /**
   File text from the task header on.
   */
  const fromHeader = (await readFile(
    miseFile,
    'utf8',
  ))
    .split(`[tasks."${task}"]`)[1] ?? '';
  /**
   The task section: the text up to the next task header.
   */
  const section = fromHeader.split('\n[tasks')[0] ?? '';
  /**
   Marker of the run line.
   */
  const runMarker = '\nrun = ';
  /**
   The run line and everything after it.
   */
  const fromRun = fromHeader.slice(fromHeader.indexOf(runMarker) + runMarker.length);
  /**
   Whether the task runs its text as TypeScript.
   */
  const inline = section.includes(inlineShellSetting);
  if (fromRun.startsWith(literalQuotes)) {
    /**
     Text after the opening quotes.
     */
    const body = fromRun.slice(literalQuotes.length);
    /**
     Text between the quotes.
     */
    const literal = body.slice(
      0,
      body.indexOf(literalQuotes),
    );
    return {
      source: literal.startsWith('\n') ? literal.slice(1) : literal,
      inline,
    };
  }
  /**
   The value of a single-line basic string: JSON strings use the same escapes for the characters the repo's tasks use.
   */
  const basic: unknown = JSON.parse(fromRun.slice(
    0,
    fromRun.indexOf('\n'),
  ));
  if ((typeof basic) !== 'string')
    throw new TypeError(`The run setting of ${task} in ${miseFile} is not a string.`);
  return {
    source: basic,
    inline,
  };
}
