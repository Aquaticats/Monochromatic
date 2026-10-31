import { readFile, } from 'node:fs/promises';
import { connect, } from 'node:net';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type {
  HostAddress,
  HostLookup,
  HostLookupResult,
} from './tunnel-bypass-hosts.ts';
import type { BypassProto, } from './tunnel-bypass-types.ts';

/**
 Module logger for the tunnel resolver.
 */
const l = tagged({ tag: 'tunnel-dns-lookup', },);

/**
 Unix socket of the systemd-resolved varlink interface.
 */
export const RESOLVE_SOCKET_PATH = '/run/systemd/resolve/io.systemd.Resolve';

/**
 Method that resolves one record type of one name on one link.
 */
export const RESOLVE_RECORD_METHOD = 'io.systemd.Resolve.ResolveRecord';

/**
 Error `systemd-resolved` returns for a name that has no record of the asked type,
 which is normal for a host with no AAAA record.
 */
export const ERROR_NO_SUCH_RECORD = 'io.systemd.Resolve.NoSuchResourceRecord';

/**
 Error `systemd-resolved` returns when the link has no DNS server.
 */
export const ERROR_NO_NAME_SERVERS = 'io.systemd.Resolve.NoNameServers';

/**
 `SD_RESOLVED_DNS`:
 ask unicast DNS only.
 */
const FLAG_DNS = 1;

/**
 `SD_RESOLVED_NO_SEARCH`:
 ask the name exactly as written, so no search domain from the network is appended.
 */
const FLAG_NO_SEARCH = 256;

/**
 `SD_RESOLVED_NO_SYNTHESIZE`:
 do not answer from local records such as the host name.
 */
const FLAG_NO_SYNTHESIZE = 2_048;

/**
 `SD_RESOLVED_NO_ZONE`:
 do not answer from the local mDNS and LLMNR zone.
 */
const FLAG_NO_ZONE = 8_192;

/**
 `SD_RESOLVED_NO_TRUST_ANCHOR`:
 do not answer from the built-in trust anchors.
 */
const FLAG_NO_TRUST_ANCHOR = 16_384;

/**
 Resolver flags for a lookup that uses only the DNS servers of the asked link.

 Only unicast DNS is asked,
 so neither mDNS nor LLMNR nor the local zone can answer,
 and the name is asked exactly as written.
 The value is the sum of the five flags above, 26881;
 `SD_RESOLVED_CLAMP_TTL` is not accepted as an input flag.
 */
export const RESOLVE_FLAGS: number = FLAG_DNS + FLAG_NO_SEARCH
  + FLAG_NO_SYNTHESIZE
  + FLAG_NO_ZONE
  + FLAG_NO_TRUST_ANCHOR;

/**
 DNS record type of an IPv4 address.
 */
export const RECORD_TYPE_A = 1;

/**
 DNS record type of an IPv6 address.
 */
export const RECORD_TYPE_AAAA = 28;

/**
 DNS class `IN`.
 */
const RECORD_CLASS_IN = 1;

/**
 Milliseconds a varlink call may take before it is abandoned.
 */
const CALL_TIMEOUT_MS = 10_000;

/**
 Bytes in an IPv4 address.
 */
const IPV4_BYTES = 4;

/**
 Bytes in an IPv6 address.
 */
const IPV6_BYTES = 16;

/**
 Bytes in one IPv6 group.
 */
const GROUP_BYTES = 2;

/**
 Bits in one byte.
 */
const BYTE_BITS = 8;

/**
 Radix of an IPv6 group.
 */
const HEX_RADIX = 16;

/**
 Bits of a DNS name label length that mark a compression pointer.
 */
const POINTER_MASK = 0xC0;

/**
 Bytes of a compression pointer.
 */
const POINTER_BYTES = 2;

/**
 Bytes from the end of a record name to its TTL field:
 two for the type and two for the class.
 */
const TTL_OFFSET_AFTER_NAME = 4;

/**
 Sends one varlink request and returns the reply text.
 */
export type VarlinkTransport = (call: { readonly payload: string; },) => Promise<string>;

/**
 Reads the kernel index of an interface.
 */
export type InterfaceIndexReader = (call: { readonly interfaceName: string; },) => Promise<number>;

/**
 Answers of one record type for one host.
 */
export type RecordAnswer = {
  /**
   Addresses with their remaining TTL.
   */
  readonly addresses: readonly HostAddress[];

  /**
   Resolver error name,
   when the call did not succeed.
   */
  readonly error?: string;
};

