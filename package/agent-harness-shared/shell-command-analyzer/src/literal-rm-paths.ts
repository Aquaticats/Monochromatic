/**
 Reads the `rm` commands whose paths the shell computes instead of spelling.
 
 This repository bans an `rm` command whose path is written through a variable,
 substitution, or arithmetic expression,
 because an unset or empty variable widens what gets deleted while the command
 text still looks narrow.
 The ban is about spelling rather than about effect:
 `rm --recursive /home/user/temp/agent/scratch` names exactly one path,
 and `rm -rf "$d"` names a path the reader cannot see.
 
 Every form the shared analyzer reads is read here too:
 program wrappers (`sudo`, `env`, `timeout`, `systemd-run`, `flock`, `nice`),
 command strings a shell or `eval` runs,
 `ssh` command lines,
 function bodies,
 commands the parser rejected,
 here-document bodies,
 commands `xargs` supplies paths to,
 and commands `find -exec`, `-execdir`, `-ok` and `-okdir` run.
 A form the analyzer does not follow stays unread, and the analyzer README
 names those forms.
 
 @module
 */

import {
  lineOfOffset,
  readIdentity,
  readOneSource,
  type SourceRead,
} from './literal-rm-command-reads.ts';
import type { LiteralRmPathFinding, } from './literal-rm-path-finding.ts';

//region Source walks

/**
 Reads a list of sources until every source they launch has been read.
 
 A source text already read under the same rule is not read twice,
 which is what makes a source that launches its own text terminate.
 
 @param pending - sources still to read, the outermost first
 
 @returns every finding the sources named, in reading order
 
 @example
 ```ts
 readSources([{ source: 'rm -rf "$d"', unwrittenPaths: false }]);
 ```
 */
function readSources(pending: readonly SourceRead[],): readonly LiteralRmPathFinding[] {
  /**
   Sources still to read, with the walk adding the ones each source launches.
   */
  const queue: SourceRead[] = [...pending,];
  /**
   Source texts already read, which ends a source that launches its own text.
   */
  const read = new Set<string>();
  /**
   Findings collected in reading order.
   */
  const findings: LiteralRmPathFinding[] = [];

  while (queue.length > 0) {
    /**
     Source text read next.
     */
    const current = queue.pop();
    if (current === undefined)
      break;
    if (read.has(readIdentity(current,),))
      continue;
    read.add(readIdentity(current,),);
    /**
     What this source named and launched.
     */
    const result = readOneSource({ read: current, },);
    findings.push(...result.findings,);
    queue.push(...result.reads,);
  }

  return findings;
}

//endregion Source walks

//region Public reads

/**
 Reads every `rm` in shell source that names its paths at run time.
 
 @param source - shell source text to read
 
 @returns one finding per offending command and word
 
 @example
 ```ts
 findLiteralRmPaths({ source: 'rm -rf "$d"', }); // [{ command: 'rm -rf "$d"', word: '"$d"' }]
 findLiteralRmPaths({ source: `rm -rf '$d'`, }); // []
 ```
 */
function findLiteralRmPaths({ source, }: { readonly source: string; },): readonly LiteralRmPathFinding[] {
  return readSources([{
    source,
    unwrittenPaths: false,
  },],);
}

/**
 Reads every `rm` in a script file's whole text that names its paths at run time.
 
 The text is read as a file rather than as one command string,
 so each finding names the file and the line its command starts on.
 
 @param path - file path the text came from, reported with every finding
 
 @param source - whole text of that file
 
 @returns one finding per offending command and word
 
 @example
 ```ts
 findLiteralRmPathsInScriptFile({ path: '/repo/clean.sh', source: 'rm -rf "$d"\n', });
 ```
 */
function findLiteralRmPathsInScriptFile({
  path,
  source,
}: {
  readonly path: string;
  readonly source: string;
},): readonly LiteralRmPathFinding[] {
  return readSources([{
    source,
    file: path,
    linesFrom(offset: number,): number {
      return lineOfOffset({
        text: source,
        offset,
      },);
    },
    unwrittenPaths: false,
  },],);
}

//endregion Public reads

export {
  findLiteralRmPaths,
  findLiteralRmPathsInScriptFile,
  readSources,
};
