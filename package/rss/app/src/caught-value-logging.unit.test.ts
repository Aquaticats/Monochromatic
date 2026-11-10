/**
 Regression coverage for a caught value that a log line dropped: every rss
 catch block formatted it with `JSON.stringify`, which renders an `Error` as
 `{}`, so the message an operator needs never reached the log.
 */

import { readdir, readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { logger, } from '@monochromatic-dev/logger-core/ts';

import { getSortedFeeds, } from './feed.ts';
import { getOPMLTexts, } from './opml-text.ts';
import { getOutlinesFromOpmls, } from './outline.ts';

/**
 Failure message a stubbed response throws while streaming its body, so a
 passing assertion proves the caught message reached the log line.
 */
const BODY_FAILURE_MESSAGE = 'body stream broke while reading the source';

/**
 Response body reader that rejects with {@link BODY_FAILURE_MESSAGE}.
 */
function failBodyRead(): Promise<string> {
  return Promise.reject(new Error(BODY_FAILURE_MESSAGE,),);
}

/**
 Builds a `fetch` replacement that answers every URL with an ok response whose
 body read fails, exercising each helper's text-conversion catch branch.
 */
function stubFailingBodyFetch(): typeof fetch {
  return async function failingBodyFetch() {
    return {
      ok: true,
      status: 200,
      text: failBodyRead,
    } as Response;
  } as typeof fetch;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: getSortedFeeds.name,
      children: [
        it({
          name: 'logs the body-read failure message instead of an empty object',
          fn: async ctx => {
            /**
             Rendered warn lines the console sink emits while the stub is active.
             */
            const lines: string[] = [];
            ctx.sinon
              .stub(console, 'warn',)
              .callsFake((...args: unknown[]) => {
                lines.push(args.map(String,).join(' ',),);
              },);
            ctx.sinon.stub(globalThis, 'fetch',).value(stubFailingBodyFetch(),);

            const feeds = await getSortedFeeds([{
              type: 'rss',
              text: 'broken',
              xmlUrl: 'https://broken.example/feed.xml',
            },],);
            await logger.flush();

            expect(feeds,).toHaveLength(0,);
            expect(lines.join('\n',),).toContain(BODY_FAILURE_MESSAGE,);
            expect(lines.join('\n',),).not.toContain('{}',);
          },
        },),
      ],
    },),
    describe({
      name: getOPMLTexts.name,
      children: [
        it({
          name: 'logs the body-read failure message instead of an empty object',
          fn: async ctx => {
            /**
             Rendered warn lines the console sink emits while the stub is active.
             */
            const lines: string[] = [];
            ctx.sinon
              .stub(console, 'warn',)
              .callsFake((...args: unknown[]) => {
                lines.push(args.map(String,).join(' ',),);
              },);
            ctx.sinon.stub(globalThis, 'fetch',).value(stubFailingBodyFetch(),);

            const texts = await getOPMLTexts(['https://broken.example/list.opml',],);
            await logger.flush();

            expect(texts,).toHaveLength(0,);
            expect(lines.join('\n',),).toContain(BODY_FAILURE_MESSAGE,);
            expect(lines.join('\n',),).not.toContain('{}',);
          },
        },),
      ],
    },),
    describe({
      name: getOutlinesFromOpmls.name,
      children: [
        it({
          name: 'logs the validation failure message instead of an empty object',
          fn: async ctx => {
            /**
             Rendered warn lines the console sink emits while the stub is active.
             */
            const lines: string[] = [];
            ctx.sinon
              .stub(console, 'warn',)
              .callsFake((...args: unknown[]) => {
                lines.push(args.map(String,).join(' ',),);
              },);
            /**
             OPML document whose only subscription carries an unusable `xmlUrl`,
             so the outline fails URL validation and the catch branch logs.
             */
            const opml = '<?xml version="1.0"?><opml version="2.0"><body>'
              + '<outline text="group"><outline text="broken" xmlUrl="not a url"/>'
              + '</outline></body></opml>';
            ctx.sinon
              .stub(globalThis, 'fetch',)
              .value(async function stubOpmlFetch() {
                return new Response(opml, { status: 200, },);
              } as typeof fetch,);

            const outlines = await getOutlinesFromOpmls(['https://broken.example/list.opml',],);
            await logger.flush();

            expect(outlines,).toHaveLength(0,);
            expect(lines.join('\n',),).toContain('Invalid',);
            expect(lines.join('\n',),).not.toContain('{}',);
          },
        },),
      ],
    },),
    describe({
      name: 'rss log lines',
      children: [
        it({
          name: 'never interpolate JSON.stringify of a caught value',
          fn: async () => {
            /**
             Source file names in this module's directory, tests excluded because
             only shipped logging code is under test.
             */
            const entries = await readdir(import.meta.dirname,);
            /**
             Source file names that ship, with the co-located tests removed.
             */
            const sourceFiles = entries
              .filter(function isShippedSource(name,) {
                return (name.endsWith('.ts',)) && (!name.endsWith('.unit.test.ts',));
              },);
            /**
             File name and source text of every shipped module, read in parallel.
             */
            const scanned = await Promise.all(sourceFiles.map(
              async function scanSource(name,) {
                /**
                 Source text searched line by line so a finding names its line.
                 */
                const text = await readFile(
                  join(
                    import.meta.dirname,
                    name,
                  ),
                  'utf8',
                );
                return {
                  name,
                  text,
                };
              },
            ),);
            /**
             Offending `file: line` labels, empty when every catch logs its value.
             */
            const offenders = scanned.flatMap(function matchSource({
              name,
              text,
            },) {
              return text
                .split('\n',)
                .flatMap(function matchLine(line, index,) {
                  if (!line.includes('JSON.stringify(error',))
                    return [];
                  return [`${name}:${String(index + 1,)}`];
                },);
            },);
            expect(sourceFiles.length,).toBeGreaterThan(0,);
            expect(offenders,).toHaveLength(0,);
          },
        },),
      ],
    },),
  ],
},);