/**
 Calls the resolver's varlink socket once.

 The request is a JSON object ended by a NUL byte,
 and the reply is read up to the next NUL byte.

 @param payload - JSON request text without the terminator.

 @param socketPath - Socket to call.

 @param timeoutMs - Milliseconds before the call is abandoned.

 @returns Reply JSON text without the terminator.

 @throws When the socket cannot be reached,
 closes before a reply,
 or does not answer in time.

 @example
 ```ts
 await callVarlink({ payload: '{"method":"org.varlink.service.GetInfo"}' });
 ```
 */
export function callVarlink(
  {
    payload,
    socketPath = RESOLVE_SOCKET_PATH,
    timeoutMs = CALL_TIMEOUT_MS,
  }: {
    readonly payload: string;
    readonly socketPath?: string;
    readonly timeoutMs?: number;
  },
): Promise<string> {
  /**
   Promise capability settled by the socket events.
   */
  const {
    promise,
    resolve,
    reject,
  } = Promise.withResolvers<string>();
  /**
   Reply bytes received so far.
   */
  const chunks: Buffer[] = [];
  /**
   Connection to the resolver.
   */
  const socket = connect(socketPath,);
  socket.setTimeout(
    timeoutMs,
    function onTimeout(): void {
      socket.destroy(new Error(`${socketPath} did not answer within ${String(timeoutMs,)} ms`,),);
    },
  );
  socket.once(
    'connect',
    function sendRequest(): void {
      socket.write(`${payload}\0`,);
    },
  );
  socket.on(
    'data',
    function collect(chunk: Buffer,): void {
      chunks.push(chunk,);
      /**
       Everything received so far.
       */
      const received = Buffer.concat(chunks,);
      /**
       Position of the reply terminator.
       */
      const end = received.indexOf(0,);
      if (end !== (-1)) {
        resolve(received.subarray(
          0,
          end,
        )
          .toString('utf8',),);
        socket.destroy();
      }
    },
  );
  socket.once(
    'error',
    function fail(error: Error,): void {
      reject(error,);
    },
  );
  socket.once(
    'close',
    function closedEarly(): void {
      reject(new Error(`${socketPath} closed before it answered`,),);
    },
  );
  return promise;
}

/**
 Reads the kernel index of an interface from sysfs.

 @param interfaceName - Interface name.

 @param sysfsRoot - Directory that lists interfaces,
 replaceable in tests.

 @returns The index.

 @throws When the name could name another path or the index is not a number.

 @example
 ```ts
 await readInterfaceIndex({ interfaceName: 'wg0' });
 ```
 */
export async function readInterfaceIndex(
  {
    interfaceName,
    sysfsRoot = '/sys/class/net',
  }: {
    readonly interfaceName: string;
    readonly sysfsRoot?: string;
  },
): Promise<number> {
  if ((interfaceName === '') || interfaceName.includes('/',)
    || (interfaceName === '.')
    || (interfaceName === '..'))
    throw new Error(`${interfaceName} is not an interface name`,);
  /**
   Index file content.
   */
  const text = await readFile(
    `${sysfsRoot}/${interfaceName}/ifindex`,
    'utf8',
  );
  /**
   Parsed index.
   */
  const index = Math.trunc(
    Number(text.trim(),),
  );
  if ((!Number.isInteger(index,)) || (index < 1))
    throw new Error(`${interfaceName} has no usable interface index: ${text.trim()}`,);
  return index;
}

/**
 Builds the varlink request that asks for one record type of one host on one link.

 @param host - Host name.

 @param type - DNS record type.

 @param interfaceIndex - Index of the tunnel interface.

 @returns Request JSON text.

 @example
 ```ts
 resolveRecordRequest({ host: 'crates.io', type: 1, interfaceIndex: 11 });
 ```
 */
export function resolveRecordRequest(
  {
    host,
    type,
    interfaceIndex,
  }: {
    readonly host: string;
    readonly type: number;
    readonly interfaceIndex: number;
  },
): string {
  return JSON.stringify({
    method: RESOLVE_RECORD_METHOD,
    parameters: {
      name: host,
      type,
      class: RECORD_CLASS_IN,
      ifindex: interfaceIndex,
      flags: RESOLVE_FLAGS,
    },
  },);
}

