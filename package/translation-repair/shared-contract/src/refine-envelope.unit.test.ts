/**
 Tests for collecting a slice's link and footnote definitions.

 `collectDefinitions` had no test, and its failure is invisible. The
 refinement gate compares a paragraph before and after a rewrite; a paragraph
 parsed alone cannot resolve a reference whose definition lives elsewhere, so
 the definitions are appended to give the parser something to resolve
 against. If collection returns nothing, both sides of the comparison parse as
 referencing nothing, they still match, and the gate passes a rewrite that
 broke a link.

 Every fixture goes through `parseDocument` rather than being hand-built. The
 whole function turns on whether the strings in `DEFINITION_KINDS` equal the
 `kind` values the parser actually emits, and a hand-built node would assert
 my belief about those strings instead of the parser's behavior.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  collectDefinitions,
  parseDocument,
} from '../dist/final/node/index.mjs';

await describe({
  name: collectDefinitions.name,
  children: [
    it({
      name: 'collects a link definition, which is the case that proves the '
        + 'kind strings match what the parser emits rather than what this '
        + 'module believes it emits',
      fn: async () => {
        /**
         Slice referencing a link defined at the bottom.
         */
        const document = parseDocument({
          text: 'The cat naps on the [windowsill][sill].\n\n[sill]: https://example.invalid/sill\n',
        },);

        expect(collectDefinitions({ document, },),).toBe(
          '[sill]: https://example.invalid/sill',
        );
      },
    },),

    it({
      name: 'collects a footnote definition, the other kind the gate needs, '
        + 'since an unresolvable footnote reference parses as ordinary text',
      fn: async () => {
        /**
         Slice referencing a footnote defined at the bottom.
         */
        const document = parseDocument({
          text: 'The cat naps all afternoon.[^nap]\n\n[^nap]: Roughly sixteen hours a day.\n',
        },);

        expect(collectDefinitions({ document, },),).toContain(
          'Roughly sixteen hours a day.',
        );
      },
    },),

    it({
      name: 'returns an EMPTY string for a slice with no definitions, rather '
        + 'than a stray newline, so appending it to a paragraph adds nothing',
      fn: async () => {
        /**
         Ordinary prose slice with nothing to resolve.
         */
        const document = parseDocument({
          text: 'The cat naps on the windowsill.\n\nShe wakes when the sun moves.\n',
        },);

        expect(collectDefinitions({ document, },),).toBe('',);
      },
    },),

    it({
      name: 'keeps EVERY definition and joins them one per line, since a slice '
        + 'referencing three links needs all three to resolve and a joiner '
        + 'that dropped duplicates or collapsed lines would break the ones it '
        + 'dropped',
      fn: async () => {
        /**
         Slice referencing two links and a footnote.
         */
        const document = parseDocument({
          text: 'The [cat][a] naps by the [window][b].[^why]\n\n'
            + '[a]: https://example.invalid/cat\n\n'
            + '[b]: https://example.invalid/window\n\n'
            + '[^why]: Because the sun is there.\n',
        },);

        /**
         Collected definition block.
         */
        const definitions = collectDefinitions({ document, },);

        expect(definitions,).toContain('[a]: https://example.invalid/cat',);
        expect(definitions,).toContain('[b]: https://example.invalid/window',);
        expect(definitions,).toContain('Because the sun is there.',);
        expect(definitions.split('\n',).length,).toBeGreaterThanOrEqual(3,);
      },
    },),

    it({
      name: 'excludes ordinary prose, so the collected block is definitions '
        + 'only: appending whole paragraphs would change what the gate '
        + 'compares rather than only what it can resolve',
      fn: async () => {
        /**
         Slice mixing prose, a heading, and one definition.
         */
        const document = parseDocument({
          text: '# The cat\n\nShe naps on the [windowsill][sill].\n\n'
            + '[sill]: https://example.invalid/sill\n',
        },);

        /**
         Collected definition block.
         */
        const definitions = collectDefinitions({ document, },);

        expect(definitions,).toBe('[sill]: https://example.invalid/sill',);
        expect(definitions,).not.toContain('The cat',);
        expect(definitions,).not.toContain('She naps',);
      },
    },),

    it({
      name: 'collects the definition even when nothing references it, because '
        + 'collection is about what the parser can resolve against and not '
        + 'about which references happen to be used in this slice',
      fn: async () => {
        /**
         Slice whose definition is never referenced.
         */
        const document = parseDocument({
          text: 'The cat naps on the windowsill.\n\n[unused]: https://example.invalid/nowhere\n',
        },);

        expect(collectDefinitions({ document, },),).toBe(
          '[unused]: https://example.invalid/nowhere',
        );
      },
    },),

    it({
      name: 'returns empty for a document with no blocks at all, rather than '
        + 'throwing on an empty slice',
      fn: async () => {
        expect(
          collectDefinitions({ document: parseDocument({ text: '', },), },),
        ).toBe('',);
      },
    },),
  ],
},);
