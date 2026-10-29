import {
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
  resolve,
} from 'node:path';
import { rolldown, } from 'rolldown';
import { dts, } from 'rolldown-plugin-dts';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/** Disposable declaration fixture workspace. */
type DeclarationFixture = {
  readonly path: string;
  [Symbol.dispose]: () => void;
};

/**
 Creates disposable declaration fixture.
 
 @returns disposable fixture outside repository state.
 */
function createDeclarationFixture(): DeclarationFixture {
  /** Unique fixture path under operating-system temporary root. */
  const path = mkdtempSync(join(scratchRoot(), 'mutates-declaration-',),);
  return {
    path,
    [Symbol.dispose]: function removeDeclarationFixture(): void {
      rmSync(path, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: 'mutation contract declaration publication',
  children: [
    it({
      name: 'preserves function overload and call-signature blocks through bundle',
      fn: async () => {
        using fixture = createDeclarationFixture();
        /** Re-exporting declaration entry. */
        const entryPath = join(fixture.path, 'entry.ts',);
        writeFileSync(entryPath, "export * from './contracts.js';\n",);
        /** Authored declarations carrying three distinct mutation contracts. */
        const contractsSource = `/**
 * Clears direct state.
 *
 * @param state - Direct state.
 *
 * @mutates state - Clears direct state before reuse.
 */
export function clearDirect(state: Set<string>,): void {
  state.clear();
}

/**
 * Clears overloaded state.
 *
 * @param state - Overloaded state.
 *
 * @mutates state - Clears overloaded state before reuse.
 */
export function clearOverload(state: Set<string>,): void;
export function clearOverload(state: Set<string>,): void {
  state.clear();
}

export type ClearCallable = {
  /**
   * Clears call-signature state.
   *
   * @param state - Call-signature state.
   *
   * @mutates state - Clears call-signature state before reuse.
   */
  (state: Set<string>,): void;
};
`;
        writeFileSync(join(fixture.path, 'contracts.ts',), contractsSource,);
        /** Fixture compiler configuration consumed by declaration plugin. */
        const tsconfigPath = join(fixture.path, 'tsconfig.json',);
        writeFileSync(
          tsconfigPath,
          '{"compilerOptions":{"declaration":true,"strict":true,"target":"ESNext"},"include":["*.ts"]}\n',
        );
        /** Declaration output directory. */
        const outDir = join(fixture.path, 'dist',);
        /** Declaration-bundling build driven through the rolldown JS API. */
        const declarationBuild = await rolldown({
          input: entryPath,
          cwd: fixture.path,
          plugins: [dts({
            generator: 'oxc',
            tsconfig: tsconfigPath,
          },),],
        },);
        await declarationBuild.write({
          dir: outDir,
          format: 'es',
        },);
        await declarationBuild.close();
        /** Bundled declaration filename selected independent of module suffix. */
        const declarationName = readdirSync(outDir,)
          .find(function declarationFile(fileName,): boolean {
            return fileName.includes('.d.') || fileName.endsWith('.d.ts',);
          },);
        if (declarationName === undefined)
          throw new Error('Expected bundled declaration output.',);
        /** Published declaration text retaining custom blocks. */
        const declarationText = readFileSync(resolve(outDir, declarationName,), 'utf8',);
        expect(declarationText.split('@mutates state',).length - 1,).toBe(3,);
        expect(declarationText.includes('Clears direct state before reuse.',),).toBe(true,);
        expect(declarationText.includes('Clears overloaded state before reuse.',),).toBe(true,);
        expect(declarationText.includes('Clears call-signature state before reuse.',),).toBe(true,);
      },
    },),
  ],
},);
