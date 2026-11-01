/**
 Guards class one hundred fifteen (yingying10, 2026-09-24): a translate
 candidate turned the original's linked title into plain words followed by
 the bare destination in parentheses, and every floor passed it, since the
 destination floor reads the bare URL as an autolink and the declared link
 name floor (class one hundred fourteen) is silent where the rendering
 carries no link under the href. It reached the slate and drew a ballot. A
 rendering that keeps a source link's destination but no longer carries it
 as a worded link is refused before any judge reads it.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { validateTranslatedSlice, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { floorTranslateVoices, } from '../dist/final/node/index.mjs';

/**
 Logger the floor writes its withheld lines to, which no case here reads.
 */
const l = tagged({ tag: 'unwrapped-link-test', },);

/**
 Original definition linking a post by its title.
 */
const SOURCE = '[^2]: [窗台上的午睡](https://example.invalid/windowsill-nap.html)';

/**
 Rendering keeping the title as the link's words.
 */
const LINKED = '[^2]: [A Nap on the Windowsill](https://example.invalid/windowsill-nap.html)';

/**
 Rendering with the title as plain words and the destination bare after it.
 */
const UNWRAPPED = '[^2]: A Nap on the Windowsill (https://example.invalid/windowsill-nap.html)';

/**
 Rendering whose link words became the destination itself.
 */
const URL_WORDED = '[^2]: A Nap on the Windowsill [https://example.invalid/windowsill-nap.html](https://example.invalid/windowsill-nap.html)';

/**
 Original linking the post and ending on an opening tag without its close,
 which the strict grammar refuses, so the validator reads it only through the
 floors that need no grammar.
 */
const TORN_SOURCE = `猫的博客：[窗台上的午睡](https://example.invalid/windowsill-nap.html)

<summary>猫的请求`;

/**
 Original linking the post that the strict grammar reads.
 */
const WHOLE_SOURCE = '猫的博客：[窗台上的午睡](https://example.invalid/windowsill-nap.html)';

/**
 Rendering keeping the link, torn the way its original is.
 */
const TORN_LINKED = `The cat's blog: [A Nap on the Windowsill](https://example.invalid/windowsill-nap.html)

<summary>The cat's requests`;

/**
 Rendering unwrapping the link, torn the way its original is.
 */
const TORN_UNWRAPPED = `The cat's blog: A Nap on the Windowsill (https://example.invalid/windowsill-nap.html)

<summary>The cat's requests`;

/**
 Rendering with a longer bare address, torn the way its original is.
 */
const TORN_LONGER = `The cat's blog: A Nap on the Windowsill (https://example.invalid/windowsill-nap.html2)

<summary>The cat's requests`;

/**
 Words of the unwrapped-link finding.
 */
const UNWRAPPED_FINDING = 'carries that destination without words linked to it';

