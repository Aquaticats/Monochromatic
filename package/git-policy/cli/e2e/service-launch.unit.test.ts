import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { instanceOf, } from './service-launch-fixture.ts';

/**
 A directory that does not exist, so the name is of the path as given.
 The same vector is pinned in `src/native/service_instance_tests.rs`, so the suite and the wrapper cannot name a worktree's
 service differently.
 */
const VECTOR_PATH = '/nonexistent/instance-vector';

/**
 The instance name of that path.
 */
const VECTOR_NAME = 'b82882c15388721fd60c7ae28508d85b';

await describe({
  name: instanceOf.name,
  children: [
    it({
      name: 'names a worktree the way the wrapper does',
      fn: async () => {
        expect(await instanceOf(VECTOR_PATH,),).toBe(VECTOR_NAME,);
      },
    },),
  ],
},);
