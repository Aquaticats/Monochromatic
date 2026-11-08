/**
 What Pi's environment may hold: a name rule that fails closed, and the few names kept in spite of it.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  carriesCredentialName,
  CREDENTIAL_NAME_WORDS,
  stripCredentialsFromEnvironment,
  VARIABLES_KEPT_DESPITE_CREDENTIAL_NAME,
} from '../dist/final/node/index.mjs';

/**
 A value a test can look for, so a leak shows as a value rather than as a name.
 */
const CREDENTIAL_VALUE = 'value-that-must-not-reach-a-command';

/**
 A value whose name carries no credential word, which every call must keep.
 */
const ORDINARY_VALUE = 'value-any-command-may-read';

await describe({
  name: '',
  children: [
    describe({
      name: carriesCredentialName.name,
      children: [
        ...CREDENTIAL_NAME_WORDS.map(function mapWord(word,) {
          return it({
            name: `answers true for a name carrying ${word}`,
            fn: async () => {
              expect(carriesCredentialName({ name: `EXAMPLE_${word}`, },),).toBe(true,);
            },
          },);
        },),
        it({
          name: 'answers true whatever the letter case',
          fn: async () => {
            expect(carriesCredentialName({ name: 'example_api_token', },),).toBe(true,);
            expect(carriesCredentialName({ name: 'ExamplePassword', },),).toBe(true,);
            expect(carriesCredentialName({ name: 'someApiKey', },),).toBe(true,);
            expect(carriesCredentialName({ name: 'aws_credentials', },),).toBe(true,);
          },
        },),
        it({
          name: 'answers false for a name carrying none of the words',
          fn: async () => {
            expect(carriesCredentialName({ name: 'HOME', },),).toBe(false,);
            expect(carriesCredentialName({ name: 'PATH', },),).toBe(false,);
            expect(carriesCredentialName({ name: 'MISE_SHELL', },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: stripCredentialsFromEnvironment.name,
      children: [
        it({
          name: 'removes a variable for every credential word and keeps nothing of them',
          fn: async () => {
            /**
             One variable per credential word, so a word left out of the rule shows.
             */
            const environment: NodeJS.ProcessEnv = Object.fromEntries(
              CREDENTIAL_NAME_WORDS.map(function toVariable(word,) {
                return [`EXAMPLE_${word}`, CREDENTIAL_VALUE,];
              },),
            );
            const stripped = stripCredentialsFromEnvironment({ environment, },);
            expect(stripped.removedNames,).toEqual(
              CREDENTIAL_NAME_WORDS.map(function toName(word,) {
                return `EXAMPLE_${word}`;
              },)
                .toSorted(),
            );
            expect(Object.keys(stripped.environment,),).toEqual([],);
            expect(JSON.stringify(stripped.environment,),).not.toContain(CREDENTIAL_VALUE,);
          },
        },),
        it({
          name: 'keeps the variables whose names carry no credential word',
          fn: async () => {
            const stripped = stripCredentialsFromEnvironment({
              environment: {
                HOME: ORDINARY_VALUE,
                PATH: ORDINARY_VALUE,
              },
            },);
            expect(stripped.environment,).toEqual({
              HOME: ORDINARY_VALUE,
              PATH: ORDINARY_VALUE,
            },);
            expect(stripped.removedNames,).toEqual([],);
          },
        },),
        it({
          name: 'removes the credential and keeps the rest in the same call',
          fn: async () => {
            const stripped = stripCredentialsFromEnvironment({
              environment: {
                MISE_GITHUB_TOKEN: CREDENTIAL_VALUE,
                HOME: ORDINARY_VALUE,
              },
            },);
            expect(Object.hasOwn(stripped.environment, 'MISE_GITHUB_TOKEN',),).toBe(false,);
            expect(stripped.environment.HOME,).toBe(ORDINARY_VALUE,);
            expect(stripped.removedNames,).toEqual(['MISE_GITHUB_TOKEN',],);
          },
        },),
        it({
          name: 'keeps each name listed as kept, and reports none of them as removed',
          fn: async () => {
            /**
             The environment holding every kept name, which an agent's work needs.
             */
            const environment: NodeJS.ProcessEnv = Object.fromEntries(
              VARIABLES_KEPT_DESPITE_CREDENTIAL_NAME.map(function toVariable(kept,) {
                return [kept.name, ORDINARY_VALUE,];
              },),
            );
            const stripped = stripCredentialsFromEnvironment({ environment, },);
            expect(Object.keys(stripped.environment,),).toEqual(
              VARIABLES_KEPT_DESPITE_CREDENTIAL_NAME.map(function toName(kept,) {
                return kept.name;
              },),
            );
            expect(stripped.removedNames,).toEqual([],);
          },
        },),
        it({
          name: 'answers an empty environment for an empty one, having removed nothing',
          fn: async () => {
            const stripped = stripCredentialsFromEnvironment({ environment: {}, },);
            expect(stripped.environment,).toEqual({},);
            expect(stripped.removedNames,).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: 'VARIABLES_KEPT_DESPITE_CREDENTIAL_NAME',
      children: [
        it({
          name: 'names only variables the rule would otherwise remove, so no entry is dead',
          fn: async () => {
            for (const kept of VARIABLES_KEPT_DESPITE_CREDENTIAL_NAME) {
              expect(carriesCredentialName({ name: kept.name, },),).toBe(true,);
            }
          },
        },),
        it({
          name: 'says why each kept name stays, so a reader can weigh removing it',
          fn: async () => {
            for (const kept of VARIABLES_KEPT_DESPITE_CREDENTIAL_NAME) {
              expect(kept.why.length,).toBeGreaterThan(0,);
            }
          },
        },),
      ],
    },),
    describe({
      name: 'CREDENTIAL_NAME_WORDS',
      children: [
        it({
          name: 'holds the words the brief names, each upper case, so the rule reads one way',
          fn: async () => {
            expect(CREDENTIAL_NAME_WORDS,).toEqual([
              'TOKEN',
              'SECRET',
              'KEY',
              'PASSWORD',
              'CREDENTIAL',
              'AUTH',
            ],);
          },
        },),
      ],
    },),
  ],
},);
