import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { killSwitchTable, } from '../dist/final/node/tunnel-firewall.mjs';
import { trafficBlockTable, } from '../dist/final/node/tunnel-traffic-block.mjs';

/**
 Interface names that made one tunnel's kill-switch table the other tunnel's traffic-block table.

 `wgq` and `wgq-blocked` are the pair that did it: the kill switch of `wgq-blocked`
 was `wg-quicker-wgq-blocked`, which is the block table of `wgq`, and the `down` of
 `wgq` deleted it while `wgq-blocked` was up.
 */
const ADVERSARIAL_NAMES: readonly string[] = [
  'wgq',
  'wgq-blocked',
  'wg0',
  'wg0-blocked',
  'block-wg0',
  'kill-wg0',
  'wg-quicker',
  'wg-quicker-block-wg0',
];

await describe({
  name: 'the kill-switch and traffic-block table names',
  children: [
    it({
      name: 'never make one tunnel the table of another, whatever the interface names are',
      fn: async () => {
        /*
         Every pair of the adversarial names, in both directions:
         no interface's block table may be any interface's kill-switch table.
         */
        const collisions = ADVERSARIAL_NAMES.flatMap(function againstEvery(blocked,) {
          return ADVERSARIAL_NAMES.map(function againstKill(switching,) {
            return {
              blocked,
              switching,
              block: trafficBlockTable({ interfaceName: blocked, },),
              kill: killSwitchTable({ interfaceName: switching, },),
            };
          },);
        },).filter(function collides({ block, kill, },) {
          return block === kill;
        },);
        expect(collisions,).toEqual([],);
      },
    },),
    it({
      name: 'names the block and the kill switch with different markers, so the two families of names cannot meet',
      fn: async () => {
        expect(trafficBlockTable({ interfaceName: 'wgq', },),).toBe('wg-quicker-block-wgq',);
        expect(killSwitchTable({ interfaceName: 'wgq-blocked', },),).toBe('wg-quicker-kill-wgq-blocked',);
        expect(trafficBlockTable({ interfaceName: 'wgq', },),).not.toBe(
          killSwitchTable({ interfaceName: 'wgq-blocked', },),
        );
      },
    },),
  ],
},);
