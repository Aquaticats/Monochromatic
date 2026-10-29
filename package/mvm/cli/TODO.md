# mvm TODO

## Windows VM support

Windows template baking and VM creation are working end-to-end.
Remaining work to validate and harden the implementation.

### Testing

- Test `mvm clone` with a Windows source VM
- Test `mvm update` to verify it rebuilds the Windows template
- Test `mvm run` (ephemeral) with a Windows VM
- Clean end-to-end test:
   delete cached template,
   run full `mvm create --image windows` through template bake + VM creation + hostname + exec

### Known issues

- `Rename-Computer` requires a reboot to take effect;
   hostname shows the template name until then
- The `exec` subcommand arg parser treats `-`-prefixed PowerShell arguments
   (e.g. `-Path`) as CLI options when passed through `mise run`;
   direct `mvm exec` works correctly
