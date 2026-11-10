/**
 Specifier classification and splitting at the boundaries a build depends on:
 which specifiers name a package, and how a package specifier divides into a
 package name and a subpath. Both answers feed `node_modules` lookup, so a
 wrong one sends resolution into the wrong directory.
 */
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isPackageSpecifier,
  splitPackageSpecifier,
} from '@monochromatic-dev/css-cli';

await describe({
  name: isPackageSpecifier.name,
  children: [
    it({
      name: 'accepts a scoped package specifier',
      fn: async () => {
        expect(isPackageSpecifier('@scope/pkg/style.css',),).toBe(true,);
      },
    },),
    it({
      name: 'accepts an unscoped package specifier',
      fn: async () => {
        expect(isPackageSpecifier('pkg/sub/style.css',),).toBe(true,);
      },
    },),
    it({
      name: 'rejects a relative specifier',
      fn: async () => {
        expect(isPackageSpecifier('./local.css',),).toBe(false,);
      },
    },),
    it({
      name: 'rejects an absolute specifier',
      fn: async () => {
        expect(isPackageSpecifier('/project/local.css',),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: splitPackageSpecifier.name,
  children: [
    it({
      name: 'splits a scoped specifier into name and subpath',
      fn: async () => {
        expect(splitPackageSpecifier('@scope/pkg/sub/path.css',),)
          .toEqual(['@scope/pkg', './sub/path.css',],);
      },
    },),
    it({
      name: 'treats a scoped specifier without a subpath as the package entry',
      fn: async () => {
        expect(splitPackageSpecifier('@scope/pkg',),)
          .toEqual(['@scope/pkg', '.',],);
      },
    },),
    it({
      name: 'splits an unscoped specifier into name and subpath',
      fn: async () => {
        expect(splitPackageSpecifier('pkg/sub/path.css',),)
          .toEqual(['pkg', './sub/path.css',],);
      },
    },),
    it({
      name: 'treats an unscoped specifier without a subpath as the package entry',
      fn: async () => {
        expect(splitPackageSpecifier('pkg',),)
          .toEqual(['pkg', '.',],);
      },
    },),
  ],
},);
