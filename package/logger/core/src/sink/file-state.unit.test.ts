/** Verify platform layouts through the built Node artifact without changing process environment. @module */
import assert from 'node:assert/strict';
import { describe, it, } from '@monochromatic-dev/test-harness-core/ts';
import { _userLogDirectory as userLogDirectory, } from '../../dist/final/node/node.mjs';

//region Platform paths are independent of the host running these tests.
await describe({ name: 'user log directory', children: [
  it({ name: 'uses absolute XDG state and rejects relative or empty overrides', fn: async function unixState() {
    for (const configured of [undefined, '', 'relative',]) {
      assert.equal(userLogDirectory({ platform: 'linux', home: '/fixture-home',
        environment: { XDG_STATE_HOME: configured, }, }), '/fixture-home/.local/state/monochromatic/log');
    }
    assert.equal(userLogDirectory({ platform: 'linux', home: '/fixture-home',
      environment: { XDG_STATE_HOME: '/fixture-state', }, }), '/fixture-state/monochromatic/log');
  } }),
  it({ name: 'uses the macOS user log directory regardless of XDG variables', fn: async function macosState() {
    assert.equal(userLogDirectory({ platform: 'darwin', home: '/fixture-home',
      environment: { XDG_STATE_HOME: '/ignored', }, }), '/fixture-home/Library/Logs/monochromatic');
  } }),
  it({ name: 'uses Windows local application data with an absolute home fallback', fn: async function windowsState() {
    assert.equal(userLogDirectory({ platform: 'win32', home: String.raw`C:\fixture-home`,
      environment: { LOCALAPPDATA: String.raw`D:\fixture-local`, }, }), String.raw`D:\fixture-local\monochromatic\Log`);
    for (const configured of [undefined, '', 'relative',]) {
      assert.equal(userLogDirectory({ platform: 'win32', home: String.raw`C:\fixture-home`,
        environment: { LOCALAPPDATA: configured, }, }), String.raw`C:\fixture-home\AppData\Local\monochromatic\Log`);
    }
  } }),
] });
//endregion
