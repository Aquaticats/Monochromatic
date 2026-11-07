/**
 Tests for the guard that keeps credentials out of Git configuration.

 Every credential below is a made-up value assembled from the pieces named in this file; none is a real secret.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  findGitConfigCredentials,
  findUrlCredentials,
  type GitConfigEntry,
  parseGitConfigList,
  redactUrlCredentials,
  USERINFO_MASK,
} from '@monochromatic-dev/config-lfs-r2-worker';

/**
 Made-up secret that stands in for a real token in every case.
 */
const FAKE_SECRET = 'fake-secret-value-for-tests';

/**
 Host that stands in for the LFS server.
 */
const HOST = 'lfs.example.test';

/**
 URL as `git lfs` stores it when a token rides in the URL.
 */
const URL_WITH_PASSWORD = `https://lfs:${FAKE_SECRET}@${HOST}`;

/**
 Plain URL a credential helper would serve.
 */
const PLAIN_URL = `https://${HOST}`;

/**
 Build an entry in the local scope.

 @param key - configuration key

 @param value - configuration value

 @returns entry that came from the repository config file
 */
function localEntry({ key, value, }: { readonly key: string; readonly value: string; },): GitConfigEntry {
  return { scope: 'local', origin: 'file:.git/config', key, value, };
}

