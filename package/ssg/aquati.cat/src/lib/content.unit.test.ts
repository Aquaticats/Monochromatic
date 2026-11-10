/**
 Tests for MDX content parsing and date attachment.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  attachDates,
  type LoadedPost,
  parseFrontmatter,
  type ResolvedDates,
} from './content.ts';

//region Frontmatter fixtures

/**
 Well-formed frontmatter block followed by a body.
 */
const WELL_FORMED = '---\ntitle: Hello\n---\nbody';

/**
 Body that survives the well-formed block above.
 */
const WELL_FORMED_BODY = 'body';

/**
 Thematic break that starts with three dashes plus one more, so it is not a fence.
 */
const THEMATIC_BREAK_FIRST = '----\n# Heading\n---\nbody';

/**
 Frontmatter block whose closing fence ends the file with no trailing newline.
 */
const CLOSING_FENCE_AT_EOF = '---\ntitle: x\n---';

/**
 Frontmatter block behind a UTF-8 byte order mark.
 */
const BOM_PREFIXED = `\uFEFF${WELL_FORMED}`;

/**
 Frontmatter block with CRLF line endings.
 */
const CRLF_BLOCK = '---\r\ntitle: x\r\n---\r\nbody';

/**
 Document whose dashes appear mid-line inside the YAML block.
 */
const DASHES_MID_LINE = '---\ntitle: a---b\n---\nbody';

/**
 Document with an opening fence and no closing fence.
 */
const UNCLOSED = '---\ntitle: x\nbody';

//endregion Frontmatter fixtures

//region Date attachment fixtures

/**
 Path key used by the date attachment fixtures.
 */
const FIXTURE_FILE_PATH = 'src/content/en/about.mdx';

/**
 Loaded post fixture awaiting date resolution.
 */
const LOADED_POST: LoadedPost = {
  lang: 'en',
  name: 'about',
  fileData: {
    title: 'About',
    description: 'About the author',
    tags: ['portfolio',],
  },
  authoredDates: {},
  body: '# About',
  filePath: FIXTURE_FILE_PATH,
  contentHash: 'fixture-content-hash',
};

/**
 Dates keyed by the fixture post's path.
 */
const DATES_BY_PATH: ReadonlyMap<string, ResolvedDates> = new Map([
  [FIXTURE_FILE_PATH, {
    published: new Date('2026-04-16T10:01:43.000Z',),
    updated: new Date('2026-05-14T08:31:54.000Z',),
  },],
],);

//endregion Date attachment fixtures

await describe({
  name: '',
  children: [
    describe({
      name: parseFrontmatter.name,
      children: [
        it({
          name: 'splits a well-formed block into data and body',
          fn: async function splitsWellFormedBlock(): Promise<void> {
            /**
             Parsed frontmatter and body.
             */
            const { data, content, } = parseFrontmatter(WELL_FORMED,);

            expect(data,).toEqual({ title: 'Hello', },);
            expect(content,).toBe(WELL_FORMED_BODY,);
          },
        },),
        it({
          name: 'rejects a fence that is not followed by a line end',
          fn: async function rejectsOverlongFence(): Promise<void> {
            /**
             Parsed result for a document that opens with a thematic break.
             */
            const { data, content, } = parseFrontmatter(THEMATIC_BREAK_FIRST,);

            expect(data,).toEqual({},);
            expect(content,).toBe(THEMATIC_BREAK_FIRST,);
          },
        },),
        it({
          name: 'accepts a closing fence at end of file',
          fn: async function acceptsClosingFenceAtEof(): Promise<void> {
            /**
             Parsed frontmatter and body.
             */
            const { data, content, } = parseFrontmatter(CLOSING_FENCE_AT_EOF,);

            expect(data,).toEqual({ title: 'x', },);
            expect(content,).toBe('',);
          },
        },),
        it({
          name: 'skips a leading byte order mark',
          fn: async function skipsByteOrderMark(): Promise<void> {
            /**
             Parsed frontmatter and body behind the BOM.
             */
            const { data, content, } = parseFrontmatter(BOM_PREFIXED,);

            expect(data,).toEqual({ title: 'Hello', },);
            expect(content,).toBe(WELL_FORMED_BODY,);
          },
        },),
        it({
          name: 'splits a CRLF block without leaving a carriage return in the body',
          fn: async function splitsCrlfBlock(): Promise<void> {
            /**
             Parsed frontmatter and body.
             */
            const { data, content, } = parseFrontmatter(CRLF_BLOCK,);

            expect(data,).toEqual({ title: 'x', },);
            expect(content,).toBe('body',);
          },
        },),
        it({
          name: 'ignores dashes that do not start a line',
          fn: async function ignoresDashesMidLine(): Promise<void> {
            /**
             Parsed frontmatter and body.
             */
            const { data, content, } = parseFrontmatter(DASHES_MID_LINE,);

            expect(data,).toEqual({ title: 'a---b', },);
            expect(content,).toBe('body',);
          },
        },),
        it({
          name: 'returns the document unchanged when no closing fence exists',
          fn: async function returnsDocumentWhenUnclosed(): Promise<void> {
            /**
             Parsed result for a document with no closing fence.
             */
            const { data, content, } = parseFrontmatter(UNCLOSED,);

            expect(data,).toEqual({},);
            expect(content,).toBe(UNCLOSED,);
          },
        },),
      ],
    },),
    describe({
      name: attachDates.name,
      children: [
        it({
          name: 'attaches resolved dates to each loaded post',
          fn: async function attachesResolvedDates(): Promise<void> {
            /**
             Hydrated posts.
             */
            const posts = attachDates({
              loadedPosts: [LOADED_POST,],
              datesByFilePath: DATES_BY_PATH,
            },);

            expect(posts,).toHaveLength(1,);
            expect(posts[0]?.data.published,).toEqual(new Date('2026-04-16T10:01:43.000Z',),);
            expect(posts[0]?.data.updated,).toEqual(new Date('2026-05-14T08:31:54.000Z',),);
            expect(posts[0]?.data.title,).toBe('About',);
          },
        },),
        it({
          name: 'rejects a post with no resolved dates',
          fn: async function rejectsMissingDates(): Promise<void> {
            /**
             Error thrown for a post missing from the date map.
             */
            let caught: unknown;
            try {
              attachDates({
                loadedPosts: [LOADED_POST,],
                datesByFilePath: new Map(),
              },);
            }
            catch (error) {
              caught = error;
            }

            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain(FIXTURE_FILE_PATH,);
          },
        },),
        it({
          name: 'sorts posts by updated date, newest first',
          fn: async function sortsByUpdatedDate(): Promise<void> {
            /**
             Older post fixture.
             */
            const older: LoadedPost = {
              ...LOADED_POST,
              name: 'older',
              filePath: 'src/content/en/older.mdx',
            };
            /**
             Date map holding both fixture posts, the later one first.
             */
            const dates = new Map(DATES_BY_PATH,);
            dates.set('src/content/en/older.mdx', {
              published: new Date('2026-01-01T00:00:00.000Z',),
              updated: new Date('2026-01-02T00:00:00.000Z',),
            },);

            /**
             Sorted posts.
             */
            const posts = attachDates({
              loadedPosts: [older, LOADED_POST,],
              datesByFilePath: dates,
            },);

            expect(posts.map(function name(post,) {
              return post.name;
            },),).toEqual(['about', 'older',],);
          },
        },),
      ],
    },),
  ],
},);
