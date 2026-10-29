import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  AmbiguousReconciliationError,
  GitHubProcessTimeoutError,
  PublicationInterruptedError,
  PublicationStoppedError,
  publishIssues,
  type GitHubApiClient,
  type RenderedIssue,
} from '../dist/final/node/index.mjs';

await describe({
  name: publishIssues.name,
  children: [
    it({
      name: 'creates Issues serially with pacing between mutations',
      fn: async () => {
        /**
         Create request titles observed by fake API.
         */
        const createdTitles: string[] = [];
        /**
         Requested pacing delays.
         */
        const waits: number[] = [];
        /**
         Next synthetic Issue number.
         */
        let nextNumber = 1;
        /**
         Fake GitHub API for high-water reads and successful creates.
         */
        const api: GitHubApiClient = async (request,) => {
          if (request.method === 'GET') {
            return {
              status: 200,
              headers: {},
              body: [],
            };
          }
          /**
           Create request body required by publisher.
           */
          const {body} = request;
          if ((body === undefined) || ((typeof body.title) !== 'string')) {
            throw new Error('missing create title',);
          }
          createdTitles.push(body.title,);
          /**
           Number allocated to current fake Issue.
           */
          const number = nextNumber;
          nextNumber += 1;
          return {
            status: 201,
            headers: {},
            body: {
              number,
              html_url: `https://github.com/Aquaticat/issues-api/issues/${String(number,)}`,
            },
          };
        };
        /**
         Two complete rendered Issue requests.
         */
        const issues: readonly RenderedIssue[] = [
          {
            position: { kind: 'record', value: 1, },
            security: false,
            title: 'First',
            body: 'First body',
            labels: ['needs-triage',],
          },
          {
            position: { kind: 'record', value: 2, },
            security: false,
            title: 'Second',
            body: 'Second body',
            labels: ['needs-triage',],
          },
        ];

        expect(await publishIssues({
          repository: {
            owner: 'Aquaticat',
            name: 'issues-api',
            url: 'https://github.com/Aquaticat/issues-api',
          },
          issues,
          api,
          wait: async (milliseconds,) => {
            waits.push(milliseconds,);
          },
        },),).toStrictEqual({
          created: [
            {
              position: { kind: 'record', value: 1, },
              number: 1,
              url: 'https://github.com/Aquaticat/issues-api/issues/1',
            },
            {
              position: { kind: 'record', value: 2, },
              number: 2,
              url: 'https://github.com/Aquaticat/issues-api/issues/2',
            },
          ],
        },);
        expect(createdTitles,).toStrictEqual(['First', 'Second',],);
        expect(waits,).toStrictEqual([1_000,],);
      },
    },),
    it({
      name: 'reconciles an ambiguous server failure before retrying',
      fn: async () => {
        /**
         Mutable fake API state hidden behind one binding.
         */
        const state = {
          creates: 0,
          highWaterReads: 0,
          waits: [] as number[],
        };
        /**
         Fake API returning one ambiguous failure followed by success.
         */
        const api: GitHubApiClient = async (request,) => {
          if (request.method === 'GET') {
            state.highWaterReads += 1;
            return {
              status: 200,
              headers: {},
              body: [],
            };
          }
          state.creates += 1;
          if (state.creates === 1) {
            return {
              status: 503,
              headers: {},
              body: { message: 'Service unavailable', },
            };
          }
          return {
            status: 201,
            headers: {},
            body: {
              number: 9,
              html_url: 'https://github.com/Aquaticat/issues-api/issues/9',
            },
          };
        };
        /**
         Single rendered Issue request.
         */
        const issue: RenderedIssue = {
          position: { kind: 'line', value: 3, },
          security: false,
          title: 'Retry me',
          body: 'Exact body',
          labels: [],
        };

        expect(await publishIssues({
          repository: {
            owner: 'Aquaticat',
            name: 'issues-api',
            url: 'https://github.com/Aquaticat/issues-api',
          },
          issues: [issue,],
          api,
          wait: async (milliseconds,) => {
            state.waits.push(milliseconds,);
          },
        },),).toStrictEqual({
          created: [
            {
              position: { kind: 'line', value: 3, },
              number: 9,
              url: 'https://github.com/Aquaticat/issues-api/issues/9',
            },
          ],
        },);
        expect(state.creates,).toBe(2,);
        expect(state.highWaterReads,).toBe(2,);
        expect(state.waits,).toStrictEqual([1_000,],);
      },
    },),
    it({
      name: 'accepts one exact reconciliation match without retry',
      fn: async () => {
        /**
         Mutable call counters for fake API.
         */
        const state = { highWaterReads: 0, creates: 0, };
        /**
         Fake timeout followed by one exact visible Issue.
         */
        const api: GitHubApiClient = async (request,) => {
          if (request.method === 'POST') {
            state.creates += 1;
            throw new GitHubProcessTimeoutError({ stdout: '', stderr: '', });
          }
          if (request.endpoint.endsWith('/issues/11',)) {
            return {
              status: 200,
              headers: {},
              body: {
                title: 'Reconciled',
                body: 'Exact body',
                html_url: 'https://github.com/Aquaticat/issues-api/issues/11',
              },
            };
          }
          state.highWaterReads += 1;
          return {
            status: 200,
            headers: {},
            body: [{ number: state.highWaterReads === 1 ? 10 : 11, },],
          };
        };
        /**
         Ambiguously created Issue request.
         */
        const issue: RenderedIssue = {
          position: { kind: 'record', value: 1, },
          security: false,
          title: 'Reconciled',
          body: 'Exact body',
          labels: [],
        };

        expect(await publishIssues({
          repository: {
            owner: 'Aquaticat',
            name: 'issues-api',
            url: 'https://github.com/Aquaticat/issues-api',
          },
          issues: [issue,],
          api,
          wait: async () => {},
        },),).toStrictEqual({
          created: [{
            position: { kind: 'record', value: 1, },
            number: 11,
            url: 'https://github.com/Aquaticat/issues-api/issues/11',
          },],
        },);
        expect(state.creates,).toBe(1,);
      },
    },),
    it({
      name: 'honors retry-after without reconciliation for rate limits',
      fn: async () => {
        /**
         Mutable fake rate-limit state.
         */
        const state = { creates: 0, waits: [] as number[], };
        /**
         Fake rate-limited create followed by success.
         */
        const api: GitHubApiClient = async (request,) => {
          if (request.method === 'GET') {
            return { status: 200, headers: {}, body: [], };
          }
          state.creates += 1;
          return state.creates === 1
            ? {
              status: 429,
              headers: { 'retry-after': '2', },
              body: { message: 'rate limited', },
            }
            : {
              status: 201,
              headers: {},
              body: {
                number: 12,
                html_url: 'https://github.com/Aquaticat/issues-api/issues/12',
              },
            };
        };
        /**
         Rate-limited Issue request.
         */
        const issue: RenderedIssue = {
          position: { kind: 'record', value: 1, },
          security: false,
          title: 'Rate limited',
          body: 'Body',
          labels: [],
        };

        await publishIssues({
          repository: {
            owner: 'Aquaticat',
            name: 'issues-api',
            url: 'https://github.com/Aquaticat/issues-api',
          },
          issues: [issue,],
          api,
          wait: async (milliseconds,) => {
            state.waits.push(milliseconds,);
          },
        },);
        expect(state.waits,).toStrictEqual([2_000,],);
      },
    },),
    it({
      name: 'stops on multiple exact reconciliation matches',
      fn: async () => {
        /**
         Mutable high-water lookup count.
         */
        const state = { highWaterReads: 0, };
        /**
         Fake API exposing two exact matches after server failure.
         */
        const api: GitHubApiClient = async (request,) => {
          if (request.method === 'POST') {
            return { status: 503, headers: {}, body: {}, };
          }
          if (request.endpoint.endsWith('/issues/11',)
            || request.endpoint.endsWith('/issues/12',))
            {
            const number = request.endpoint.endsWith('/11',) ? 11 : 12;
            return {
              status: 200,
              headers: {},
              body: {
                title: 'Ambiguous',
                body: 'Exact body',
                html_url: `https://github.com/Aquaticat/issues-api/issues/${String(number,)}`,
              },
            };
          }
          state.highWaterReads += 1;
          return {
            status: 200,
            headers: {},
            body: [{ number: state.highWaterReads === 1 ? 10 : 12, },],
          };
        };
        /**
         Captured multiple-match failure.
         */
        let caught: unknown;
        try {
          await publishIssues({
            repository: {
              owner: 'Aquaticat',
              name: 'issues-api',
              url: 'https://github.com/Aquaticat/issues-api',
            },
            issues: [{
              position: { kind: 'record', value: 1, },
              security: false,
              title: 'Ambiguous',
              body: 'Exact body',
              labels: [],
            },],
            api,
            wait: async () => {},
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(PublicationStoppedError,);
        expect((caught as PublicationStoppedError).cause,).toBeInstanceOf(
          AmbiguousReconciliationError,
        );
        expect(
          ((caught as PublicationStoppedError).cause as AmbiguousReconciliationError).urls,
        ).toStrictEqual([
          'https://github.com/Aquaticat/issues-api/issues/11',
          'https://github.com/Aquaticat/issues-api/issues/12',
        ],);
      },
    },),
    it({
      name: 'reports created Issues and positioned stopping failure',
      fn: async () => {
        /**
         Mutable fake create count.
         */
        const state = { creates: 0, };
        /**
         Fake API succeeding once then returning terminal validation status.
         */
        const api: GitHubApiClient = async (request,) => {
          if (request.method === 'GET') {
            return { status: 200, headers: {}, body: [], };
          }
          state.creates += 1;
          return state.creates === 1
            ? {
              status: 201,
              headers: {},
              body: {
                number: 20,
                html_url: 'https://github.com/Aquaticat/issues-api/issues/20',
              },
            }
            : {
              status: 422,
              headers: {},
              body: { message: 'Validation failed', },
            };
        };
        /**
         Captured positioned publication failure.
         */
        let caught: unknown;
        try {
          await publishIssues({
            repository: {
              owner: 'Aquaticat',
              name: 'issues-api',
              url: 'https://github.com/Aquaticat/issues-api',
            },
            issues: [
              {
                position: { kind: 'record', value: 1, },
                security: false,
                title: 'Created',
                body: 'Created body',
                labels: [],
              },
              {
                position: { kind: 'record', value: 2, },
                security: false,
                title: 'Rejected',
                body: 'Rejected body',
                labels: [],
              },
            ],
            api,
            wait: async () => {},
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(PublicationStoppedError,);
        expect((caught as PublicationStoppedError).position,).toStrictEqual({
          kind: 'record',
          value: 2,
        },);
        expect((caught as PublicationStoppedError).created,).toStrictEqual([
          {
            position: { kind: 'record', value: 1, },
            number: 20,
            url: 'https://github.com/Aquaticat/issues-api/issues/20',
          },
        ],);
      },
    },),
    it({
      name: 'stops future creation after publication interrupt',
      fn: async () => {
        /**
         Mutable publication interrupt state.
         */
        const state = { creates: 0, interrupted: false, };
        /**
         Fake API that raises stop flag after first successful create.
         */
        const api: GitHubApiClient = async (request,) => {
          if (request.method === 'GET') {
            return { status: 200, headers: {}, body: [], };
          }
          state.creates += 1;
          state.interrupted = true;
          return {
            status: 201,
            headers: {},
            body: {
              number: 30,
              html_url: 'https://github.com/Aquaticat/issues-api/issues/30',
            },
          };
        };
        /**
         Captured handled interruption.
         */
        let caught: unknown;
        try {
          await publishIssues({
            repository: {
              owner: 'Aquaticat',
              name: 'issues-api',
              url: 'https://github.com/Aquaticat/issues-api',
            },
            issues: [
              {
                position: { kind: 'record', value: 1, },
                security: false,
                title: 'First',
                body: 'First body',
                labels: [],
              },
              {
                position: { kind: 'record', value: 2, },
                security: false,
                title: 'Second',
                body: 'Second body',
                labels: [],
              },
            ],
            api,
            wait: async () => {},
            shouldStop: () => state.interrupted,
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(PublicationInterruptedError,);
        expect((caught as PublicationInterruptedError).created,).toHaveLength(1,);
        expect(state.creates,).toBe(1,);
      },
    },),
  ],
},);
