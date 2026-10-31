import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseResolvectlDns,
  parseWgEndpoints,
  protectedKeys,
} from '../dist/final/node/tunnel-dependencies.mjs';

await describe({
  name: parseResolvectlDns.name,
  children: [
    it({
      name: 'reads every server of the link, with DNS-over-TLS names, scopes and ports removed',
      fn: async () => {
        expect(parseResolvectlDns({
          text: 'Link 11 (fr-par-fr1): 2606:4700:4700::1111 1.1.1.1 9.9.9.9 149.112.112.112 2620:fe::fe 2620:fe::9\n',
        },),).toEqual([
          '2606:4700:4700::1111',
          '1.1.1.1',
          '9.9.9.9',
          '149.112.112.112',
          '2620:fe::fe',
          '2620:fe::9',
        ],);
        expect(parseResolvectlDns({ text: 'Link 11 (wg0): 1.1.1.1#cloudflare-dns.com fe80::1%3 9.9.9.9:53 [2606:4700:4700::1111]:853\n', },),).toEqual([
          '1.1.1.1',
          'fe80::1',
          '9.9.9.9',
          '2606:4700:4700::1111',
        ],);
      },
    },),
    it({
      name: 'reports no server for a link without DNS, empty output and text that is not a link line',
      fn: async () => {
        expect(parseResolvectlDns({ text: 'Link 11 (wg0): \n', },),).toEqual([],);
        expect(parseResolvectlDns({ text: 'Link 11 (wg0):\n', },),).toEqual([],);
        expect(parseResolvectlDns({ text: '', },),).toEqual([],);
        expect(parseResolvectlDns({ text: 'Link 11 (wg0): not-an-address\n', },),).toEqual([],);
        expect(parseResolvectlDns({ text: 'no label here 1.1.1.1\n', },),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: parseWgEndpoints.name,
  children: [
    it({
      name: 'reads the address of every peer endpoint without its port, for both families',
      fn: async () => {
        expect(parseWgEndpoints({
          text: 'abc=\t203.0.113.1:51820\ndef=\t[2001:db8::1]:51820\nghi=\t(none)\n',
        },),).toEqual(['203.0.113.1', '2001:db8::1',],);
        expect(parseWgEndpoints({ text: '', },),).toEqual([],);
        expect(parseWgEndpoints({ text: 'abc=\n', },),).toEqual([],);
        expect(parseWgEndpoints({ text: 'abc=\thost.example:51820\n', },),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: protectedKeys.name,
  children: [
    it({
      name: 'builds the keys host rules use, in normalized spelling, for DNS servers and endpoints of both families',
      fn: async () => {
        /**
         Keys of the tunnel's own addresses.
         */
        const keys = protectedKeys({
          dependencies: {
            dnsServers: ['1.1.1.1', '2606:4700:4700::1111',],
            endpoints: ['203.0.113.1', '2001:DB8::1',],
          },
        },);
        expect(keys.has('-4|1.1.1.1',),).toBe(true,);
        expect(keys.has('-6|2606:4700:4700:0:0:0:0:1111',),).toBe(true,);
        expect(keys.has('-4|203.0.113.1',),).toBe(true,);
        expect(keys.has('-6|2001:db8:0:0:0:0:0:1',),).toBe(true,);
        expect(keys.has('-4|9.9.9.9',),).toBe(false,);
        expect(protectedKeys({ dependencies: { dnsServers: [], endpoints: [], }, },).size,).toBe(0,);
      },
    },),
  ],
},);
