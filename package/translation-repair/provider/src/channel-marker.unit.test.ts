/**
 Tests for removing a truncated provider channel marker ahead of JSON.

 The case that matters is the one the previous implementation could not
 handle. It matched the exact string `|>`, which is what survived on
 2026-08-12; on 2026-08-13 the surviving tail grew to `p|>` and `ep|>` and the
 exact match stopped firing, losing 21 of the 23 voices lost in that run
 window. The rule under test is therefore the SHAPE of a `<|word|>` tail
 rather than a vocabulary of known markers.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { stripChannelMarker, } from '../dist/final/node/index.mjs';

await describe({
  name: stripChannelMarker.name,
  children: [
    it({
      name: 'strips the two-character tail the first fix was written for, so '
        + 'the case that cost 507 schema-mismatches in one pass stays covered',
      fn: async () => {
        expect(stripChannelMarker({
          text: String.raw`|>{"count":2,"first":"Mittens"}`,
        },),).toStrictEqual({
          content: String.raw`{"count":2,"first":"Mittens"}`,
          marker: '|>',
        },);
      },
    },),

    it({
      name: 'strips the LONGER tails that broke the exact-match rule: `p|>` and '
        + '`ep|>` are what the provider left behind on 2026-08-13, and matching '
        + 'a vocabulary rather than a shape is what let them through',
      fn: async () => {
        expect(stripChannelMarker({
          text: String.raw`p|>{"count":2}`,
        },),).toStrictEqual({
          content: String.raw`{"count":2}`,
          marker: 'p|>',
        },);
        expect(stripChannelMarker({
          text: String.raw`ep|>{"count":2}`,
        },),).toStrictEqual({
          content: String.raw`{"count":2}`,
          marker: 'ep|>',
        },);
        expect(stripChannelMarker({
          text: String.raw`sep|>{"count":2}`,
        },),).toStrictEqual({
          content: String.raw`{"count":2}`,
          marker: 'sep|>',
        },);
      },
    },),

    it({
      name: 'strips a WHOLE marker, opening characters included, since a token '
        + 'that never got filtered at all is the same defect with none of it '
        + 'removed rather than a different one',
      fn: async () => {
        expect(stripChannelMarker({
          text: String.raw`<|im_sep|>{"count":2}`,
        },),).toStrictEqual({
          content: String.raw`{"count":2}`,
          marker: '<|im_sep|>',
        },);
      },
    },),

    it({
      name: 'REPORTS the fragment it removed, because the only reason the '
        + '2026-08-13 recurrence was diagnosable is that the raw opening had '
        + 'been recorded, and a silent strip loses that signal the next time '
        + 'the token filter changes shape',
      fn: async () => {
        expect(stripChannelMarker({
          text: String.raw`ep|>{"count":2}`,
        },).marker,).toBe('ep|>',);
      },
    },),

    it({
      name: 'leaves the marker in place when what follows is NOT a JSON value, '
        + 'so a reply that opens with those characters and then apologizes '
        + 'still fails to parse and reaches the refusal detector rather than '
        + 'being silently mended',
      fn: async () => {
        expect(stripChannelMarker({ text: '|> I cannot help with that.', },),)
          .toStrictEqual({
            content: '|> I cannot help with that.',
            marker: '',
          },);
        expect(stripChannelMarker({ text: 'p|> I cannot help with that.', },),)
          .toStrictEqual({
            content: 'p|> I cannot help with that.',
            marker: '',
          },);
      },
    },),

    it({
      name: 'refuses a leading fragment carrying a SPACE, which is what keeps '
        + 'this from degrading into "skip junk until the first brace": prose '
        + 'ahead of JSON must stay a parse failure',
      fn: async () => {
        expect(stripChannelMarker({
          text: String.raw`no|>{"count":2}`.replace(
            'no',
            'n o',
          ),
        },).marker,).toBe('',);
      },
    },),

    it({
      name: 'leaves an APOLOGY THAT IS FOLLOWED BY VALID JSON untouched, which '
        + 'is the case that actually separates this from a skip-to-the-brace '
        + 'rule. A refusal with no brace in it would be left alone by both '
        + 'implementations and so proves nothing; this one is mended by the '
        + 'rule we rejected and must not be mended here',
      fn: async () => {
        expect(stripChannelMarker({
          text: 'I cannot comply.\n{"count":2}',
        },),).toStrictEqual({
          content: 'I cannot comply.\n{"count":2}',
          marker: '',
        },);
      },
    },),

    it({
      name: 'strips a marker sitting in front of a FENCED object, since the '
        + 'fence stripper runs before this and cannot see a fence hidden '
        + 'behind a marker: without this the voice is lost to the very defect '
        + 'this function repairs',
      fn: async () => {
        expect(stripChannelMarker({
          text: 'p|>```json\n{"count":2}\n```',
        },),).toStrictEqual({
          content: '```json\n{"count":2}\n```',
          marker: 'p|>',
        },);
      },
    },),

    it({
      name: 'consumes SEVERAL leaked markers in a row, which is what two tokens '
        + 'straddling one delta boundary produces, while still requiring every '
        + 'fragment to be marker-shaped on its own',
      fn: async () => {
        expect(stripChannelMarker({
          text: String.raw`p|><|im_sep|>{"count":2}`,
        },),).toStrictEqual({
          content: String.raw`{"count":2}`,
          marker: 'p|><|im_sep|>',
        },);
      },
    },),

    it({
      name: 'leaves the input WHOLLY untouched when a marker run does not reach '
        + 'real content, rather than returning it partially repaired, so a '
        + 'caller can never parse a fragment of a reply as if it were all of it',
      fn: async () => {
        expect(stripChannelMarker({
          text: 'p|>|> I still cannot help.',
        },),).toStrictEqual({
          content: 'p|>|> I still cannot help.',
          marker: '',
        },);
      },
    },),

    it({
      name: 'does NOT strip a bare `>`, which is deliberate: every observed '
        + 'tail closes with `|>`, and accepting `>` alone would also eat a '
        + 'Markdown blockquote marker sitting in front of JSON',
      fn: async () => {
        expect(stripChannelMarker({
          text: String.raw`>{"count":2}`,
        },).marker,).toBe('',);
      },
    },),

    it({
      name: 'refuses a fragment longer than a marker of this family can be, so '
        + 'a sentence that happens to close with those two characters is not '
        + 'mistaken for a token tail',
      fn: async () => {
        expect(stripChannelMarker({
          text: String.raw`averylongprefix|>{"count":2}`,
        },).marker,).toBe('',);
      },
    },),

    it({
      name: 'leaves ordinary content untouched, since every other model on the '
        + 'roster returns bare JSON and must keep parsing exactly as before',
      fn: async () => {
        expect(stripChannelMarker({
          text: String.raw`{"count":2}`,
        },),).toStrictEqual({
          content: String.raw`{"count":2}`,
          marker: '',
        },);
      },
    },),

    it({
      name: 'does NOT treat a `|>` sitting inside the JSON as a marker close, '
        + 'which is the way a shape rule could have eaten real content',
      fn: async () => {
        expect(stripChannelMarker({
          text: String.raw`{"note":"a|>b"}`,
        },),).toStrictEqual({
          content: String.raw`{"note":"a|>b"}`,
          marker: '',
        },);
      },
    },),

    it({
      name: 'strips ahead of a JSON ARRAY as well as an object, since stages '
        + 'whose schema is a list would otherwise keep losing their voice',
      fn: async () => {
        expect(stripChannelMarker({
          text: String.raw`p|>[{"name":"Mittens"}]`,
        },),).toStrictEqual({
          content: String.raw`[{"name":"Mittens"}]`,
          marker: 'p|>',
        },);
      },
    },),
  ],
},);