await describe({
  name: 'a source link rendered as words and a bare destination (class one hundred fifteen)',
  children: [
    it({
      name: 'REFUSES a rendering that unwraps the link, and names the destination',
      fn: async () => {
        /**
         Verdict on the unwrapped rendering.
         */
        const verdict = validateTranslatedSlice({
          sourceText: SOURCE,
          candidateText: UNWRAPPED,
          pageText: LINKED,
        },);
        expect(verdict.kind,).toBe('invalid',);
        if (verdict.kind !== 'invalid')
          throw new Error('unreachable',);
        expect(verdict.findings.join('\n',),).toContain('https://example.invalid/windowsill-nap.html',);
      },
    },),
    it({
      name: 'LEAVES A LONGER ADDRESS TO THE DESTINATION FLOOR (ledger B23): one that only begins with the original\'s '
        + 'is not that destination kept, so the finding does not say it is',
      fn: async () => {
        /**
         Verdict on a rendering whose bare address runs on past the original's.
         */
        const verdict = validateTranslatedSlice({
          sourceText: SOURCE,
          candidateText: '[^2]: A Nap on the Windowsill (https://example.invalid/windowsill-nap.html2)',
          pageText: LINKED,
        },);
        expect(verdict.kind,).toBe('invalid',);
        if (verdict.kind !== 'invalid')
          throw new Error('unreachable',);
        expect(verdict.findings.join('\n',),).not.toContain(UNWRAPPED_FINDING,);
      },
    },),
    it({
      name: 'TEARS BOTH SIDES of the plain-markdown cases: the strict grammar refuses the rendering and the original',
      fn: async () => {
        // The positive control for the two cases after it. A rendering the
        // strict grammar reads, or an original it reads, would send those
        // cases through the strict path and prove nothing about the fallback.
        /**
         Verdict on the torn rendering against an original the grammar reads.
         */
        const candidateTorn = validateTranslatedSlice({
          sourceText: WHOLE_SOURCE,
          candidateText: TORN_LINKED,
        },);
        expect(candidateTorn.kind,).toBe('invalid',);
        if (candidateTorn.kind !== 'invalid')
          throw new Error('unreachable',);
        expect(candidateTorn.findings.join('\n',),).toContain('could not be parsed as Markdown',);
        expect(validateTranslatedSlice({
          sourceText: TORN_SOURCE,
          candidateText: TORN_LINKED,
        },).kind,).toBe('unknown',);
      },
    },),
    it({
      name: 'REFUSES AN UNWRAP IN A RENDERING THE STRICT GRAMMAR REFUSES (ledger B23), reading its destinations under '
        + 'plain markdown, since where the original is refused too nothing else would',
      fn: async () => {
        /**
         Verdict on the torn unwrapped rendering.
         */
        const verdict = validateTranslatedSlice({
          sourceText: TORN_SOURCE,
          candidateText: TORN_UNWRAPPED,
        },);
        expect(verdict.kind,).toBe('invalid',);
        if (verdict.kind !== 'invalid')
          throw new Error('unreachable',);
        expect(verdict.findings.join('\n',),).toContain(UNWRAPPED_FINDING,);
      },
    },),
    it({
      name: 'LEAVES A LONGER ADDRESS UNREAD AS THE ORIGINAL\'S under plain markdown too',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: TORN_SOURCE,
          candidateText: TORN_LONGER,
        },).kind,).toBe('unknown',);
      },
    },),
    it({
      name: 'REFUSES a rendering whose link words became the destination',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: SOURCE,
          candidateText: URL_WORDED,
          pageText: LINKED,
        },).kind,).toBe('invalid',);
      },
    },),
    it({
      name: 'REFUSES a rendering whose link words show a reader nothing, the title written as plain words beside '
        + 'a link of a zero-width space, a Hangul filler, or both among spaces, exactly as it refuses a link of '
        + 'spaces: the link is unwrapped all the same',
      fn: async () => {
        /**
         Rendering whose link carries the given words after the title in plain words.

         @param words - link words

         @returns Verdict on it

         @example
         ```ts
         const verdict = verdictWith('\u{200B}',);
         ```
         */
        function verdictWith(words: string,): ReturnType<typeof validateTranslatedSlice> {
          return validateTranslatedSlice({
            sourceText: SOURCE,
            candidateText: `[^2]: A Nap on the Windowsill [${words}](https://example.invalid/windowsill-nap.html)`,
            pageText: LINKED,
          },);
        }
        /** Verdict on a link of spaces, which the floor already refuses. */
        const spaced = verdictWith('   ',);
        expect(spaced,).toEqual({
          kind: 'invalid',
          findings: [
            'The ORIGINAL links words to https://example.invalid/windowsill-nap.html as '
              + '[words](https://example.invalid/windowsill-nap.html), but your translation carries that destination '
              + 'without words linked to it. Keep the link: put the rendered words inside the brackets and the '
              + 'destination in the parentheses right after them, as the ORIGINAL does.',
          ],
        },);
        expect([
          '\u{200B}',
          '\u{3164}',
          ' \u{200B}\u{3164} ',
        ].map(verdictWith,),).toEqual([
          spaced,
          spaced,
          spaced,
        ],);
      },
    },),
    it({
      name: 'ACCEPTS the worded link, and stays silent where the source link is worded by its own destination',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: SOURCE,
          candidateText: LINKED,
          pageText: LINKED,
        },).kind,).toBe('valid',);
        expect(validateTranslatedSlice({
          sourceText: '猫的博客：[https://example.invalid/](https://example.invalid/)',
          candidateText: 'The cat\'s blog: https://example.invalid/',
        },).kind,).toBe('valid',);
      },
    },),
    it({
      name: 'WITHHOLDS the unwrapped candidate from the translate slate',
      fn: async () => {
        /**
         Slate after the floor.
         */
        const floored = floorTranslateVoices({
          voices: [
            {
              modelId: 'hf:cat/Cat-A' as unknown as RosterModelId,
              value: { translation: UNWRAPPED, },
            },
            {
              modelId: 'hf:cat/Cat-B' as unknown as RosterModelId,
              value: { translation: LINKED, },
            },
          ],
          sourceText: SOURCE,
          incumbentText: LINKED,
          lineStructured: false,
          l,
        },);
        expect(floored.voices.map(function text(voice,): string {
          return voice.value.translation;
        },),).toEqual([LINKED,],);
      },
    },),
    it({
      name: 'LEAVES A CANDIDATE NESTED TOO DEEPLY TO READ to the floor that reads its shape, which refuses it, '
        + 'where plain markdown\'s refusal once ended the validation',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: SOURCE,
          candidateText: `${'>'.repeat(16_000,)} cat`,
        },),).toEqual({
          kind: 'invalid',
          findings: [
            'Your translation could not be parsed as Markdown: MdxParseError: MDX body refused to parse because it '
              + 'is nested too deeply to read: its container markers pass the bound of 256 at line 1, column 257; '
              + 'corpus documents compile as MDX upstream, so failure signals corruption or an unsupported '
              + 'construct.',
          ],
        },);
      },
    },),
  ],
},);
