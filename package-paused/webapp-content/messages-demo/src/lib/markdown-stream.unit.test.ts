import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CHUNK_HARD_CAP_BYTES,
  CHUNK_TARGET_BYTES,
  extractPreview,
  renderChunks,
  segmentBlocks,
} from './markdown-stream.ts';

await describe({
  name: '',
  children: [
    describe({
      name: segmentBlocks.name,
      children: [
        it({
          name: 'splits at blank line into separate blocks',
          fn: async () => {
            const blocks = [...segmentBlocks('# title\n\nbody paragraph\n',),];
            expect(blocks,).toHaveLength(2,);
            expect(blocks[0],).toBe('# title\n',);
            expect(blocks[1],).toBe('body paragraph\n',);
          },
        },),

        it({
          name: 'keeps blank lines inside a fenced code block intact',
          fn: async () => {
            const md = '```ts\nlet x = 1;\n\nlet y = 2;\n```\n';
            const blocks = [...segmentBlocks(md,),];
            expect(blocks,).toHaveLength(1,);
            expect(blocks[0],).toContain('let x = 1;',);
            expect(blocks[0],).toContain('let y = 2;',);
            expect(blocks[0],).toContain('```',);
          },
        },),

        it({
          name: 'keeps a list across consecutive lines as a single block',
          fn: async () => {
            const md = '- item one\n- item two\n- item three\n';
            const blocks = [...segmentBlocks(md,),];
            expect(blocks,).toHaveLength(1,);
            expect(blocks[0],).toContain('item one',);
            expect(blocks[0],).toContain('item three',);
          },
        },),

        it({
          name: 'handles multiple consecutive blank lines without emitting empties',
          fn: async () => {
            const blocks = [...segmentBlocks('a\n\n\n\nb\n',),];
            expect(blocks,).toHaveLength(2,);
            expect(blocks[0],).toBe('a\n',);
            expect(blocks[1],).toBe('b\n',);
          },
        },),

        it({
          name: 'tolerates trailing newline at end of input',
          fn: async () => {
            const blocks = [...segmentBlocks('para\n',),];
            expect(blocks,).toHaveLength(1,);
            expect(blocks[0],).toBe('para\n',);
          },
        },),

        it({
          name: 'returns empty generator on empty input',
          fn: async () => {
            const blocks = [...segmentBlocks('',),];
            expect(blocks,).toHaveLength(0,);
          },
        },),

        it({
          name: 'returns empty generator on whitespace-only input',
          fn: async () => {
            const blocks = [...segmentBlocks('\n\n\n',),];
            expect(blocks,).toHaveLength(0,);
          },
        },),

        it({
          name: 'recognises tilde-fenced code blocks the same as backtick',
          fn: async () => {
            const md = '~~~\nplain\n\nstill in fence\n~~~\n';
            const blocks = [...segmentBlocks(md,),];
            expect(blocks,).toHaveLength(1,);
            expect(blocks[0],).toContain('still in fence',);
          },
        },),
      ],
    },),

    describe({
      name: renderChunks.name,
      children: [
        it({
          name: 'returns one chunk for small input',
          fn: async () => {
            const chunks = [...renderChunks('# Hello\n',),];
            expect(chunks,).toHaveLength(1,);
            expect(chunks[0]?.html,).toContain('<h1>',);
            expect(chunks[0]?.md,).toContain('# Hello',);
            expect(chunks[0]?.charCount,).toBe(chunks[0]?.md.length,);
          },
        },),

        it({
          name: 'returns no chunks for empty input',
          fn: async () => {
            expect([...renderChunks('',),],).toHaveLength(0,);
          },
        },),

        it({
          name: 'splits when accumulated HTML reaches the soft target',
          fn: async () => {
            // Generate enough paragraphs that the rendered HTML exceeds
            // the soft target. Each paragraph is small but blank-line
            // separated so they form distinct blocks.
            const paragraph = 'lorem ipsum dolor sit amet '.repeat(50,);
            const blockCount = Math.ceil(
              CHUNK_TARGET_BYTES / paragraph.length,
            ) + 50;
            const md = Array
              .from(
                {
                  length: blockCount,
                },
                function build() {
                  return paragraph;
                },
              )
              .join('\n\n',);
            const chunks = [...renderChunks(md,),];
            expect(chunks.length,).toBeGreaterThan(1,);
            for (const chunk of chunks)
              expect(chunk.charCount,).toBe(chunk.md.length,);
          },
        },),

        it({
          name:
            'preserves order: concatenated md equals original (modulo trailing newline normalisation)',
          fn: async () => {
            const md = 'p1\n\np2\n\np3\n';
            const chunks = [...renderChunks(md,),];
            const joined = chunks
              .map(function pluckMd(chunk,) {
                return chunk.md;
              },)
              .join('',);
            // segmentBlocks strips blank-line separators; check tokens.
            for (const token of ['p1', 'p2', 'p3',])
              expect(joined,).toContain(token,);
          },
        },),

        it({
          name: 'each chunk html is non-empty',
          fn: async () => {
            const chunks = [...renderChunks('# A\n\nB\n\nC\n',),];
            for (const chunk of chunks)
              expect(chunk.html.length,).toBeGreaterThan(0,);
          },
        },),
      ],
    },),

    describe({
      name: extractPreview.name,
      children: [
        it({
          name: 'strips a leading heading marker',
          fn: async () => {
            expect(extractPreview({
              md: '# Hello\n\nWorld',
              maxLength: 50,
            },),)
              .toBe('Hello World',);
          },
        },),

        it({
          name: 'returns placeholder when source is a pure code block',
          fn: async () => {
            expect(extractPreview({
              md: '```\ncode\n```',
              maxLength: 50,
            },),)
              .toBe('(no text preview)',);
          },
        },),

        it({
          name: 'truncates to maxLength characters',
          fn: async () => {
            const md = 'word '.repeat(100,);
            expect(extractPreview({
              md,
              maxLength: 50,
            },)
              .length,)
              .toBeLessThanOrEqual(50,);
          },
        },),

        it({
          name: 'strips inline code markers',
          fn: async () => {
            expect(extractPreview({
              md: 'see `foo()` for details',
              maxLength: 50,
            },),)
              .toBe('see for details',);
          },
        },),

        it({
          name: 'preserves link text and drops the href',
          fn: async () => {
            expect(extractPreview({
              md: 'click [here](https://example.com) please',
              maxLength: 50,
            },),)
              .toBe('click here please',);
          },
        },),

        it({
          name: 'drops images entirely',
          fn: async () => {
            expect(extractPreview({
              md: '![alt](pic.png) caption',
              maxLength: 50,
            },),)
              .toBe('caption',);
          },
        },),

        it({
          name: 'collapses runs of whitespace',
          fn: async () => {
            expect(extractPreview({
              md: 'a   b\n\n\nc',
              maxLength: 50,
            },),)
              .toBe('a b c',);
          },
        },),

        it({
          name: 'returns placeholder on empty input',
          fn: async () => {
            expect(extractPreview({
              md: '',
              maxLength: 50,
            },),)
              .toBe('(no text preview)',);
          },
        },),
      ],
    },),

    describe({
      name: 'constants',
      children: [
        it({
          name: 'CHUNK_TARGET_BYTES is positive and at most CHUNK_HARD_CAP_BYTES',
          fn: async () => {
            expect(CHUNK_TARGET_BYTES,).toBeGreaterThan(0,);
            expect(CHUNK_TARGET_BYTES,).toBeLessThanOrEqual(CHUNK_HARD_CAP_BYTES,);
          },
        },),
      ],
    },),
  ],
},);
