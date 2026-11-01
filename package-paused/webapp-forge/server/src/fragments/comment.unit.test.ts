/**
 * Unit tests for the standalone comment renderer.
 *
 * Pure function. Asserts permalink anchor id, body+author rendering, and
 * XSS escape on every text-bearing field.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import { renderComment, } from './comment.ts';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: renderComment.name,
      concurrency: 1,
      children: [
        it({
          name: 'sets the permalink anchor id to comment-<id>',
          async fn() {
            await Promise.resolve();
            const { html, } = renderComment({
              id: 'c1',
              authorLogin: 'alice',
              body: 'first',
              createdAt: '2026-05-06T12:00:00Z',
            },);
            expect(html.includes('id="comment-c1"',),).toBe(true,);
            expect(html.includes('data-comment-id="c1"',),).toBe(true,);
          },
        },),
        it({
          name: 'renders author and body',
          async fn() {
            await Promise.resolve();
            const { html, } = renderComment({
              id: 'c1',
              authorLogin: 'alice',
              body: 'A reply',
              createdAt: '2026-05-06T12:00:00Z',
            },);
            expect(html.includes('alice',),).toBe(true,);
            expect(html.includes('A reply',),).toBe(true,);
          },
        },),
        it({
          name: 'escapes XSS payloads in body and author',
          async fn() {
            await Promise.resolve();
            const xss = '<svg/onload=alert(1)>';
            const { html, } = renderComment({
              id: 'c1',
              authorLogin: xss,
              body: xss,
              createdAt: '2026-05-06T12:00:00Z',
            },);
            expect(html.includes('<svg',),).toBe(false,);
            expect(html.includes('&lt;svg',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
