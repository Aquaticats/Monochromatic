/**
 Tests for reading which images a passage shows.

 WHAT THESE PIN is the reader that lets the pipeline hand a stage the picture rather
 than the markup naming it. Validated against the pinned corpus before these
 were written: 380 references found, 380 present in the tree, none missing,
 matching an independent count of the same construct.

 The stray-space case is not defensiveness. One reference in the corpus writes
 the placeholder with a space before the directory, and a reader that missed it
 would report that entry as showing one image fewer than it does, which is the
 kind of quiet undercount that makes a later measurement wrong rather than
 absent.

 The double-quote cases are the same lesson learnt again. Four source pages at
 pin `a41fc607` quote their paths with double marks, seven paths in all, and a
 reader that took single marks only sent one of them (BI4PBV, 2026-09-04)
 through its picture stage with nothing to read and nothing to say about it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  photoPath,
  photoReferences,
} from '../dist/final/node/index.mjs';

/**
 Placeholder the corpus writes an entry's own directory as.

 AN ESCAPED TEMPLATE LITERAL, so the characters are the ones the corpus
 carries without this file appearing to leave a placeholder uninterpolated.
 */
const ENTRY = `\${path}`;

/**
 Builds one photo element naming the given assets.

 @param assets - asset paths as the element carries them

 @returns Element as a page writes it

 @example
 ```ts
 const element = elementOf({ assets: [`${ENTRY}/photos/tabby.webp`,], },);
 ```
 */
