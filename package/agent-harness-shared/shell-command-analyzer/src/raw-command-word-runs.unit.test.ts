import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rawCommandWordRuns, } from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

//region Helpers

/**
 Copies the word runs into plain arrays, so a failed comparison prints the words read.

 @param source - shell source text to read

 @returns each run as an array of its words

 @example
 ```ts
 runsOf('cd x && bun test'); // [['cd', 'x'], ['bun', 'test']]
 ```
 */
function runsOf(source: string,): string[][] {
  return rawCommandWordRuns({ source, },)
    .map(function wordsOfRun(words,): string[] {
      return [...words,];
    },);
}

/**
 Collects the runs whose words are exactly the given ones.

 @param source - shell source text to read

 @param words - words the run has to hold, in order

 @returns whether one run reads exactly those words

 @example
 ```ts
 hasRun({ source: 'cat <<EOF\nwhile pgrep x\nEOF\n"', words: ['while', 'pgrep', 'x'] }); // true
 ```
 */
function hasRun({
  source,
  words,
}: {
  readonly source: string;
  readonly words: readonly string[];
},): boolean {
  return runsOf(source,)
    .some(function runHoldsExactly(wordsOfRun,): boolean {
      return (wordsOfRun.length === words.length)
        && (wordsOfRun.every(function sameWord(word, index,): boolean {
          return word === words[index];
        },));
    },);
}

//endregion Helpers

await describe({
  name: 'raw command word runs',
  children: [
    describe({
      name: rawCommandWordRuns.name,
      children: [
        it({
          name: 'splits commands on the separators Bash reads',
          fn: async function testSeparators() {
            expect(runsOf('cd x && bun test',),).toEqual([
              [
                'cd',
                'x',
              ],
              [
                'bun',
                'test',
              ],
            ],);
          },
        },),
        it({
          name: 'joins a word a line continuation splits',
          fn: async function testLineContinuation() {
            expect(runsOf('pg\\\nrep -f x',),).toEqual([
              [
                'pgrep',
                '-f',
                'x',
              ],
            ],);
          },
        },),
        it({
          name: 'reads an ANSI-C quoted word as the text it spells',
          fn: async function testAnsiCQuote() {
            expect(runsOf("while $'pgrep' -f x",),).toEqual([
              [
                'while',
                'pgrep',
                '-f',
                'x',
              ],
            ],);
          },
        },),
        it({
          name: 'keeps a redirection that names a descriptor in one word',
          fn: async function testDescriptorRedirection() {
            expect(runsOf('while >&2 pgrep -f x',),).toEqual([
              [
                'while',
                '>&2',
                'pgrep',
                '-f',
                'x',
              ],
            ],);
          },
        },),
        it({
          name: 'keeps a joined redirection in one word',
          fn: async function testJoinedRedirection() {
            expect(runsOf('while &>/dev/null pgrep -f x',),).toEqual([
              [
                'while',
                '&>/dev/null',
                'pgrep',
                '-f',
                'x',
              ],
            ],);
          },
        },),
        it({
          name: 'reads the commands after a quote the source never closes',
          fn: async function testUnclosedQuote() {
            expect(hasRun({
              source: "cat <<'EOF'\ndon't\nEOF\nwhile pgrep x; do sleep 1; done \"",
              words: [
                'while',
                'pgrep',
                'x',
              ],
            },),).toBe(true,);
          },
        },),
        it({
          name: 'reads the commands after a stray quote in an unparsable word',
          fn: async function testStrayQuote() {
            expect(hasRun({
              source: 'echo don\'t && while pgrep x; do sleep 1; done "',
              words: [
                'while',
                'pgrep',
                'x',
              ],
            },),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
