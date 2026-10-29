/**
 Tests for `selectLaunchCommand`, the pure new-instance command selection:
 override table first (case-insensitive), then the desktop file, then the bare
 resource class, then nothing.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NoAppIdentityError } from './errors.ts';
import { selectLaunchCommand } from './launch.ts';

await describe({
  name: selectLaunchCommand.name,
  children: [
    it({
      name: 'uses the ghostty override for a fresh process',
      fn: async () => {
        expect(selectLaunchCommand({ desktopFileName: '', resourceClass: 'ghostty' })).toEqual({
          cmd: 'ghostty',
          args: ['--gtk-single-instance=false'],
        });
      },
    }),
    it({
      name: 'matches the override by the reverse-DNS resource class too',
      fn: async () => {
        expect(
          selectLaunchCommand({ desktopFileName: '', resourceClass: 'com.mitchellh.ghostty' }),
        ).toEqual({ cmd: 'ghostty', args: ['--gtk-single-instance=false'] });
      },
    }),
    it({
      name: 'matches overrides case-insensitively',
      fn: async () => {
        expect(selectLaunchCommand({ desktopFileName: '', resourceClass: 'Firefox' })).toEqual({
          cmd: 'firefox',
          args: ['--new-window'],
        });
      },
    }),
    it({
      name: 'falls back to the desktop file via kstart',
      fn: async () => {
        expect(
          selectLaunchCommand({ desktopFileName: 'org.kde.konsole', resourceClass: 'konsole' }),
        ).toEqual({ cmd: 'kstart', args: ['--application', 'org.kde.konsole'] });
      },
    }),
    it({
      name: 'falls back to the bare resource class via kstart',
      fn: async () => {
        expect(selectLaunchCommand({ desktopFileName: '', resourceClass: 'someapp' })).toEqual({
          cmd: 'kstart',
          args: ['someapp'],
        });
      },
    }),
    it({
      name: 'throws NoAppIdentityError when no app identity is provided',
      fn: async () => {
        expect(() => selectLaunchCommand({ desktopFileName: '', resourceClass: '' })).toThrow(
          NoAppIdentityError,
        );
      },
    }),
  ],
});
