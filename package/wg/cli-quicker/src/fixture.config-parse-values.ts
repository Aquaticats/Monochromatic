/**
 Golden fixtures for the DNS token classifier behind `isIpLiteral`.

 Verdicts were recorded from the `/^[0-9.]+$/u` site the linear scan replaced,
 so the unit test and the differential probe replay one shared input set.

 @module
 */

/**
 Giant digit run pinning linear classification on one huge token.
 */
const GIANT_DIGIT_TOKEN = '1'.repeat(10_000,);

/**
 Giant letter run pinning a huge non-literal token.
 */
const GIANT_SEARCH_TOKEN = 'a'.repeat(10_000,);

/**
 One recorded classification verdict for `isIpLiteral`.
 */
export type IpLiteralFixture = {
  /**
   Verdict the replaced pattern produced for `token`.
   */
  readonly expected: boolean;

  /**
   One comma-separated DNS token as the classifier receives it.
   */
  readonly token: string;
};

/**
 Tokens reaching the colon branch, classified literal without a digits-and-dots check.
 */
export const IP_LITERAL_COLON_FIXTURES: readonly IpLiteralFixture[] = [
  {
    expected: true,
    token: ':',
  },
  {
    expected: true,
    token: '::1',
  },
  {
    expected: true,
    token: '2001:db8::1',
  },
  {
    expected: true,
    token: 'fe80::1%eth0',
  },
  {
    expected: true,
    token: 'dns.google:53',
  },
  {
    expected: true,
    token: '1.2.3.4:5',
  },
  {
    expected: true,
    token: '999.999.999.999:0',
  },
];

/**
 Tokens that are one run of ASCII digits and dots, literal even when no valid IPv4 address.
 */
export const IP_LITERAL_DIGIT_DOT_FIXTURES: readonly IpLiteralFixture[] = [
  {
    expected: true,
    token: '0',
  },
  {
    expected: true,
    token: '9',
  },
  {
    expected: true,
    token: '198.245.51.147',
  },
  {
    expected: true,
    token: '1.1.1.1',
  },
  {
    expected: true,
    token: '0.0.0.0',
  },
  {
    expected: true,
    token: '0000',
  },
  {
    expected: true,
    token: '1234567890',
  },
  {
    expected: true,
    token: '.',
  },
  {
    expected: true,
    token: '...',
  },
  {
    expected: true,
    token: '1..2',
  },
  {
    expected: true,
    token: '1.2.3.4.5',
  },
  {
    expected: true,
    token: '999.999.999.999',
  },
  {
    expected: true,
    token: '.0.',
  },
  {
    expected: true,
    token: '0.',
  },
];

/**
 Tokens the pattern rejected: the empty failure path, search domains, boundary
 whitespace and control characters, and every non-ASCII digit shape the `u`
 flag keeps outside the class.
 */
export const IP_LITERAL_REJECTED_FIXTURES: readonly IpLiteralFixture[] = [
  {
    expected: false,
    token: '',
  },
  {
    expected: false,
    token: 'dns.google',
  },
  {
    expected: false,
    token: 'lan.example',
  },
  {
    expected: false,
    token: 'a',
  },
  {
    expected: false,
    token: '1e5',
  },
  {
    expected: false,
    token: '-1',
  },
  {
    expected: false,
    token: '+1',
  },
  {
    expected: false,
    token: '1,1',
  },
  {
    expected: false,
    token: '0x10',
  },
  {
    expected: false,
    token: '1.1.1.1/32',
  },
  {
    expected: false,
    token: ' 1.1.1.1',
  },
  {
    expected: false,
    token: '1.1.1.1 ',
  },
  {
    expected: false,
    token: `1.1.1.1${String.fromCodePoint(0x0A,)}`,
  },
  {
    expected: false,
    token: `${String.fromCodePoint(0x0A,)}1.1.1.1`,
  },
  {
    expected: false,
    token: `1.1.1.1${String.fromCodePoint(0x0D,)}${String.fromCodePoint(0x0A,)}`,
  },
  {
    expected: false,
    token: `1.1.1.1${String.fromCodePoint(0x09,)}`,
  },
  {
    expected: false,
    token: `1.1.1.1${String.fromCodePoint(0x00,)}`,
  },
  {
    expected: false,
    token: String.fromCodePoint(0x06_60,),
  },
  {
    expected: false,
    token: `${String.fromCodePoint(0x06_60,)}.${String.fromCodePoint(0x06_60,)}`,
  },
  {
    expected: false,
    token: String.fromCodePoint(0xFF_10,),
  },
  {
    expected: false,
    token: String.fromCodePoint(0x1_D7_D9,),
  },
  {
    expected: false,
    token: `1${String.fromCodePoint(0x1_D7_D9,)}`,
  },
  {
    expected: false,
    token: String.fromCodePoint(0xD8_00,),
  },
  {
    expected: false,
    token: String.fromCodePoint(0xDC_00,),
  },
];

/**
 One recorded DNS field split for `parseConfigText`.
 */
export type DnsFieldFixture = {
  /**
   Tokens the split records as IP literals.
   */
  readonly dns: readonly string[];

  /**
   Tokens the split records as search domains.
   */
  readonly dnsSearch: readonly string[];

  /**
   Behaviour the fixture pins, reused as the test name.
   */
  readonly name: string;

  /**
   Raw `DNS =` value text after the surrounding `DNS = ` prefix.
   */
  readonly value: string;
};

/**
 DNS field values as the site's real input source delivers them, split into
 literal and search-domain sides.
 */
export const DNS_FIELD_FIXTURES: readonly DnsFieldFixture[] = [
  {
    dns: [
      '198.245.51.147',
      '1.1.1.1',
    ],
    dnsSearch: ['lan.example'],
    name: 'splits literal addresses from search domains',
    value: '198.245.51.147, 1.1.1.1, lan.example',
  },
  {
    dns: [
      '::1',
      'fe80::1',
    ],
    dnsSearch: ['lan.example'],
    name: 'keeps IPv6 literals on the literal side',
    value: '::1, fe80::1, lan.example',
  },
  {
    dns: [
      '999.999.999.999',
      '1..2',
      '...',
      '.',
    ],
    dnsSearch: [],
    name: 'keeps digit-and-dot runs literal even when they are not valid IPv4 addresses',
    value: '999.999.999.999, 1..2, ..., .',
  },
  {
    dns: ['1.1.1.1'],
    dnsSearch: ['lan.example'],
    name: 'drops empty comma segments',
    value: '1.1.1.1,, , lan.example',
  },
  {
    dns: ['1.1.1.1'],
    dnsSearch: ['lan.example'],
    name: 'trims whitespace around each comma-separated token',
    value: '  1.1.1.1 ,  lan.example  ',
  },
  {
    dns: [GIANT_DIGIT_TOKEN],
    dnsSearch: [GIANT_SEARCH_TOKEN],
    name: 'keeps a giant digit run literal and a giant letter run as a search domain',
    value: `${GIANT_DIGIT_TOKEN}, ${GIANT_SEARCH_TOKEN}`,
  },
];