/**
 Finds the end of the owner name at the start of a wire-format record:
 after the terminating zero label or a compression pointer.

 @param bytes - Record bytes.

 @returns Position of the first byte after the name.

 @example
 ```ts
 endOfName({ bytes: Buffer.from([0]) }); // 1
 ```
 */
function endOfName({ bytes, }: { readonly bytes: Buffer; },): number {
  for (let position = 0; position < bytes.length;) {
    /**
     Length byte of the next label.
     */
    const length = bytes[position] ?? 0;
    if (length === 0)
      return position + 1;
    if ((length & POINTER_MASK) === POINTER_MASK)
      return position + POINTER_BYTES;
    position += length + 1;
  }
  return bytes.length;
}

/**
 Reads the TTL out of one raw wire-format resource record.

 @param raw - Base64 of the record,
 starting with its owner name.

 @returns Seconds,
 or zero when the record is too short to hold a TTL.

 @example
 ```ts
 ttlOfRawRecord({ raw: 'AQ...' });
 ```
 */
export function ttlOfRawRecord({ raw, }: { readonly raw: string; },): number {
  /**
   Record bytes.
   */
  const bytes = Buffer.from(
    raw,
    'base64',
  );
  /**
   Position of the 32-bit TTL.
   */
  const ttlAt = endOfName({ bytes, },) + TTL_OFFSET_AFTER_NAME;
  if ((ttlAt + IPV4_BYTES) > bytes.length)
    return 0;
  return bytes.readUInt32BE(ttlAt,);
}

/**
 Renders address bytes as text of the given family.

 @param bytes - Four or sixteen bytes.

 @returns Address text,
 or an empty string for any other length.

 @example
 ```ts
 addressText({ bytes: [140, 82, 112, 3] }); // '140.82.112.3'
 ```
 */
export function addressText({ bytes, }: { readonly bytes: readonly number[]; },): string {
  if (bytes.length === IPV4_BYTES)
    return bytes.join('.',);
  if (bytes.length !== IPV6_BYTES)
    return '';
  /**
   Groups of two bytes.
   */
  const groups: string[] = [];
  for (let index = 0; index < IPV6_BYTES; index += GROUP_BYTES)
    groups.push(((((bytes[index] ?? 0) << BYTE_BITS) | (bytes[index + 1] ?? 0)).toString(HEX_RADIX,)),);
  return groups.join(':',);
}

/**
 Reports whether a parsed JSON value is a non-null object.

 @param value - Parsed JSON.

 @returns Whether the value has named fields.

 @example
 ```ts
 isFieldRecord({ error: 'x' });
 ```
 */
function isFieldRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Reads the address and TTL of one answered record,
 or nothing for a record that holds no address of the asked family.

 @param record - One entry of the reply's `rrs`.

 @param family - Asked address family.

 @returns The address,
 or an empty list.

 @example
 ```ts
 addressOfRecord({ record, family: '-4' });
 ```
 */
function addressOfRecord(
  {
    record,
    family,
  }: {
    readonly record: unknown;
    readonly family: BypassProto;
  },
): readonly HostAddress[] {
  if ((!isFieldRecord(record,)) || (!isFieldRecord(record.rr,))
    || (!Array.isArray(record.rr
      .address,)))
    return [];
  /**
   Address bytes as numbers.
   */
  const bytes: readonly number[] = (record.rr
    .address as readonly unknown[]).filter(function isByte(item,): item is number {
      return (typeof item) === 'number';
    },);
  if (bytes.length !== (family === '-4' ? IPV4_BYTES : IPV6_BYTES))
    return [];
  /**
   Address text.
   */
  const address = addressText({ bytes, },);
  return [{
    family,
    address,
    ttlSeconds: (typeof record.raw) === 'string' ? ttlOfRawRecord({ raw: record.raw, },) : 0,
  },];
}

/**
 Reads one record-type reply.

 @param text - Reply JSON text.

 @param family - Asked address family.

 @returns Addresses with TTLs,
 and the resolver's error name when the reply was an error.

 @example
 ```ts
 parseRecordReply({ text, family: '-4' });
 ```
 */