await describe({
  name: 'git config credential guard',
  children: [
    describe({
      name: findUrlCredentials.name,
      children: [
        it({
          name: 'finds a password in the userinfo of a web URL',
          fn: async () => {
            expect(findUrlCredentials({ text: URL_WITH_PASSWORD, },).map((finding,) => finding.kind),).toEqual(['password',],);
          },
        },),
        it({
          name: 'finds a user name alone on a web scheme, because hosted services read it as the token',
          fn: async () => {
            expect(findUrlCredentials({ text: `https://${FAKE_SECRET}@${HOST}/repo.git`, },).map((finding,) => finding.kind),)
              .toEqual(['user-on-web-scheme',],);
          },
        },),
        it({
          name: 'allows a user name alone on ssh, where it names the account and not a secret',
          fn: async () => {
            expect(findUrlCredentials({ text: `ssh://git@${HOST}/repo.git`, },),).toEqual([],);
          },
        },),
        it({
          name: 'finds a password on a scheme that is not a web scheme',
          fn: async () => {
            expect(findUrlCredentials({ text: `ssh://git:${FAKE_SECRET}@${HOST}/repo.git`, },).length,).toBe(1,);
          },
        },),
        it({
          name: 'passes a plain URL',
          fn: async () => {
            expect(findUrlCredentials({ text: PLAIN_URL, },),).toEqual([],);
          },
        },),
        it({
          name: 'ignores an @ in the path, the query or the fragment',
          fn: async () => {
            expect(findUrlCredentials({ text: `${PLAIN_URL}/a@b?x=y@z#f@g`, },),).toEqual([],);
          },
        },),
        it({
          name: 'ignores scp-style remotes, which have no scheme separator',
          fn: async () => {
            expect(findUrlCredentials({ text: `git@${HOST}:org/repo.git`, },),).toEqual([],);
          },
        },),
        it({
          name: 'ignores a separator with no scheme before it',
          fn: async () => {
            expect(findUrlCredentials({ text: `://x:${FAKE_SECRET}@${HOST}`, },),).toEqual([],);
          },
        },),
        it({
          name: 'treats an empty userinfo as none',
          fn: async () => {
            expect(findUrlCredentials({ text: `https://@${HOST}`, },),).toEqual([],);
          },
        },),
        it({
          name: 'ends the userinfo at the last @ of the authority, so an @ inside the password cannot hide the host',
          fn: async () => {
            /**
             URL whose password contains an `@`.
             */
            const text = `https://lfs:${FAKE_SECRET}@suffix@${HOST}/x`;
            expect(redactUrlCredentials({ text, },),).toBe(`https://${USERINFO_MASK}@${HOST}/x`,);
          },
        },),
        it({
          name: 'finds several URLs in one text and reports them in order',
          fn: async () => {
            /**
             Two credentialed URLs around a plain one.
             */
            const text = `${URL_WITH_PASSWORD} ${PLAIN_URL} https://u2:${FAKE_SECRET}2@${HOST}`;
            expect(findUrlCredentials({ text, },).length,).toBe(2,);
          },
        },),
        it({
          name: 'stops the authority at quotes, angle brackets and whitespace',
          fn: async () => {
            expect(findUrlCredentials({ text: `"${PLAIN_URL}" "u:p@x"`, },),).toEqual([],);
            expect(findUrlCredentials({ text: `<${PLAIN_URL}> then user@host`, },),).toEqual([],);
          },
        },),
        it({
          name: 'finds a credential in a URL inside a larger line',
          fn: async () => {
            expect(findUrlCredentials({ text: `batch response: Server error ${URL_WITH_PASSWORD}/objects/batch from HTTP 503`, },).length,)
              .toBe(1,);
          },
        },),
        it({
          name: 'handles a long text of separators without a host in linear time',
          fn: async () => {
            /**
             Many separators with no authority; a quadratic scan would stall here.
             */
            const text = 'a://'.repeat(50_000,);
            expect(findUrlCredentials({ text, },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: redactUrlCredentials.name,
      children: [
        it({
          name: 'replaces only the userinfo and never leaves the secret in the output',
          fn: async () => {
            /**
             Text printed after masking.
             */
            const masked = redactUrlCredentials({ text: `see ${URL_WITH_PASSWORD}/objects/batch now`, },);
            expect(masked,).toBe(`see https://${USERINFO_MASK}@${HOST}/objects/batch now`,);
            expect(masked.includes(FAKE_SECRET,),).toBe(false,);
          },
        },),
        it({
          name: 'returns text without credentials unchanged',
          fn: async () => {
            expect(redactUrlCredentials({ text: `${PLAIN_URL}/a b`, },),).toBe(`${PLAIN_URL}/a b`,);
          },
        },),
      ],
    },),
    describe({
      name: findGitConfigCredentials.name,
      children: [
        it({
          name: 'flags lfs.url that carries a password',
          fn: async () => {
            /**
             Findings for the credentialed lfs.url.
             */
            const findings = findGitConfigCredentials({ entries: [localEntry({ key: 'lfs.url', value: URL_WITH_PASSWORD, },),], },);
            expect(findings.map((finding,) => finding.reason),).toEqual(['credential-in-value',],);
            expect(findings[0]?.redactedKey,).toBe('lfs.url',);
          },
        },),
        it({
          name: 'flags a remote URL and a push URL that carry a token',
          fn: async () => {
            expect(findGitConfigCredentials({
              entries: [
                localEntry({ key: 'remote.origin.url', value: `https://${FAKE_SECRET}@${HOST}/r.git`, },),
                localEntry({ key: 'remote.origin.pushurl', value: URL_WITH_PASSWORD, },),
              ],
            },).length,).toBe(2,);
          },
        },),
        it({
          name: 'flags a key name whose subsection embeds the credential, and masks it in the finding',
          fn: async () => {
            /**
             Key `git-lfs` writes for the locks-verification setting of a credentialed endpoint.
             */
            const key = `lfs.${URL_WITH_PASSWORD}.locksverify`;
            /**
             Findings for that key.
             */
            const findings = findGitConfigCredentials({ entries: [localEntry({ key, value: 'false', },),], },);
            expect(findings.map((finding,) => finding.reason),).toEqual(['credential-in-key-name',],);
            expect(JSON.stringify(findings,).includes(FAKE_SECRET,),).toBe(false,);
            expect(findings[0]?.redactedKey,).toBe(`lfs.https://${USERINFO_MASK}@${HOST}.locksverify`,);
          },
        },),
        it({
          name: 'flags a credential both in the key name and in the value as two findings',
          fn: async () => {
            expect(findGitConfigCredentials({
              entries: [localEntry({ key: `lfs.${URL_WITH_PASSWORD}.url`, value: URL_WITH_PASSWORD, },),],
            },).map((finding,) => finding.reason),).toEqual(['credential-in-key-name', 'credential-in-value',],);
          },
        },),
        it({
          name: 'flags an Authorization header set through http.extraHeader in any letter case',
          fn: async () => {
            /**
             Findings for three spellings of the key and header.
             */
            const findings = findGitConfigCredentials({
              entries: [
                localEntry({ key: 'http.extraHeader', value: `Authorization: Basic ${FAKE_SECRET}`, },),
                localEntry({ key: `http.${PLAIN_URL}.extraheader`, value: `  authorization: Bearer ${FAKE_SECRET}`, },),
                localEntry({ key: 'http.extraheader', value: `Proxy-Authorization: Basic ${FAKE_SECRET}`, },),
              ],
            },);
            expect(findings.map((finding,) => finding.reason),).toEqual([
              'authorization-header-in-value',
              'authorization-header-in-value',
              'authorization-header-in-value',
            ],);
          },
        },),
        it({
          name: 'passes an extra header that is not a credential',
          fn: async () => {
            expect(findGitConfigCredentials({
              entries: [localEntry({ key: 'http.extraHeader', value: 'Accept: application/vnd.git-lfs+json', },),],
            },),).toEqual([],);
          },
        },),
        it({
          name: 'passes the plain setup: a plain lfs.url, a credential helper and a locksverify key for the plain URL',
          fn: async () => {
            expect(findGitConfigCredentials({
              entries: [
                localEntry({ key: 'lfs.url', value: PLAIN_URL, },),
                localEntry({ key: `lfs.${PLAIN_URL}.locksverify`, value: 'false', },),
                localEntry({ key: `credential.${PLAIN_URL}.helper`, value: 'store', },),
                localEntry({ key: 'remote.origin.url', value: 'https://github.com/Aquaticat/Monochromatic', },),
              ],
            },),).toEqual([],);
          },
        },),
        it({
          name: 'keeps the scope and origin of the entry so the user can find the file to edit',
          fn: async () => {
            expect(findGitConfigCredentials({
              entries: [{ scope: 'global', origin: 'file:/home/x/.gitconfig', key: 'lfs.url', value: URL_WITH_PASSWORD, },],
            },),).toEqual([
              { scope: 'global', origin: 'file:/home/x/.gitconfig', redactedKey: 'lfs.url', reason: 'credential-in-value', },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: parseGitConfigList.name,
      children: [
        it({
          name: 'reads scope, origin, key and value from NUL-delimited output',
          fn: async () => {
            /**
             Output of `git config --list --null --show-scope --show-origin` for two entries.
             */
            const output = `local\0file:.git/config\0lfs.url\n${PLAIN_URL}\0global\0file:/h/.gitconfig\0user.name\nA B\0`;
            expect(parseGitConfigList({ output, },),).toEqual([
              { scope: 'local', origin: 'file:.git/config', key: 'lfs.url', value: PLAIN_URL, },
              { scope: 'global', origin: 'file:/h/.gitconfig', key: 'user.name', value: 'A B', },
            ],);
          },
        },),
        it({
          name: 'reads a key with no value as an empty value',
          fn: async () => {
            expect(parseGitConfigList({ output: 'local\0file:.git/config\0core.bare\0', },),).toEqual([
              { scope: 'local', origin: 'file:.git/config', key: 'core.bare', value: '', },
            ],);
          },
        },),
        it({
          name: 'keeps a value that holds line feeds',
          fn: async () => {
            expect(parseGitConfigList({ output: 'local\0file:.git/config\0alias.x\na\nb\0', },)[0]?.value,).toBe('a\nb',);
          },
        },),
        it({
          name: 'reads empty output as no entries',
          fn: async () => {
            expect(parseGitConfigList({ output: '', },),).toEqual([],);
          },
        },),
        it({
          name: 'rejects output that ends inside a record or without a final NUL',
          fn: async () => {
            expect(() => parseGitConfigList({ output: 'local\0file:.git/config\0', },),).toThrow(SyntaxError,);
            expect(() => parseGitConfigList({ output: 'local\0file:.git/config\0k\nv', },),).toThrow(SyntaxError,);
          },
        },),
      ],
    },),
  ],
},);
