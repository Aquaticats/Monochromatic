/**
 Node builtins reached by the members used, never through a default namespace.
 
 The repository standard prefers named imports, and every other package here
 spells `node:path` that way (`import { dirname, join, } from 'node:path'`, as
 in `package/logger/core/src/sink/file.ts`).
 A namespace import is the exception this suite removes: two dialects leave a
 reviewer needing to know which package a file lives in before judging an
 import line, and the second spelling drifts back in one copy at a time.
 `import process from 'node:process'` is the repository-wide idiom and stays.
 
 The suite reads source text rather than the built bundle, because a bundler
 decides for itself whether to keep a namespace import.
 
 @module
 */

import {
  readdirSync,
  readFileSync,
} from 'node:fs';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Source scanning

/**
 Directory holding the modules under test; this suite sits beside them.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 Suffix of the modules this suite reads.
 */
const SOURCE_FILE_SUFFIX = '.ts';

/* oxlint-disable no-restricted-syntax/no-regex -- The scanner reads source text to find one fixed access form, an import declaration, which no string API expresses once its clause wraps over lines. Every quantifier runs over a bounded negated character class with no nested repetition, the pattern is anchored per line, and the input is this package's own sources, tens of kilobytes in all. */
/**
 One import declaration whose specifier names a Node builtin.
 
 Group `clause` is what sits between `import` and `from`,
 so it is either a default or namespace binding,
 or a set of braces.
 Group `specifier` is the module name,
 which includes the `node:` protocol that marks a builtin.
 A clause wraps over several lines in this repository,
 and a negated character class crosses newlines,
 so one declaration still reads as one match.
 
 @example
 ```ts
 [..."import { join, } from 'node:path';".matchAll(NODE_IMPORT_PATTERN)]; // one match
 ```
 */
const NODE_IMPORT_PATTERN = /^import\s+(?<clause>[^;]*?)\s+from\s+'(?<specifier>node:[^']+)';/gmu;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 Specifier the repository imports as a default binding,
 so a default import of it is the standard rather than a finding.
 
 @example
 ```ts
 import process from 'node:process';
 ```
 */
const DEFAULT_IMPORTED_SPECIFIER = 'node:process';

/**
 Text that opens a set of named bindings,
 which is the only clause shape that imports members by name.
 
 @example
 ```ts
 '{'; // the clause of `import { join, } from 'node:path'`
 ```
 */
const NAMED_BINDINGS_OPENING = '{';

//endregion Source scanning

//region Findings

/**
 Collects every Node builtin one source text imports as a default binding.
 
 @param fileName - Name the finding names, so a failure points at a file.
 
 @param text - Source text to scan.
 
 @returns One line per default-imported builtin; empty when the text follows
 the standard.
 
 @example
 ```ts
 defaultNodeImportFindings({
   fileName: 'config-dir.ts',
   text: "import path from 'node:path';",
 }); // ['config-dir.ts: node:path imported by default']
 ```
 */
function defaultNodeImportFindings({
  fileName,
  text,
}: {
  readonly fileName: string;
  readonly text: string;
},): string[] {
  /**
   Findings for this text, in the order the declarations appear.
   */
  const findings: string[] = [];
  for (const match of text.matchAll(NODE_IMPORT_PATTERN,)) {
    /**
     Clause between `import` and `from`,
     which decides whether the declaration imports members.
     */
    const clause = match.groups
      ?.clause;
    /**
     Module the declaration imports.
     */
    const specifier = match.groups
      ?.specifier;
    if ((clause === undefined) || (specifier === undefined)) {
      throw new Error(`the import pattern matched without capturing ${fileName}`,);
    }
    if (specifier === DEFAULT_IMPORTED_SPECIFIER)
      continue;
    if (clause.trim().startsWith(NAMED_BINDINGS_OPENING,))
      continue;
    findings.push(`${fileName}: ${specifier} imported by default`,);
  }
  return findings;
}

//endregion Findings

await describe({
  name: defaultNodeImportFindings.name,
  children: [
    it({
      name: 'reports a default import of a Node builtin',
      fn: async () => {
        expect(defaultNodeImportFindings({
          fileName: 'sample.ts',
          text: [
            "import path from 'node:path';",
            '',
            'path.join(\'a\', \'b\',);',
          ].join('\n',),
        },),).toStrictEqual(['sample.ts: node:path imported by default',],);
      },
    },),

    it({
      name: 'accepts named bindings, a side-effect import and the node:process default',
      fn: async () => {
        expect(defaultNodeImportFindings({
          fileName: 'sample.ts',
          text: [
            "import { dirname, join, } from 'node:path';",
            'import {',
            '  readFileSync,',
            "} from 'node:fs';",
            "import process from 'node:process';",
            "import './side-effect.ts';",
          ].join('\n',),
        },),).toStrictEqual([],);
      },
    },),

    it({
      name: 'finds no default-imported builtin in this package\'s sources',
      fn: async () => {
        /**
         Findings across every module of this package.
         */
        const findings = readdirSync(SOURCE_DIRECTORY,)
          .filter(function isSourceFile(entry: string,): boolean {
            return entry.endsWith(SOURCE_FILE_SUFFIX,);
          },)
          .flatMap(function findingsOfEntry(entry: string,): string[] {
            return defaultNodeImportFindings({
              fileName: entry,
              text: readFileSync(
                join(
                  SOURCE_DIRECTORY,
                  entry,
                ),
                'utf8',
              ),
            },);
          },);

        expect(findings,).toStrictEqual([],);
      },
    },),
  ],
},);
