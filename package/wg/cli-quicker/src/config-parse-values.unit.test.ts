import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  isIpLiteral,
  parseConfigText,
} from '../dist/final/node/config.mjs';
import {
  DNS_FIELD_FIXTURES,
  type DnsFieldFixture,
  IP_LITERAL_COLON_FIXTURES,
  IP_LITERAL_DIGIT_DOT_FIXTURES,
  IP_LITERAL_REJECTED_FIXTURES,
  type IpLiteralFixture,
} from './fixture.config-parse-values.ts';

/**
 Builds one parameterised case replaying a recorded classifier verdict.

 @param fixture - Recorded token and verdict.

 @returns Test entry asserting the shipped classifier reproduces the verdict.
 */
function classifierCase({ expected, token, }: IpLiteralFixture,) {
  return it({
    name: `classifies ${JSON.stringify(token,)} as ${expected ? 'an IP literal' : 'a search domain'}`,
    fn: async () => {
      expect(isIpLiteral({ token, },),).toBe(expected,);
    },
  },);
}

/**
 Builds one parameterised case replaying a recorded DNS field split.

 @param fixture - Recorded field value and its two recorded sides.

 @returns Test entry asserting `parseConfigText` reproduces the split.
 */
function dnsFieldCase({ dns, dnsSearch, name, value, }: DnsFieldFixture,) {
  return it({
    name,
    fn: async () => {
      const config = parseConfigText({
        interfaceName: 'wg0',
        text: [
          '[Interface]',
          `DNS = ${value}`,
        ].join('\n',),
      },);
      expect(config.dns,).toEqual(dns,);
      expect(config.dnsSearch,).toEqual(dnsSearch,);
    },
  },);
}

await describe({
  name: '',
  children: [
    describe({
      name: isIpLiteral.name,
      children: [
        //region Colon branch

        ...IP_LITERAL_COLON_FIXTURES
          .map(classifierCase,),

        //endregion Colon branch

        //region Digits and dots run

        ...IP_LITERAL_DIGIT_DOT_FIXTURES
          .map(classifierCase,),

        //endregion Digits and dots run

        //region Rejected tokens

        ...IP_LITERAL_REJECTED_FIXTURES
          .map(classifierCase,),

        //endregion Rejected tokens
      ],
    },),
    describe({
      name: parseConfigText.name,
      children: DNS_FIELD_FIXTURES
        .map(dnsFieldCase,),
    },),
  ],
},);
