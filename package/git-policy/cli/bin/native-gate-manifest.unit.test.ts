import {
  mkdir,
  mkdtempDisposable,
  readFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { writeGateManifest, } from './native-gate-manifest.ts';

/**
 Private scratch parent outside the repository, whose backing device is known.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 An ID in the shape podman writes, so the file's own text says which container it names.
 */
const suiteId = '3f2a9c1e'.repeat(8,);

await describe({
  name: writeGateManifest.name,
  children: [
    it({
      name: 'writes the run and every container into the evidence directory, and no field for the directory itself',
      fn: async () => {
        await mkdir(
          scratchParent,
          {
            recursive: true,
            mode: 0o700,
          },
        );
        /**
         The evidence directory this manifest is written into, removed when the check ends.
         */
        await using tree = await mkdtempDisposable(join(
          scratchParent,
          'gate-manifest-test-',
        ),);
        await writeGateManifest({
          evidenceDirectory: tree.path,
          base: 'a'.repeat(64),
          baseTag: 'localhost/git-policy-native-test-base:abc-def',
          cachyosImage: 'docker.io/cachyos/cachyos-v4:latest',
          cachyosImageId: 'b'.repeat(64),
          image: 'c'.repeat(64),
          fixtureRoot: {
            path: '/work/package/git-policy/cli/target/gate-fixtures',
            mount: 'rw,size=768m,mode=1777',
          },
          limits: [
            '--rm',
            '--memory=2g',
          ],
          run: {
            id: 'native-HZPg8O',
            label: 'dev.monochromatic.git-policy.gate-run=native-HZPg8O',
            containers: [
              {
                role: 'suite',
                name: 'gpc-gate-native-HZPg8O-suite',
                label: 'dev.monochromatic.git-policy.gate-run=native-HZPg8O',
                cidFile: 'target/verification/native-HZPg8O/suite.cid',
                id: suiteId,
              },
            ],
          },
          user: '1000:1000',
        },);
        /**
         The file as its readers parse it.
         */
        const written: unknown = JSON.parse(await readFile(
          join(
            tree.path,
            'manifest.json',
          ),
          'utf8',
        ));
        expect(written,).toEqual({
          base: 'a'.repeat(64),
          baseTag: 'localhost/git-policy-native-test-base:abc-def',
          cachyosImage: 'docker.io/cachyos/cachyos-v4:latest',
          cachyosImageId: 'b'.repeat(64),
          image: 'c'.repeat(64),
          fixtureRoot: {
            path: '/work/package/git-policy/cli/target/gate-fixtures',
            mount: 'rw,size=768m,mode=1777',
          },
          limits: [
            '--rm',
            '--memory=2g',
          ],
          run: {
            id: 'native-HZPg8O',
            label: 'dev.monochromatic.git-policy.gate-run=native-HZPg8O',
            containers: [
              {
                role: 'suite',
                name: 'gpc-gate-native-HZPg8O-suite',
                label: 'dev.monochromatic.git-policy.gate-run=native-HZPg8O',
                cidFile: 'target/verification/native-HZPg8O/suite.cid',
                id: suiteId,
              },
            ],
          },
          user: '1000:1000',
        },);
      },
      timeout: 20_000,
    },),
  ],
},);
