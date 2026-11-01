/**
 Address record returned by deterministic resolver fixtures.
 */
type LookupAddress = {
  readonly address: string;
};

/**
 ASN fixture networks keyed by normalized ASN.
 */
type AsnNetworkRecords = Readonly<Record<string, readonly string[]>>;

/**
 Error raised when fixture receives unregistered hostname.
 */
class UnexpectedLookupError extends Error {
  /**
   Stable error type name.
   */
  override name = 'UnexpectedLookupError';
}

/**
 Deterministic operating-system-style hostname absence.
 */
class FixtureDnsNotFoundError extends Error {
  /**
   Stable resolver code consumed by domain handling.
   */
  readonly code = 'ENOTFOUND';

  /**
   Stable error type name.
   */
  override name = 'FixtureDnsNotFoundError';
}

/**
 Error raised when fixture receives unregistered ASN.
 */
class UnexpectedAsnLookupError extends Error {
  /**
   Stable error type name.
   */
  override name = 'UnexpectedAsnLookupError';
}

/**
 Error raised when expected failing operation resolves.
 */
class ExpectedFailureError extends Error {
  /**
   Stable error type name.
   */
  override name = 'ExpectedFailureError';
}

/**
 Deterministic resolver records shared by generation tests.
 */
const LOOKUP_RECORDS: Readonly<Record<string, readonly LookupAddress[]>> = {
  'allowed.example': [
    { address: '192.0.2.1', },
    { address: '2001:db8::1', },
  ],
  'disallowed.example': [
    { address: '192.0.2.1', },
  ],
  'empty.example': [],
  'inline#comment.example': [
    { address: '198.51.100.7', },
  ],
  'invalid-address.example': [
    { address: 'not-an-ip', },
  ],
};

/**
 Hostnames reproducing Node `ENOTFOUND` resolver failure.
 */
const NOT_FOUND_HOSTNAMES: ReadonlySet<string> = new Set([
  'missing-one.example',
  'missing-two.example',
],);

/**
 Deterministic ASN records covering CIDR,
 address,
 empty,
 and invalid responses.
 */
const ASN_NETWORK_RECORDS: AsnNetworkRecords = {
  AS64500: [
    '192.0.2.0/24',
    '2001:db8:100::/48',
  ],
  AS64501: [
    '192.0.2.0/25',
    '2001:db8:100::/49',
  ],
  AS64502: [
    '198.51.100.9',
    '2001:db8::9',
  ],
  AS64503: [],
  AS64504: ['not-a-network',],
  AS64506: [],
};

/**
 Deterministically resolves fixture hostnames.
 
 @param hostname - Fixture hostname.
 
 @returns Registered fixture addresses.
 
 @throws {@link UnexpectedLookupError} when hostname has no fixture.
 
 @example
 ```ts
 fixtureLookup({ hostname: 'allowed.example' });
 ```
 */
export function fixtureLookup(
  { hostname, }: { readonly hostname: string; },
): readonly LookupAddress[] {
  if (NOT_FOUND_HOSTNAMES.has(hostname,))
    throw new FixtureDnsNotFoundError(`getaddrinfo ENOTFOUND ${hostname}`,);
  /**
   Registered addresses for requested hostname.
   */
  const records = LOOKUP_RECORDS[hostname];
  if (records === undefined)
    throw new UnexpectedLookupError(`Unexpected lookup: ${hostname}`,);
  return records;
}

/**
 Deterministically resolves fixture ASNs.
 
 @param asn - Normalized fixture ASN.
 
 @returns Registered network and address records.
 
 @throws {@link UnexpectedAsnLookupError} when ASN has no fixture.
 
 @example
 ```ts
 fixtureAsnLookup({ asn: 'AS64500' });
 ```
 */
export function fixtureAsnLookup(
  { asn, }: { readonly asn: string; },
): readonly string[] {
  /**
   Registered networks for requested ASN.
   */
  const records = ASN_NETWORK_RECORDS[asn];
  if (records === undefined)
    throw new UnexpectedAsnLookupError(`Unexpected ASN lookup: ${asn}`,);
  return records;
}

/**
 Captures rejection from asynchronous operation.
 
 @param operation - Operation expected to reject.
 
 @returns Rejection value.
 
 @throws {@link ExpectedFailureError} when operation resolves.
 
 @example
 ```ts
 await captureError({ operation: async () => { throw new Error('fixture'); } });
 ```
 */
export async function captureError(
  { operation, }: { readonly operation: () => Promise<unknown>; },
): Promise<unknown> {
  try {
    await operation();
  }
  catch (error: unknown) {
    return error;
  }
  throw new ExpectedFailureError('Expected operation to reject.',);
}
