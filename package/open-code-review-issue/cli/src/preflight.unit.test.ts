import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  preflightPublication,
  type GitHubApiClient,
} from '../dist/final/node/index.mjs';

await describe({
  name: preflightPublication.name,
  children: [
    it({
      name: 'selects title prefix and verified commit source link',
      fn: async () => {
        /**
         Endpoints observed by fake GitHub API.
         */
        const endpoints: string[] = [];
        /**
         Fake API returning absent label and verified commit.
         */
        const api: GitHubApiClient = async (request,) => {
          endpoints.push(request.endpoint,);
          if (request.endpoint.endsWith('/labels/needs-triage',)) {
            return {
              status: 404,
              headers: {},
              body: { message: 'Not Found', },
            };
          }
          return {
            status: 200,
            headers: {},
            body: {
              sha: 'abcdefabcdefabcdefabcdefabcdefabcdefabcd',
            },
          };
        };

        expect(await preflightPublication({
          repository: {
            owner: 'Aquaticat',
            name: 'issues-api',
            url: 'https://github.com/Aquaticat/issues-api',
          },
          resolvedHead: 'abcdefabcdefabcdefabcdefabcdefabcdefabcd',
          api,
        },),).toStrictEqual({
          needsTriageLabel: false,
          sourceLink: {
            repository: 'Aquaticat/issues-api',
            commit: 'abcdefabcdefabcdefabcdefabcdefabcdefabcd',
          },
        },);
        expect(endpoints,).toStrictEqual([
          'repos/Aquaticat/issues-api/labels/needs-triage',
          'repos/Aquaticat/issues-api/commits/abcdefabcdefabcdefabcdefabcdefabcdefabcd',
        ],);
      },
    },),
  ],
},);
