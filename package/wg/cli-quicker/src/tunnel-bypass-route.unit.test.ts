import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  isAbsentTableDiagnostic,
  normalizePhysicalDefaultRoute,
} from '../dist/final/node/tunnel-bypass-route.mjs';

await describe({
  name: normalizePhysicalDefaultRoute.name,
  children: [
    it({
      name: 'strips source table and protocol ownership attributes',
      fn: async () => {
        expect(normalizePhysicalDefaultRoute({
          line: 'default via 192.0.2.1 dev eth0 proto dhcp table main src 192.0.2.2 metric 100',
        },),).toEqual([
          'default',
          'via',
          '192.0.2.1',
          'dev',
          'eth0',
          'src',
          '192.0.2.2',
          'metric',
          '100',
        ],);
      },
    },),
    it({
      name: 'strips volatile IPv6 expiration while preserving preference',
      fn: async () => {
        expect(normalizePhysicalDefaultRoute({
          line: 'default via fe80::1 dev eth0 proto ra metric 1024 expires 1200sec pref medium',
        },),).toEqual([
          'default',
          'via',
          'fe80::1',
          'dev',
          'eth0',
          'metric',
          '1024',
          'pref',
          'medium',
        ],);
      },
    },),
    it({
      name: 'removes every repeated source-owned pair',
      fn: async () => {
        expect(normalizePhysicalDefaultRoute({
          line: 'default proto static via 198.51.100.1 proto boot dev eth0 table main table 254',
        },),).toEqual([
          'default',
          'via',
          '198.51.100.1',
          'dev',
          'eth0',
        ],);
      },
    },),
    it({
      name: 'accepts only exact family-table absence diagnostic',
      fn: async () => {
        expect(isAbsentTableDiagnostic({
          proto: '-6',
          exitCode: 2,
          stderr: 'Error: ipv6: FIB table does not exist.\nDump terminated\n',
        },),).toBe(true,);
        expect(isAbsentTableDiagnostic({
          proto: '-4',
          exitCode: 2,
          stderr: 'Error: ipv4: FIB table does not exist.\nDump terminated\n',
        },),).toBe(true,);
      },
    },),
    it({
      name: 'rejects mismatched or augmented table diagnostics',
      fn: async () => {
        expect(isAbsentTableDiagnostic({
          proto: '-6',
          exitCode: 1,
          stderr: 'Error: ipv6: FIB table does not exist.\nDump terminated\n',
        },),).toBe(false,);
        expect(isAbsentTableDiagnostic({
          proto: '-6',
          exitCode: 2,
          stderr: 'Error: ipv4: FIB table does not exist.\nDump terminated\n',
        },),).toBe(false,);
        expect(isAbsentTableDiagnostic({
          proto: '-6',
          exitCode: 2,
          stderr: 'Error: ipv6: FIB table does not exist.\nDump terminated\nunexpected',
        },),).toBe(false,);
      },
    },),
    it({
      name: 'preserves multipath nexthop stream',
      fn: async () => {
        expect(normalizePhysicalDefaultRoute({
          line: 'default proto static metric 100 nexthop via 192.0.2.1 dev eth0 weight 1 nexthop via 198.51.100.1 dev eth1 weight 2',
        },),).toEqual([
          'default',
          'metric',
          '100',
          'nexthop',
          'via',
          '192.0.2.1',
          'dev',
          'eth0',
          'weight',
          '1',
          'nexthop',
          'via',
          '198.51.100.1',
          'dev',
          'eth1',
          'weight',
          '2',
        ],);
      },
    },),
  ],
},);
