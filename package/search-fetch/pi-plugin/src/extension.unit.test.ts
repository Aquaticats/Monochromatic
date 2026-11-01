/**
 Unit tests for Pi Search Fetch extension registration.
 
 @module
 */

import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  LINKUP_WEB_FETCH_TOOL_NAME,
  LINKUP_WEB_SEARCH_TOOL_NAME,
  registerPiLinkup,
  type LinkupConfig,
  type SearchFetchToolClient,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Config fixture.
 */
const CONFIG: LinkupConfig = {
  blocklist: [],
  source: {
    path: '/home/test/.pi/agent/extensions/pi-search-fetch.json',
    loaded: false,
  },
};

/**
 Client fixture unused during registration.
 */
const CLIENT: SearchFetchToolClient = {
  async search() {
    return {
      provider: 'exa',
      response: { results: [], },
    };
  },
  async fetch() {
    return {
      provider: 'linkup',
      response: { markdown: 'ok', },
    };
  },
};

//endregion Fixtures

await describe({
  name: registerPiLinkup.name,
  children: [
    it({
      name: 'registers exactly search and fetch tools',
      fn: async () => {
        /**
         Local value for fakePi.
         */
        const fakePi = fakePiApi();

        registerPiLinkup({
          pi: fakePi.api,
          config: CONFIG,
          client: CLIENT,
        },);

        expect(fakePi.tools,).toEqual([
          LINKUP_WEB_SEARCH_TOOL_NAME,
          LINKUP_WEB_FETCH_TOOL_NAME,
        ],);
        expect(fakePi.tools.includes('linkup_web_answer',),).toBe(false,);
      },
    },),
  ],
},);

//region Helpers

/**
 Fake Pi API harness.
 */
type FakePi = {
  /**
   Fake Pi API.
   */
  readonly api: ExtensionAPI;
  /**
   Registered tool names.
   */
  readonly tools: string[];
};

/**
 Create fake Pi API that records tools.
 
 @returns fake Pi API harness
 */
function fakePiApi(): FakePi {
  /**
   Local value for tools.
   */
  const tools: string[] = [];
  /**
   Local value for api.
   */
  const api = {
    registerTool(tool: { readonly name: string; },) {
      tools.push(tool.name,);
    },
  } as unknown as ExtensionAPI;
  return {
    api,
    tools,
  };
}

//endregion Helpers