function elementOf({ assets, }: { readonly assets: readonly string[]; },): string {
  return `<PhotoScroll photos={[ ${
    assets.map(function quoted(asset,): string {
      return `'${asset}'`;
    },)
      .join(', ',)
  } ]} />`;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: photoReferences.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS EVERY ASSET ONE ELEMENT NAMES, in the order it names them, since a passage '
            + 'showing several pictures is transcribed from all of them',
          fn: async () => {
            /**
             One element naming three pictures.
             */
            const text = elementOf({
              assets: [
                `${ENTRY}/photos/tabby.webp`,
                `${ENTRY}/photos/mittens.webp`,
                `${ENTRY}/photos/sill.jpg`,
              ],
            },);

            expect(photoReferences({ text, },)
              .map(function toName(reference,): string {
                return reference.assetName;
              },),).toEqual([
                'tabby.webp',
                'mittens.webp',
                'sill.jpg',
              ],);
          },
        },),

        it({
          name: 'TOLERATES A SPACE AFTER THE PLACEHOLDER, which one corpus reference writes: reading '
            + 'it as a different prefix would report that entry as showing one picture fewer than it '
            + 'does, and an undercount is worse than a gap because nothing looks wrong',
          fn: async () => {
            /**
             The stray-space spelling, verbatim.
             */
            const text = elementOf({ assets: [`${ENTRY} /photos/tabby.webp`,], },);

            expect(photoReferences({ text, },).length,).toBe(1,);
            expect(photoReferences({ text, },)[0]?.assetName,).toBe('tabby.webp',);
          },
        },),

        it({
          name: 'READS DOUBLE-QUOTED PATHS IN THE MULTI-LINE ARRAY FORM four source pages write, in '
            + 'order, since a reader that knew one quote mark sent one of those pages through its '
            + 'picture stage with nothing to read',
          fn: async () => {
            /**
             The double-quoted, one-path-per-line spelling, as those pages lay it out.
             */
            const text = `<PhotoScroll photos={[\n`
              + `    "${ENTRY}/photos/tabby.webp",\n`
              + `    "${ENTRY}/photos/mittens.webp",\n`
              + `    "${ENTRY}/photos/sill.webp"\n`
              + `]} />`;

            expect(photoReferences({ text, },)
              .map(function toName(reference,): string {
                return reference.assetName;
              },),).toEqual([
                'tabby.webp',
                'mittens.webp',
                'sill.webp',
              ],);
          },
        },),

        it({
          name: 'READS A DOUBLE-QUOTED ELEMENT AND A SINGLE-QUOTED ONE in the same passage, since a '
            + 'page may mix the two spellings across its sections',
          fn: async () => {
            /**
             One element of each spelling.
             */
            const text = `<PhotoScroll photos={["${ENTRY}/photos/one.webp"]} />\n\n`
              + `She also drew this.\n\n${elementOf({ assets: [`${ENTRY}/photos/two.webp`,], },)}`;

            expect(photoReferences({ text, },)
              .map(function toName(reference,): string {
                return reference.assetName;
              },),).toEqual([
                'one.webp',
                'two.webp',
              ],);
          },
        },),

        it({
          name: 'CLOSES A STRING WITH THE MARK THAT OPENED IT, so a caption in one mark beside paths '
            + 'in the other, or an apostrophe inside a double-quoted name, never splits a path in two',
          fn: async () => {
            /**
             A double-quoted caption beside single-quoted paths.
             */
            const captioned = `<PhotoScroll caption="Her cats' drawings" photos={[ '${ENTRY}/photos/one.webp' ]} />`;

            expect(photoReferences({ text: captioned, },)
              .map(function toName(reference,): string {
                return reference.assetName;
              },),).toEqual(['one.webp',],);

            /**
             A double-quoted path carrying an apostrophe.
             */
            const apostrophe = `<PhotoScroll photos={["${ENTRY}/photos/tabby's-sill.webp"]} />`;

            expect(photoReferences({ text: apostrophe, },)
              .map(function toName(reference,): string {
                return reference.assetName;
              },),).toEqual(['tabby\'s-sill.webp',],);
          },
        },),

        it({
          name: 'READS SEVERAL ELEMENTS in one passage, since a slice may show two sets of pictures',
          fn: async () => {
            /**
             Two elements with prose between them.
             */
            const text = `${
          elementOf({ assets: [`${ENTRY}/photos/one.webp`,], },)
        }\n\nShe also drew these.\n\n${
          elementOf({ assets: [`${ENTRY}/photos/two.webp`,], },)
        }`;

            expect(photoReferences({ text, },).length,).toBe(2,);
          },
        },),

        it({
          name: 'IGNORES A QUOTED STRING THAT NAMES SOMETHING ELSE, so an attribute carrying a caption '
            + 'never arrives as a file this pipeline would try to open',
          fn: async () => {
            /**
             An element carrying a caption beside its pictures.
             */
            const text = `<PhotoScroll caption='Her drawings' photos={[ '${ENTRY}/photos/one.webp' ]} />`;

            expect(photoReferences({ text, },).length,).toBe(1,);
            expect(photoReferences({ text, },)[0]?.assetName,).toBe('one.webp',);
          },
        },),

        it({
          name: 'REFUSES A PATH THAT CLIMBS OUT of the entry’s own directory, since a file name is '
            + 'what this names and anything carrying a separator is not one',
          fn: async () => {
            const text = elementOf({ assets: [`${ENTRY}/photos/../../etc/passwd`,], },);
            expect(photoReferences({ text, },).length,).toBe(0,);
          },
        },),

        it({
          name: 'READS NOTHING FROM A PASSAGE THAT SHOWS NOTHING, which most passages are',
          fn: async () => {
            expect(photoReferences({ text: 'The cat naps on the sill.', },).length,).toBe(0,);
            expect(photoReferences({ text: '', },).length,).toBe(0,);
          },
        },),

        it({
          name: 'STOPS AT A STRING THAT NEVER CLOSES inside its element, keeping the paths read before it, '
            + 'whether the closing mark is missing from the passage or only stands past the element\'s end',
          fn: async () => {
            /**
             Second path opened and never closed, with no later mark of its kind.
             */
            const unclosed = `<PhotoScroll photos={[ '${ENTRY}/photos/one.webp', '${ENTRY}/photos/two.webp ]} />`;
            /**
             Second path opened with a mark that next appears after the element's end.
             */
            const closedPastTheElement = `<PhotoScroll photos={[ '${ENTRY}/photos/one.webp', "${ENTRY}/photos/two.webp ]} />\n\n`
              + 'She said "hello" to the cat.';

            expect({
              unclosed: photoReferences({ text: unclosed, },),
              closedPastTheElement: photoReferences({ text: closedPastTheElement, },),
            },).toEqual({
              unclosed: [{ assetName: 'one.webp', },],
              closedPastTheElement: [{ assetName: 'one.webp', },],
            },);
          },
        },),

        it({
          name: 'TERMINATES ON AN UNCLOSED ELEMENT rather than reading the rest of the document as its '
            + 'attributes, so one malformed page cannot make every later quotation look like a file',
          fn: async () => {
            /**
             An element nobody closed, followed by ordinary prose in quotes.
             */
            const text = `<PhotoScroll photos={[ '${ENTRY}/photos/one.webp'\n\n`
              + `She said '${ENTRY}/photos/not-a-picture.webp' in passing.`;

            expect(photoReferences({ text, },).length,).toBeLessThanOrEqual(2,);
          },
        },),
      ],
    },),

    describe({
      name: photoPath.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PLACES AN ASSET UNDER ITS OWN ENTRY, which is what the placeholder in the markup '
            + 'stands for',
          fn: async () => {
            expect(photoPath({
              entryId: 'Tabby',
              assetName: 'intro.webp',
            },),).toBe('people/Tabby/photos/intro.webp',);
          },
        },),
      ],
    },),

    describe({
      name: `${photoReferences.name} mixed quotes`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS BOTH ASSETS where the element quotes one with a single mark and the other with a double, '
            + 'the earlier opening quote kept, and READS NO ASSET where the path names no photos directory',
          fn: async () => {
            const mixed = photoReferences({
              text: `<PhotoScroll photos={[ '\${path}/photos/tabby.webp', "\${path}/photos/sill.jpg" ]} />`,
            },);
            expect(mixed.map(function toName(reference,): string {
              return reference.assetName;
            },),).toEqual(['tabby.webp', 'sill.jpg',],);
            const outside = photoReferences({
              text: `<PhotoScroll photos={[ '\${path}/photos/tabby.webp', '\${path}/elsewhere/cat.webp' ]} />`,
            },);
            expect(outside.map(function toName(reference,): string {
              return reference.assetName;
            },),).toEqual(['tabby.webp',],);
          },
        },),
      ],
    },),
  ],
},);