export function parseRecordReply(
  {
    text,
    family,
  }: {
    readonly text: string;
    readonly family: BypassProto;
  },
): RecordAnswer {
  /**
   Parsed reply.
   */
  const reply: unknown = JSON.parse(text,);
  if (!isFieldRecord(reply,))
    return {
      addresses: [],
      error: 'the reply was not a JSON object',
    };
  if ((typeof reply.error) === 'string')
    return {
      addresses: [],
      error: reply.error as string,
    };
  if ((!isFieldRecord(reply.parameters,)) || (!Array.isArray(reply.parameters
    .rrs,)))
    return {
      addresses: [],
      error: 'the reply held no record list',
    };
  return {
    addresses: (reply.parameters
      .rrs as readonly unknown[]).flatMap(function toAddress(record,): readonly HostAddress[] {
        return addressOfRecord({
          record,
          family,
        },);
      },),
  };
}

/**
 Asks the tunnel link's DNS servers for one record type of one host.

 @param host - Host name.

 @param family - Address family to ask for.

 @param interfaceIndex - Index of the tunnel interface.

 @param transport - Varlink transport,
 replaceable in tests.

 @returns Answers,
 or the resolver's error name.

 @example
 ```ts
 await askRecord({ host: 'crates.io', family: '-4', interfaceIndex: 11, transport: callVarlink });
 ```
 */
async function askRecord(
  {
    host,
    family,
    interfaceIndex,
    transport,
  }: {
    readonly host: string;
    readonly family: BypassProto;
    readonly interfaceIndex: number;
    readonly transport: VarlinkTransport;
  },
): Promise<RecordAnswer> {
  /**
   Reply text.
   */
  const text = await transport({
    payload: resolveRecordRequest({
      host,
      type: family === '-4' ? RECORD_TYPE_A : RECORD_TYPE_AAAA,
      interfaceIndex,
    },),
  },);
  return parseRecordReply({
    text,
    family,
  },);
}

/**
 Words for a family whose lookup found nothing.

 @param family - Asked family.

 @param answer - What came back.

 @returns Short description for the failure text.

 @example
 ```ts
 describeMissing({ family: '-4', answer: { addresses: [], error: 'x' } });
 ```
 */
function describeMissing(
  {
    family,
    answer,
  }: {
    readonly family: BypassProto;
    readonly answer: RecordAnswer;
  },
): string {
  return `${family === '-4' ? 'A' : 'AAAA'} ${answer.error ?? 'empty'}`;
}

/**
 Creates the resolver for update hosts.

 Every answer comes from the DNS servers the tunnel configuration sets on the tunnel link,
 asked through the tunnel,
 and from nowhere else:
 the lookup names the link index,
 so `systemd-resolved` sends the query out of that link to those servers only.
 The link keeps its own DNS-over-TLS setting,
 so the transport the owner configured is the transport used.

 @param interfaceName - Tunnel interface name.

 @param transport - Varlink transport,
 replaceable in tests.

 @param readIndex - Interface index reader,
 replaceable in tests.

 @returns A resolver returning addresses with TTLs.

 @example
 ```ts
 const lookup = createTunnelHostLookup({ interfaceName: 'wg0' });
 ```
 */
export function createTunnelHostLookup(
  {
    interfaceName,
    transport = callVarlink,
    readIndex = readInterfaceIndex,
  }: {
    readonly interfaceName: string;
    readonly transport?: VarlinkTransport;
    readonly readIndex?: InterfaceIndexReader;
  },
): HostLookup {
  return async function lookupThroughTunnel({ host, },): Promise<HostLookupResult> {
    /**
     Index read on every lookup, because the interface is recreated by each `up`.
     */
    const interfaceIndex = await readIndex({ interfaceName, },);
    /**
     Both families, asked together.
     */
    const [four, six,] = await Promise.all([
      askRecord({
        host,
        family: '-4',
        interfaceIndex,
        transport,
      },),
      askRecord({
        host,
        family: '-6',
        interfaceIndex,
        transport,
      },),
    ],);
    /**
     Addresses of both families.
     */
    const addresses = [
      ...four.addresses,
      ...six.addresses,
    ];
    if (addresses.length > 0) {
      l.debug(`${host}: ${String(addresses.length,)} address(es) through ${interfaceName}`,);
      return { addresses, };
    }
    if ((four.error === ERROR_NO_NAME_SERVERS) || (six.error === ERROR_NO_NAME_SERVERS))
      return {
        addresses,
        failure: `${host}: the tunnel link ${interfaceName} has no DNS server, so the name was not resolved`,
      };
    return {
      addresses,
      failure: `no address for ${host} through ${interfaceName}: ${describeMissing({
        family: '-4',
        answer: four,
      },)}; ${describeMissing({
        family: '-6',
        answer: six,
      },)}`,
    };
  };
}
