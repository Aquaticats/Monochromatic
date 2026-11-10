// Checks of the credential-name question the confinement check asks of the environment it hands a server.
// What: whether a variable's name is one the check withholds from a confined server, and what the two readers it is
// built from answer on their own.
// Why: a server that reaches a token is a confinement failure, so the question has to answer exactly what the pattern
// it replaced answered: the four words in any case, anywhere in the name, and no character above ASCII standing in for
// an ASCII letter.
import assert from 'node:assert/strict';
import { test } from 'node:test';

import { holdsIgnoringAsciiCase } from './ascii-case-fold.ts';
import { isCredentialEnvironmentName } from './credential-environment-name.ts';

test(
  'reads a variable name that holds a credential word',
  () => {
    assert.equal(isCredentialEnvironmentName('GITHUB_TOKEN'), true);
    assert.equal(isCredentialEnvironmentName('API_KEY'), true);
    assert.equal(isCredentialEnvironmentName('OPENAI_API_KEY'), true);
    assert.equal(isCredentialEnvironmentName('CLIENT_SECRET'), true);
    assert.equal(isCredentialEnvironmentName('DB_PASSWORD'), true);
  }
);

test(
  'reads a credential word in any case',
  () => {
    assert.equal(isCredentialEnvironmentName('api_key'), true);
    assert.equal(isCredentialEnvironmentName('Api_Key'), true);
    assert.equal(isCredentialEnvironmentName('secret'), true);
    assert.equal(isCredentialEnvironmentName('Password'), true);
    assert.equal(isCredentialEnvironmentName('token'), true);
  }
);

test(
  'reads a credential word anywhere in a longer name',
  () => {
    assert.equal(isCredentialEnvironmentName('KEYBOARD_LAYOUT'), true);
    assert.equal(isCredentialEnvironmentName('MONKEY'), true);
    assert.equal(isCredentialEnvironmentName('PASSWORDLESS_MODE'), true);
    assert.equal(isCredentialEnvironmentName('TOKENIZER_MODEL'), true);
    assert.equal(isCredentialEnvironmentName('KEYS'), true);
  }
);

test(
  'reads no credential word from an ordinary variable name',
  () => {
    assert.equal(isCredentialEnvironmentName('PATH'), false);
    assert.equal(isCredentialEnvironmentName('HOME'), false);
    assert.equal(isCredentialEnvironmentName('SHELL'), false);
    assert.equal(isCredentialEnvironmentName('XDG_CACHE_HOME'), false);
    assert.equal(isCredentialEnvironmentName(''), false);
  }
);

test(
  'reads no credential word from a name that spells one with a character above ASCII',
  () => {
    // The `i` flag leaves such a character unfolded when its uppercase form is an ASCII letter.
    assert.equal(isCredentialEnvironmentName('\u017fecret'), false);
    assert.equal(isCredentialEnvironmentName('\u212aey'), false);
  }
);

test(
  'finds a phrase in a text whatever the case of either side',
  () => {
    assert.equal(holdsIgnoringAsciiCase('GITHUB_TOKEN', 'token'), true);
    assert.equal(holdsIgnoringAsciiCase('github_token', 'token'), true);
    assert.equal(holdsIgnoringAsciiCase('aTokenb', 'token'), true);
    assert.equal(holdsIgnoringAsciiCase('tok', 'token'), false);
    assert.equal(holdsIgnoringAsciiCase('', 'token'), false);
    assert.equal(holdsIgnoringAsciiCase('en', 'en'), true);
  }
);
