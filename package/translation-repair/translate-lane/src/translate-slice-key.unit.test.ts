/**
 Tests for the cross-run key that decides when two runs are looking at the same
 slice.

 WHAT THESE PIN is the pair of promises the key has to keep at once. It must
 SEPARATE two questions that can have different answers, or one arm of a
 comparison reads the other's cached result and the two report as identical.
 And it must not separate anything else, or a settled corpus is discarded for
 nothing.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  TRANSLATE_SLICE_CACHE_VERSION,
  translateRunShape,
  translateSliceKey,
} from '../dist/final/node/index.mjs';

/**
 Roster both sides of every comparison share.
 */
const MODELS = {
  translatorModelIds: ['hf:cat/Cat-A',
    'hf:cat/Cat-B',] as never,
  judgeModelIds: ['hf:cat/Cat-A',
    'hf:cat/Cat-B',
    'hf:cat/Cat-C',] as never,
};

/**
 Run shape every case keys against.
 */
const RUN_SHAPE = translateRunShape({ models: MODELS, },);

/**
 Slice original every case keys.
 */
const SOURCE_TEXT = '小猫在窗台上睡觉。\n';

/**
 Translation already sitting in the archive for it.
 */
const INCUMBENT_TEXT = 'The cat sleeps on the windowsill.\n';

/**
 Key a windowless slice hashes to, pinned rather than recomputed.

 WHY A LITERAL RATHER THAN A COMPARISON. Every other case here asks whether two
 keys agree, and a change that moved BOTH sides would pass all of them while
 discarding every settled slice in the pinned corpus, since a resumed record is
 found by this exact string. Only a value written down outside the code catches
 that. It moves when {@link TRANSLATE_SLICE_CACHE_VERSION} moves, which is the
 intended signal: a bump means the corpus is deliberately being rebought.

 MOVED on 2026-08-22 when version went to 6 for judge line-structure policy,
 then on 2026-08-28 when version went to 7 for target-authoritative contributor
 spelling in metadata comments and version 8 for body contributor public
 identities. Version 9 adds deterministic complete-form contributor floor.
 Version 10 replaces identical decline retry with distinct challenge task.
 Version 11 continues absent passages from latest rejection evidence.
 Version 12 excludes archive fallback that fails deterministic source floor.
 Version 13 ships a withheld archive slice past a declined follow-up round.
 Version 14 refuses disputed archive wording and keys the dispute note.
 Version 15 carries every floor, sheet and quorum change made after 14 landed.
 Version 16 reads the typography fold and the guillemet floor from the page parse (n156).
 Version 17 reads a shortcut reference and an address label as no prose and floors a refused candidate that carries a
 guillemet (n159).
 Version 18 states in the house tense rule, in the sentence that puts the life in the past, that quoted words keep the
 speaker's own tense (n166).
 Version 19 lists the narration between marks of different single-quotation pairs among the tense candidates a judge is shown.
 Version 20 scopes reader protection to the account of how a death or an attempt happened and cuts translators' notes from
 the judges (the XingZ6015 read).
 Version 21 cuts a translators' note alone on a quoted line with its line, where the judges saw a bare quote line.
 Version 22 refuses a candidate that respells a Han name the archive spells inside a sentence, and the identity lines
 list those spellings (zheermao101, run zheermao16).
 Version 22 also cuts the translators' notes from the surrounding existing translation the candidate selection slate shows,
 which a heading that names the original in its own words had left whole, and from the translation the coverage sheet shows.
 These questions changed without input fields.
 Version 23 has the selection ballot and the coverage judge give their reasons before their votes.
 Version 24 declares the answer schemas of the translate lane in alphabetical order (ledger B762).
 Version 25 gathers under the quorum rule that lets a short bench close one voice fewer than every live seat once the
 straggler grace ends (ledger B816).
 Version 26 moves the fence length and the rendered-break counts out of the system text into the user turn (R4 of the
 third review of the Claude route).
 Version 27 reads a reply value that differs from a listed enum or const value only in
 capitalization as the listed value.

 The roster feeding {@link RUN_SHAPE} is invented, so a production roster change
 leaves this alone.
 */
