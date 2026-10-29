/**
 * Unit tests for the dependency-graph mapping.
 *
 * Pure function; no fixtures, no DB. Each case asserts the exact set of
 * fragment keys produced for a given (event, context) pair.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import {
  dependenciesFor,
  type EventInput,
  type ResolvedEventContext,
} from './dependency-graph.ts';
import {
  ANY_LABEL,
  filterListKey,
  issueDetailKey,
} from './fragment-keys.ts';

await describe({
  name: '',
  children: [
    describe({
      name: dependenciesFor.name,
      children: [
        it({
          name:
            'comment.created emits issue detail + per-label filter lists for both states',
          async fn() {
            const event: EventInput = {
              kind: 'comment.created',
              resourceId: 'issue-1',
            };
            const context: ResolvedEventContext = {
              repoId: 'repo-1',
              issueLabelIds: ['bug',],
              repoLabelIds: [
                'bug',
                'feat',
              ],
              issueState: 'open',
            };
            const keys = dependenciesFor({
              event,
              context,
            },);
            expect(
              keys.has(issueDetailKey({
              repoId: 'repo-1',
              issueId: 'issue-1',
            },),),
            )
              .toBe(true,);
            expect(
              keys.has(filterListKey({
              repoId: 'repo-1',
              labelId: ANY_LABEL,
              state: 'open',
            },),),
            )
              .toBe(true,);
            expect(
              keys.has(filterListKey({
              repoId: 'repo-1',
              labelId: ANY_LABEL,
              state: 'closed',
            },),),
            )
              .toBe(true,);
            expect(
              keys.has(filterListKey({
              repoId: 'repo-1',
              labelId: 'bug',
              state: 'open',
            },),),
            )
              .toBe(true,);
            expect(
              keys.has(filterListKey({
              repoId: 'repo-1',
              labelId: 'bug',
              state: 'closed',
            },),),
            )
              .toBe(true,);
            // `feat` is not on this issue, so per-issue path does not include it.
            // `issue.labeled` adds repo-wide; `comment.created` does not.
            expect(
              keys.has(filterListKey({
              repoId: 'repo-1',
              labelId: 'feat',
              state: 'open',
            },),),
            )
              .toBe(false,);
          },
        },),
        it({
          name:
            'issue.labeled additionally invalidates every repo label filter for the current state',
          async fn() {
            const event: EventInput = {
              kind: 'issue.labeled',
              resourceId: 'issue-1',
            };
            const context: ResolvedEventContext = {
              repoId: 'repo-1',
              issueLabelIds: ['bug',],
              repoLabelIds: [
                'bug',
                'feat',
                'docs',
              ],
              issueState: 'open',
            };
            const keys = dependenciesFor({
              event,
              context,
            },);
            for (const labelId of ['bug', 'feat', 'docs',]) {
              expect(
                keys.has(filterListKey({
                repoId: 'repo-1',
                labelId,
                state: 'open',
              },),),
              )
                .toBe(true,);
            }
          },
        },),
        it({
          name: 'returns deduplicated keys (Set semantics)',
          async fn() {
            const event: EventInput = {
              kind: 'issue.labeled',
              resourceId: 'issue-1',
            };
            const context: ResolvedEventContext = {
              repoId: 'repo-1',
              issueLabelIds: [
                'bug',
                'bug',
              ],
              repoLabelIds: ['bug',],
              issueState: 'open',
            };
            const keys = dependenciesFor({
              event,
              context,
            },);
            const bugOpen = filterListKey({
              repoId: 'repo-1',
              labelId: 'bug',
              state: 'open',
            },);
            const occurrences = [...keys,]
              .filter(function isBugOpen(k,) {
                return k === bugOpen;
              },)
              .length;
            expect(occurrences,).toBe(1,);
          },
        },),
        it({
          name: 'issue with zero labels still emits the no-filter lists',
          async fn() {
            const event: EventInput = {
              kind: 'comment.created',
              resourceId: 'issue-1',
            };
            const context: ResolvedEventContext = {
              repoId: 'repo-1',
              issueLabelIds: [],
              repoLabelIds: ['bug',],
              issueState: 'open',
            };
            const keys = dependenciesFor({
              event,
              context,
            },);
            expect(
              keys.has(filterListKey({
              repoId: 'repo-1',
              labelId: ANY_LABEL,
              state: 'open',
            },),),
            )
              .toBe(true,);
            // No per-label keys.
            expect(
              keys.has(filterListKey({
              repoId: 'repo-1',
              labelId: 'bug',
              state: 'open',
            },),),
            )
              .toBe(false,);
          },
        },),
        it({
          name: 'issue.created behaves like comment.created (no extra repo-wide fanout)',
          async fn() {
            const event: EventInput = {
              kind: 'issue.created',
              resourceId: 'issue-1',
            };
            const context: ResolvedEventContext = {
              repoId: 'repo-1',
              issueLabelIds: [],
              repoLabelIds: [
                'bug',
                'feat',
              ],
              issueState: 'open',
            };
            const keys = dependenciesFor({
              event,
              context,
            },);
            expect(
              keys.has(filterListKey({
              repoId: 'repo-1',
              labelId: 'bug',
              state: 'open',
            },),),
            )
              .toBe(false,);
            expect(
              keys.has(filterListKey({
              repoId: 'repo-1',
              labelId: ANY_LABEL,
              state: 'open',
            },),),
            )
              .toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
