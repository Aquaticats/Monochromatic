import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { parseConfigText, } from '../dist/final/node/config.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: parseConfigText.name,
      children: [
        //region Interface key parsing

        it({
          name: 'parses addresses, DNS, MTU, and Table from the interface section',
          fn: async () => {
            const config = parseConfigText({
              interfaceName: 'wg0',
              text: [
                '[Interface]',
                'PrivateKey = aaaaa',
                'Address = 172.17.170.170/32, fd00::1/128',
                'DNS = 198.245.51.147, 1.1.1.1, lan.example',
                'MTU = 1420',
                'Table = off',
                '',
                '[Peer]',
                'PublicKey = bbbbb',
                'AllowedIPs = 0.0.0.0/0',
              ].join('\n',),
            },);
            expect(config.addresses,).toEqual(['172.17.170.170/32', 'fd00::1/128',],);
            expect(config.dns,).toEqual(['198.245.51.147', '1.1.1.1',],);
            expect(config.dnsSearch,).toEqual(['lan.example',],);
            expect(config.mtu,).toBe(1_420,);
            expect(config.table,).toBe('off',);
          },
        },),

        it({
          name: 'collects hooks with %i preserved and values unstripped',
          fn: async () => {
            const config = parseConfigText({
              interfaceName: 'wg0',
              text: [
                '[Interface]',
                'PrivateKey = aaaaa',
                'PreUp = echo pre %i # trailing comment',
                'PostUp = echo post %i',
                'PreDown = echo predown %i',
                'PostDown = echo postdown %i',
              ].join('\n',),
            },);
            // Hooks use the unstripped value (comment preserved), matching wg-quick.
            expect(config.preUp,).toEqual(['echo pre %i # trailing comment',],);
            expect(config.postUp,).toEqual(['echo post %i',],);
            expect(config.preDown,).toEqual(['echo predown %i',],);
            expect(config.postDown,).toEqual(['echo postdown %i',],);
          },
        },),

        it({
          name: 'parses the ExemptMark interface key',
          fn: async () => {
            const config = parseConfigText({
              interfaceName: 'wg0',
              text: ['[Interface]', 'PrivateKey = aaaaa', 'ExemptMark = 100',].join('\n',),
            },);
            expect(config.exemptMark,).toBe(100,);
          },
        },),

        it({
          name: 'rejects an ExemptMark carrying netavark masquerade mask 0x2000',
          fn: async () => {
            expect(() => {
              parseConfigText({
                interfaceName: 'wg0',
                text: ['[Interface]', 'PrivateKey = aaaaa', 'ExemptMark = 8888',].join('\n',),
              },);
            },).toThrow('0x2000',);
          },
        },),

        it({
          name: 'omits ExemptMark when absent',
          fn: async () => {
            const config = parseConfigText({
              interfaceName: 'wg0',
              text: ['[Interface]', 'PrivateKey = aaaaa',].join('\n',),
            },);
            expect(config.exemptMark,).toBeUndefined();
          },
        },),

        //endregion Interface key parsing

        //region Peer endpoint parsing

        it({
          name: 'collects distinct endpoint ports across hostname IPv4 and bracketed IPv6 peers',
          fn: async () => {
            const config = parseConfigText({
              interfaceName: 'wg0',
              text: [
                '[Interface]',
                'PrivateKey = aaaaa',
                '[Peer]',
                'Endpoint = vpn.example:2049',
                '[Peer]',
                'Endpoint = 192.0.2.1:2049',
                '[Peer]',
                'Endpoint = [2001:db8::1]:51820 # transport',
              ].join('\n',),
            },);
            expect(config.endpointPorts,).toEqual([2_049, 51_820,],);
            expect(config.wgConfig,).toContain('Endpoint = vpn.example:2049',);
            expect(config.wgConfig,).toContain('Endpoint = [2001:db8::1]:51820 # transport',);
          },
        },),

        ...[
          'vpn.example',
          ':51820',
          '2001:db8::1:51820',
          'vpn.example:0',
          'vpn.example:65536',
          'vpn.example:https',
        ].map(function invalidEndpoint(value,) {
          return it({
            name: `rejects invalid endpoint ${value}`,
            fn: async () => {
              expect(() => parseConfigText({
                interfaceName: 'wg0',
                text: ['[Interface]', 'PrivateKey = aaaaa', '[Peer]', `Endpoint = ${value}`,].join('\n',),
              },),).toThrow('Endpoint',);
            },
          },);
        },),

        it({
          name: 'does not collect Endpoint outside a peer section',
          fn: async () => {
            const config = parseConfigText({
              interfaceName: 'wg0',
              text: ['[Interface]', 'Endpoint = misplaced.example:51820',].join('\n',),
            },);
            expect(config.endpointPorts,).toEqual([],);
          },
        },),

        //endregion Peer endpoint parsing

        //region wgConfig reconstruction

        it({
          name: 'forwards the [Interface] header and PrivateKey so wg addconf accepts it',
          fn: async () => {
            const config = parseConfigText({
              interfaceName: 'wg0',
              text: [
                '[Interface]',
                'PrivateKey = aaaaa',
                'Address = 10.0.0.1/32',
                '',
                '[Peer]',
                'PublicKey = bbbbb',
                'AllowedIPs = 0.0.0.0/0',
              ].join('\n',),
            },);
            expect(config.wgConfig,).toContain('[Interface]',);
            expect(config.wgConfig,).toContain('PrivateKey = aaaaa',);
            expect(config.wgConfig,).toContain('[Peer]',);
            expect(config.wgConfig,).toContain('AllowedIPs = 0.0.0.0/0',);
          },
        },),

        it({
          name: 'omits consumed MTU from the forwarded peer config',
          fn: async () => {
            const config = parseConfigText({
              interfaceName: 'wg0',
              text: [
                '[Interface]',
                'PrivateKey = aaaaa',
                'MTU = 1420',
              ].join('\n',),
            },);
            expect(config.wgConfig,).not.toContain('MTU',);
          },
        },),

        //endregion wgConfig reconstruction

        //region Large AllowedIPs linearity

        it({
          name: 'parses a very large AllowedIPs value without quadratic slowdown',
          fn: async () => {
            /**
             Large comma-separated prefix list approximating a wg-allowedips expansion.
             */
            const big = Array.from(
              { length: 4_000, },
              function toPrefix(_, index,): string {
                return `10.${(index >> 8) & 255}.${index & 255}.0/24`;
              },
            ).join(',',);
            const start = performance.now();
            const config = parseConfigText({
              interfaceName: 'wg0',
              text: ['[Interface]', 'PrivateKey = aaaaa', '[Peer]', `AllowedIPs = ${big}`,].join('\n',),
            },);
            /**
             Elapsed milliseconds for the parse.
             */
            const elapsed = performance.now() - start;
            expect(config.wgConfig,).toContain('AllowedIPs = ',);
            expect(elapsed,).toBeLessThan(250,);
          },
        },),

        //endregion Large AllowedIPs linearity

        //region Case-insensitivity and comments

        it({
          name: 'matches keys case-insensitively and strips inline comments',
          fn: async () => {
            const config = parseConfigText({
              interfaceName: 'wg0',
              text: ['[interface]', 'PRIVATEKEY = aaaaa', 'address = 10.0.0.2/32 # loopback',].join('\n',),
            },);
            expect(config.addresses,).toEqual(['10.0.0.2/32',],);
          },
        },),

        //endregion Case-insensitivity and comments
      ],
    },),
  ],
},);