const LEGACY_WINDOWLESS_KEY = '3aafcd70beafc75dd298bc1c1f9ef3aa58bc95bbe69cbfe076d2eddb55e3bb44';

/**
 One slice's key, with whatever this case wants to vary.

 @param neighbouringSourceText - wider window, absent for an ordinary run

 @returns Key for that slice under that window

 @example
 ```ts
 const key = keyFor({},);
 ```
 */
function keyFor(
  { neighbouringSourceText, }: { readonly neighbouringSourceText?: string; },
): string {
  return translateSliceKey({
    runShape: RUN_SHAPE,
    sourceText: SOURCE_TEXT,
    incumbentText: INCUMBENT_TEXT,
    incumbentKind: 'present',
    lineStructured: false,
    ...((neighbouringSourceText === undefined)
      ? {}
      : { neighbouringSourceText, }),
  },);
}

await describe({
  name: translateSliceKey.name,
  children: [
    it({
      name: 'IS AT VERSION 27: the reader of a model\'s reply takes a value that differs from a listed enum or const '
        + 'value only in capitalization as the listed value, and the key hashes the version, the shape of the run and '
        + 'the texts, not the reader, so only this number retires a record written under the reader before',
      fn: async () => {
        expect(TRANSLATE_SLICE_CACHE_VERSION,).toBe(27,);
      },
    },),
    it({
      name: 'SEPARATES a run shown the pages the original cites from one shown none, and keeps the run '
        + 'shape of an entry that cites nothing (class thirty-six, 2026-09-16)',
      fn: async () => {
        expect(translateRunShape({
          models: MODELS,
          referenceContext: '- reference 1 https://blog.example/mittens ("In memory of Mittens"): Mittens had an older sister who was also a tabby.',
        },),).not.toBe(RUN_SHAPE,);
        expect(translateRunShape({
          models: MODELS,
          referenceContext: '',
        },),).toBe(RUN_SHAPE,);
      },
    },),
    it({
      name: 'SEPARATES a run whose entry records a death, and the subject it names, from one that records none, and keeps the run shape of an entry that records none',
      fn: async () => {
        /**
         The shape of an entry that records a death and names its subject.
         */
        const named = translateRunShape({
          models: MODELS,
          lifeEnded: true,
          subjectNames: ['Mittens',],
        },);
        expect(named,).not.toBe(RUN_SHAPE,);
        expect(translateRunShape({
          models: MODELS,
          lifeEnded: true,
          subjectNames: ['Whiskers',],
        },),).not.toBe(named,);
        expect(translateRunShape({
          models: MODELS,
          lifeEnded: false,
        },),).toBe(RUN_SHAPE,);
      },
    },),
    it({
      name: 'SEPARATES a run whose subject carries a declared alias from one that names the subject alone, since the tense candidates read every name',
      fn: async () => {
        expect(translateRunShape({
          models: MODELS,
          lifeEnded: true,
          subjectNames: [
            'Mittens',
            'Whiskers',
          ],
        },),).not.toBe(translateRunShape({
          models: MODELS,
          lifeEnded: true,
          subjectNames: ['Mittens',],
        },),);
      },
    },),
    it({
      name: 'SEPARATES a slice whose translators were shown an attested detail from the same slice '
        + 'without one, so the lane written before the writer sheet carried them is not served back '
        + '(class thirty-nine, 2026-09-16)',
      fn: async () => {
        /**
         Reference line every arm shares.
         */
        const referenceContext = '- reference 1 https://blog.example/mittens ("In memory of Mittens"): Mittens had an older sister who was also a tabby.';
        expect(translateRunShape({
          models: MODELS,
          referenceContext,
          attestedLines: ['- attested: the ARCHIVE\'s "who is also a tabby" is stated by reference 1 ("also a tabby"), 3 of 4 voices checked word for word',],
        },),).not.toBe(translateRunShape({
          models: MODELS,
          referenceContext,
        },),);
        expect(translateRunShape({
          models: MODELS,
          attestedLines: [],
        },),).toBe(RUN_SHAPE,);
      },
    },),
    it({
      name: 'SEPARATES a slice judged with the neighbouring original from the same slice judged '
        + 'without it, which is what stops the two arms of the window trial sharing a cached answer and '
        + 'reporting a window change as having made no difference',
      fn: async () => {
        expect(keyFor({ neighbouringSourceText: '她看着外面的鸟。\n', },),)
          .not
          .toBe(keyFor({},),);
      },
    },),
    it({
      name: 'gives an ABSENT window and an EMPTY one the same key, so a caller that computes the '
        + 'neighbours of a lone slice and gets nothing does not discard the entry a caller that '
        + 'never asked would have used',
      fn: async () => {
        expect(keyFor({ neighbouringSourceText: '', },),).toBe(keyFor({},),);
      },
    },),
    it({
      name: 'separates two DIFFERENT windows, since a judge shown one neighbour can answer '
        + 'differently from one shown another',
      fn: async () => {
        expect(keyFor({ neighbouringSourceText: '她看着外面的鸟。\n', },),)
          .not
          .toBe(keyFor({ neighbouringSourceText: '傍晚她回到炉火旁。\n', },),);
      },
    },),
    it({
      name: 'SEPARATES FRONT MATTER ROLE from identical Markdown text because prompts and validators differ',
      fn: async () => {
        expect(translateSliceKey({
          runShape: RUN_SHAPE,
          sourceText: SOURCE_TEXT,
          incumbentText: INCUMBENT_TEXT,
          incumbentKind: 'present',
          syntax: 'front-matter',
          lineStructured: false,
        },),).not.toBe(keyFor({},),);
      },
    },),
    it({
      name: 'keys the same slice to the same string when nothing varies, which is the whole point '
        + 'of the key and the thing every separation case in this file is measured against',
      fn: async () => {
        expect(keyFor({},),).toBe(keyFor({},),);
      },
    },),
    it({
      name: 'treats an EXPLICITLY passed `undefined` as absence, reaching the function rather than '
        + 'being stripped by a caller: every other case here goes through a helper that removes '
        + 'the property, so without this one nothing tests what the function does with it',
      fn: async () => {
        /**
         Argument a JavaScript caller can build and TypeScript refuses.

         `exactOptionalPropertyTypes` rejects this property as `undefined`,
         which is a real guard and is why the cast is here rather than the
         type being loosened to admit it. What the guard does not reach is an
         untyped caller, a spread of a partly filled record, or a wrapper
         compiled before this property existed, and each of those hands the
         function exactly this object. What it does with it is the runtime
         behaviour under test.
         */
        const explicitlyAbsent = {
          runShape: RUN_SHAPE,
          sourceText: SOURCE_TEXT,
          incumbentText: INCUMBENT_TEXT,
          incumbentKind: 'present',
          lineStructured: false,
          neighbouringSourceText: undefined,
        } as unknown as Parameters<typeof translateSliceKey>[0];

        expect(translateSliceKey(explicitlyAbsent,),).toBe(keyFor({},),);
      },
    },),
    it({
      name: 'HASHES A WINDOWLESS SLICE TO A PINNED STRING, which is what stands between a change to this '
        + 'serialization and the silent rebuying of every settled slice in the corpus. A deliberate cache-version '
        + 'bump moves this value and should; an accidental one is what this catches',
      fn: async () => {
        expect(keyFor({},),).toBe(LEGACY_WINDOWLESS_KEY,);
      },
    },),
  ],
},);
