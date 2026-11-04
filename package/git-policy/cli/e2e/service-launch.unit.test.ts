import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { SCENARIO_CATALOG, } from './scenario-catalog-fixture.ts';
import { scenarioRuntimeDirectory, } from './service-fixture.ts';
import { instanceOf, } from './service-launch-fixture.ts';

/**
 A directory that does not exist, so the name is of the path as given.
 The same vector is pinned in `src/native/service/service_instance_tests.rs`, so the suite and the wrapper cannot name a worktree's
 service differently.
 */
const VECTOR_PATH = '/nonexistent/instance-vector';

/**
 The instance name of that path.
 */
const VECTOR_NAME = 'b82882c15388721fd60c7ae28508d85b';

/**
 The longest socket path the operating system accepts, in bytes: `sun_path` of `sockaddr_un` holds 108 bytes with its
 terminator on Linux.
 */
const SOCKET_PATH_LIMIT = 107;

/**
 Where the suite's scenario roots sit in its container.
 */
const SCENARIO_ROOT_PREFIX = '/work/2.55.0/';

await describe({
  name: instanceOf.name,
  children: [
    it({
      name: 'names a worktree the way the wrapper does',
      fn: async () => {
        expect(await instanceOf(VECTOR_PATH,),).toBe(VECTOR_NAME,);
      },
    },),
    it({
      name: 'keeps the socket of every scenario within the length a socket path may have',
      fn: async () => {
        const longest = Math.max(...SCENARIO_CATALOG.map(function lengthOf(scenario,) {
          return Buffer.byteLength(
            `${scenarioRuntimeDirectory(`${SCENARIO_ROOT_PREFIX}${scenario.name}`,)}/cli-git/${VECTOR_NAME}.sock`,
          );
        },),);
        expect(longest,).toBeLessThanOrEqual(SOCKET_PATH_LIMIT,);
      },
    },),
  ],
},);
