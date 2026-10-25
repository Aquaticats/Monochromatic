import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  findCategoricalDismissal,
  findTrailingQuestion,
  findUncertainty,
  NO_MATCH,
  stripNonProseRegions,
} from './uncertainty.ts';

await describe({
  name: 'uncertainty detection',
  children: [
    describe({
      name: findUncertainty.name,
      children: [
        it({
          name: 'matches a hedge phrase in prose',
          fn: async () => {
            expect(findUncertainty('This is probably the fix.',),).toEqual({ phrase: 'probably', },);
          },
        },),
        it({
          name: 'returns NO_MATCH when prose has no hedges',
          fn: async () => {
            expect(findUncertainty('A fact stated plainly.',),).toBe(NO_MATCH,);
          },
        },),
        it({
          name: 'matches an admitted gap in each spoken form',
          fn: async () => {
            const admissions = [
              ["I didn't check whether the release job runs the tests.", "i didn't check",],
              ['I did not check which controller the board uses.', 'i did not check',],
              ["I haven't verified this, and CI has to cover it.", "i haven't verified",],
              ['I have not read the final reports yet.', 'i have not read',],
              ["I haven't tested an actual reboot.", "i haven't tested",],
              ["I haven't confirmed that.", "i haven't confirmed",],
              ['I did not run the combined task.', 'i did not run',],
              ['Not verified by me: the agent says the lint run exits 1.', 'not verified',],
              ['Not checked: whether FRC collides with a retired code.', 'not checked',],
              ['Not tested yet: two commits running at once.', 'not tested',],
            ] as const;

            for (const [text, phrase,] of admissions) {
              expect(findUncertainty(text,),).toEqual({ phrase, },);
            }
          },
        },),
        it({
          name: 'matches an admitted gap in an evidence-gathering step other than checking',
          fn: async () => {
            const admissions = [
              ["It isn't blank-free. I didn't try to remove it.", "i didn't try",],
              ["The lag is unknown because I didn't measure the capture-to-file delay.", "i didn't measure",],
              ['I did not measure the loop that the savings come from.', 'i did not measure',],
              ["I haven't measured the net effect on collisions.", "i haven't measured",],
              ['I have not tried building it with Bazel.', 'i have not tried',],
              ["I haven't looked into the logger error.", "i haven't looked",],
              ['I did not look at the popup screenshots.', 'i did not look',],
              ["I haven't reproduced it with two hooks running at once.", "i haven't reproduced",],
              ['I did not rerun scoped mutation for those tests.', 'i did not rerun',],
              ["I didn't re-run the failing guards.", "i didn't re-run",],
              ["The pointer rested over no character, but I haven't shown that.", "i haven't shown",],
              ['Test gap: no regression test can show the blank.', 'test gap',],
              ['Not viewed: the 14 popups, only the log.', 'not viewed',],
            ] as const;

            for (const [text, phrase,] of admissions) {
              expect(findUncertainty(text,),).toEqual({ phrase, },);
            }
          },
        },),
        it({
          name: 'matches an admitted gap written with a curly apostrophe',
          fn: async () => {
            expect(findUncertainty('I didn’t check whether the release can save that cache.',),)
              .toEqual({ phrase: "i didn't check", },);
          },
        },),
        it({
          name: 'leaves a negation that is not an admitted gap alone',
          fn: async () => {
            expect(findUncertainty("I haven't deleted anything, and I didn't touch the other session's files.",),)
              .toBe(NO_MATCH,);
          },
        },),
        it({
          name: 'leaves the bare adjectives untested, unverified, unexplained and unmeasured alone',
          fn: async () => {
            expect(findUncertainty('The merge is untested and the version job is unverified.',),).toBe(NO_MATCH,);
            expect(findUncertainty('The file shows an unexplained abnormality and an unmeasured cost.',),)
              .toBe(NO_MATCH,);
            expect(findUncertainty("I didn't touch the tree and I tried the build, which passed.",),).toBe(NO_MATCH,);
          },
        },),
      ],
    },),
    describe({
      name: findCategoricalDismissal.name,
      children: [
        it({
          name: 'flags uncited "project doesn\'t use" dismissal',
          fn: async () => {
            expect(
              findCategoricalDismissal(
                "All JSX rules: project doesn't use JSX.",
              ),
            )
              .toEqual({ phrase: "project doesn't use", },);
          },
        },),
        it({
          name: 'flags uncited "we don\'t use" dismissal',
          fn: async () => {
            expect(findCategoricalDismissal("Skip; we don't use enums.",),)
              .toEqual({ phrase: "we don't use", },);
          },
        },),
        it({
          name: 'flags uncited "is already handled by" dismissal',
          fn: async () => {
            expect(
              findCategoricalDismissal(
                'Skip; operator-linebreak is already handled by dprint.',
              ),
            )
              .toEqual({ phrase: 'is already handled by', },);
          },
        },),
        it({
          name: 'allows dismissal when the same line cites a file path',
          fn: async () => {
            expect(
              findCategoricalDismissal(
                'Skip; is already handled by dprint (`package/config/dprint/index.json:103`).',
              ),
            )
              .toBe(NO_MATCH,);
          },
        },),
        it({
          name: 'allows dismissal when the same line names AGENTS.md',
          fn: async () => {
            expect(
              findCategoricalDismissal(
                "Skip; the project doesn't use this (AGENTS.md bans it).",
              ),
            )
              .toBe(NO_MATCH,);
          },
        },),
        it({
          name: 'does not blanket-allow dismissals when citations are on other lines',
          fn: async () => {
            const text = [
              'A bullet with a real citation: see `package/foo/bar.ts:42`.',
              "A separate bullet; project doesn't use JSX.",
            ]
              .join('\n',);
            expect(findCategoricalDismissal(text,),)
              .toEqual({ phrase: "project doesn't use", },);
          },
        },),
        it({
          name: 'returns NO_MATCH when prose has no dismissal',
          fn: async () => {
            expect(
              findCategoricalDismissal('Just a factual statement, no dismissal.',),
            )
              .toBe(NO_MATCH,);
          },
        },),
        it({
          name: 'flags "codebase doesn\'t have" without citation',
          fn: async () => {
            expect(
              findCategoricalDismissal(
                "Skip; the codebase doesn't have any Vue files.",
              ),
            )
              .toEqual({ phrase: "the codebase doesn't have", },);
          },
        },),
        it({
          name: 'flags "doesn\'t apply here" without citation',
          fn: async () => {
            expect(
              findCategoricalDismissal(
                "Skip; that rule doesn't apply here.",
              ),
            )
              .toEqual({ phrase: "doesn't apply here", },);
          },
        },),
        it({
          name: 'allows dismissal accompanied by a line-number suffix',
          fn: async () => {
            expect(
              findCategoricalDismissal(
                'Skip; is already covered by dprint at line :103.',
              ),
            )
              .toBe(NO_MATCH,);
          },
        },),
      ],
    },),
    describe({
      name: findTrailingQuestion.name,
      children: [
        it({
          name: 'flags a trailing question',
          fn: async () => {
            expect(findTrailingQuestion('I finished. Want me to run tests?',),)
              .toEqual({ sentence: 'Want me to run tests?', },);
          },
        },),
        it({
          name: 'returns NO_MATCH when no trailing question',
          fn: async () => {
            expect(findTrailingQuestion('I finished. Tests pass.',),).toBe(
              NO_MATCH,
            );
          },
        },),
        it({
          name: 'flags a question that spans the whole message (sentence start at 0)',
          fn: async () => {
            expect(findTrailingQuestion('Want me to commit?',),)
              .toEqual({ sentence: 'Want me to commit?', },);
          },
        },),
        it({
          name: 'finds the sentence start after a terminator and multiple spaces',
          fn: async () => {
            expect(findTrailingQuestion('Done!   Should I proceed?',),)
              .toEqual({ sentence: 'Should I proceed?', },);
          },
        },),
        it({
          name: 'ignores a rhetorical trailing question',
          fn: async () => {
            expect(findTrailingQuestion('I fixed it. Why would anyone object?',),).toBe(
              NO_MATCH,
            );
          },
        },),
      ],
    },),
    describe({
      name: stripNonProseRegions.name,
      children: [
        it({
          name: 'strips fenced code blocks',
          fn: async () => {
            const cleaned = stripNonProseRegions(
              'before\n```js\nmaybe();\n```\nafter',
            );
            expect(cleaned.includes('maybe',),).toBe(false,);
          },
        },),
        it({
          name: 'strips inline code',
          fn: async () => {
            const cleaned = stripNonProseRegions('use `maybe` here',);
            expect(cleaned.includes('maybe',),).toBe(false,);
          },
        },),
        it({
          name: 'strips a single-quoted string',
          fn: async () => {
            const cleaned = stripNonProseRegions("the word 'maybe' appears here",);
            expect(cleaned.includes('maybe',),).toBe(false,);
          },
        },),
        it({
          name: 'keeps text between two contractions on the same line',
          fn: async () => {
            const cleaned = stripNonProseRegions("The workflow's comments say it works, but I didn't check.",);
            expect(cleaned.includes('comments say it works',),).toBe(true,);
            expect(findUncertainty(cleaned,),).toEqual({ phrase: "i didn't check", },);
          },
        },),
        it({
          name: 'finds a hedge between two contractions, which a paired apostrophe used to hide',
          fn: async () => {
            const prose = stripNonProseRegions("It doesn't build, probably because the cache isn't warm.",);
            expect(findUncertainty(prose,),).toEqual({ phrase: 'probably', },);
          },
        },),
        it({
          name: 'strips a quote that opens or closes at the very edge of the text',
          fn: async () => {
            const cleaned = stripNonProseRegions("'maybe' is the word, and so is 'perhaps'",);
            expect(cleaned.includes('maybe',),).toBe(false,);
            expect(cleaned.includes('perhaps',),).toBe(false,);
          },
        },),
        it({
          name: 'still strips a quote that opens after a contraction',
          fn: async () => {
            const cleaned = stripNonProseRegions("It doesn't mean 'maybe' here.",);
            expect(cleaned.includes('maybe',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
