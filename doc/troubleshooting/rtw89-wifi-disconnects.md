# Realtek `rtw89` scan-adjacent Wi-Fi disconnects

## Status

A Bazzite host using an RTL8852CE adapter intermittently lost a strong Wi-Fi connection after a scan.
The durable local mitigation is to disable Wi-Fi power saving through IWD and let NetworkManager own reconnection.

The mitigation survived an IWD restart and restored connectivity without manual activation.
It is verified as a workaround,
 not as proof that the adapter,
 firmware,
 or access point contains a specific defect.
The leading hypothesis is that an RTL8852C power-save and scan interaction contributed to the reported beacon loss.

Do not infer stability from an empty or incomplete journal interval.
The user reported frequent disconnects with the earlier `wpa_supplicant` backend,
 but no retained correlated capture establishes whether they had the same mechanism.
Changing to IWD therefore does not establish when the problem began.

## Affected environment

The diagnosed host had:

- Bazzite `44.20260721.0`.
- Kernel `7.1.3-ogc5.1.fc44.x86_64`,
   based on OGC Linux tag `v7.1.3-ogc5`.
- NetworkManager `1.56.1-2.fc44`.
- IWD `3.10-1.fc44.bazzite`.
- Realtek RTL8852CE PCI device `10ec:c852`,
   driven by `rtw89_8852ce`.
- RTL8852C firmware `0.27.129.4`.
- An 802.11ax connection at 5660 MHz with signal near `-44 dBm` before the captured failure.

## Symptoms

The user-visible symptom was a spontaneous loss of network connectivity.
The connection could sometimes be restored manually,
 but reconnect attempts could first encounter:

- association status `30`,
   meaning the access point rejected association temporarily;
- disconnect reason `4`,
   `DISASSOC_DUE_TO_INACTIVITY`;
- disconnect reason `15`,
   `4WAY_HANDSHAKE_TIMEOUT`.

Those values describe stages in recovery.
They do not,
 by themselves,
 identify the event that caused the original link loss.

## Diagnostic capture

The useful capture correlated four independent boundaries:

- NetworkManager and IWD journals;
- kernel messages;
- `nl80211` link and scan events from `iw`;
- continuous gateway and internet pings.

Representative commands were:

```sh
sudo journalctl --follow --output=short-monotonic --unit=NetworkManager.service --unit=iwd.service
sudo journalctl --follow --output=short-monotonic --dmesg
sudo iw event -t -f
ping <gateway-address>
ping <internet-address>
```

NetworkManager Wi-Fi trace logging was enabled only for diagnosis:

```sh
sudo nmcli general logging level TRACE domains WIFI:TRACE,DEVICE:TRACE,PLATFORM:TRACE,CORE:DEBUG
```

After capture,
 logging was returned to its normal runtime level:

```sh
sudo nmcli general logging level INFO domains DEFAULT
```

The correlated failure proceeded as follows:

1.  IWD started a full scan while the station remained associated.
2.  Gateway and internet replies stopped during the scan.
3.  The scan completed.
4.  About six seconds after scan completion,
     and roughly eleven seconds after scan start,
     the driver reported a CQM beacon-loss event despite the preceding strong signal.
5.  mac80211 sent connection probes and locally disconnected with reason `4` when they received no acknowledgement.
6.  The first recovery attempts encountered temporary association rejection and a four-way-handshake timeout.
7.  A later activation succeeded.

A full scan is not proof of a fault because successful connections also scan.
The diagnostic value is the ordering of scan,
 reachability loss,
 beacon loss,
 and local disconnection.

JEV:
 Wi-Fi drop debugging:
 reproduce with live link,
 supplicant,
 kernel,
 and reachability capture;
 journal silence or later recovery isn't cause.

## Source-level explanation

### Driver and mac80211 path

The inspected OGC Linux source explains the captured sequence:

- `drivers/net/wireless/realtek/rtw89/fw.c` builds a hardware-scan plan that periodically returns to the operating
  channel when cumulative off-channel time exceeds its limit.
- `drivers/net/wireless/realtek/rtw89/mac.c` turns the firmware beacon-loss notification into
  `ieee80211_beacon_loss()` after scan and off-channel guards permit it.
- `net/mac80211/mlme.c` polls the access point after connection loss and disconnects with
  `WLAN_REASON_DISASSOC_DUE_TO_INACTIVITY` when the probes are not acknowledged.

This is why reason `4` is a local consequence of failed reachability checks,
 not evidence that the access point
first chose to disconnect the station.

RTL8852C firmware `0.27.129.4` is new enough for both mitigations recognized by the inspected driver:

- `BEACON_LOSS_COUNT_V1`,
   enabled for RTL8852C firmware `0.27.128.0` or newer;
- `BEACON_TRACKING`,
   enabled for RTL8852C firmware `0.27.129.1` or newer.

The driver configures `RTW89_BCN_LOSS_CNT` as `60`.
With the observed 100 TU beacon interval,
 that corresponds to about 6.1 seconds.
This is consistent with the measured gap from scan completion to beacon loss,
 but does not prove when the timer began or what caused the missed beacons.
The beacon-tracking path operates only while low-power state is enabled.
The installed kernel and firmware therefore already contain the known tolerance and tracking mechanisms,
but the failure still occurred while power saving was on.

### IWD path

IWD receives the kernel CQM beacon-loss event and schedules its station roaming response.
The inspected `src/netdev.c` and `src/station.c` paths do not originate the driver's beacon-loss decision.
This supports retaining IWD rather than treating the backend as the root cause.

IWD also owns the device power-save setting when it manages the Wi-Fi interface.
Its `PowerSaveDisable` driver quirk is matched against the kernel driver name and disables power saving through
`nl80211` during device setup.

### Confidence and remaining uncertainty

The causal confidence is moderate:

- The failure followed a scan and reported beacon loss at strong signal.
- Power saving was unexpectedly on before the failure.
- After the persistent workaround and IWD restart,
   twelve full roam scans completed without another beacon-loss event or disconnect.
- A related `rtw89` report describes beacon loss caused by station and access-point disagreement about power save,
  with driver power-save disablement as the workaround.

The capture does not prove whether the initiating defect is in RTL8852C firmware,
 the Linux driver,
 or the access
point's handling of power-save state.
A short observation without recurrence is supporting evidence,
 not proof of permanent stability.

## Durable workaround

### Disable power saving in IWD

Add or merge this section in `/etc/iwd/main.conf`:

```ini
[DriverQuirks]
PowerSaveDisable=rtw89_8852ce
```

The value must match the kernel driver name reported by `ethtool --driver <interface>`.
On the diagnosed host,
 IWD logged both the driver-quirk match and power-save disablement after restart.

The existing NetworkManager setting did not provide this result:

```ini
[connection]
wifi.powersave=2
```

While that setting was present,
 `iw dev <interface> get power_save` still reported `on` under the IWD backend.
The IWD driver quirk addresses the owner that actually configures this interface.

Disabling power saving can increase energy use.
That tradeoff is appropriate for a stationary or externally powered host where reliable connectivity is preferred.

### Let NetworkManager own autoconnect

Set `/etc/NetworkManager/conf.d/iwd.conf` to include:

```ini
[device]
wifi.backend=iwd
wifi.iwd.autoconnect=no
```

NetworkManager `1.56.1` defaults `wifi.iwd.autoconnect` to `yes`.
In that mode,
 IWD controls ranking and autoconnection,
 and NetworkManager ignores connection settings including
`autoconnect-priority` and `autoconnect-retries`.
Setting it to `no` lets the NetworkManager profile's retry policy take effect while retaining IWD as the backend.

Configure the saved profile to reconnect indefinitely:

```sh
sudo nmcli connection modify <connection-name-or-uuid> \
  connection.autoconnect yes \
  connection.autoconnect-retries 0
```

A retry value of `0` means forever.
This can produce repeated authentication attempts and logs if credentials or access-point policy remain invalid.

Apply the daemon configurations from a local session because service restarts temporarily interrupt networking:

```sh
sudo systemctl restart iwd.service
sudo systemctl restart NetworkManager.service
```

## Verification

Verify live power state and persistent configuration:

```sh
iw dev <interface> get power_save
nmcli --get-values connection.autoconnect,connection.autoconnect-retries connection show \
  <connection-name-or-uuid>
systemctl --no-pager --full status iwd.service NetworkManager.service
```

Expected results are:

```text
Power save: off
yes
0
```

Verify the end-user boundary rather than relying on daemon startup alone:

```sh
nmcli --terse --fields GENERAL.STATE,GENERAL.CONNECTION device show <interface>
ping -c 3 <gateway-address>
ping -c 3 <internet-address>
```

For restart persistence,
 restart only IWD and observe recovery without activating the profile manually:

```sh
sudo systemctl restart iwd.service
```

The diagnosed host briefly showed the Wi-Fi device as unavailable while IWD re-registered it.
NetworkManager then selected the saved profile automatically,
 completed association,
 restored global connectivity,
and left `iw` power saving off.
Temporary unavailability during backend restart is not a failed verification if automatic recovery completes.

## Approaches that did not resolve the problem

### Changing the Wi-Fi backend

The user reported that the earlier `wpa_supplicant` backend also disconnected frequently.
An absence of retained journal entries from that period cannot establish that it was stable or that its failures had
 the same mechanism.
Retain IWD unless a new,
 correlated comparison demonstrates a backend-specific failure.

### NetworkManager power-save configuration alone

`wifi.powersave=2` was already configured,
 but the live IWD-managed interface reported power saving as on.
Use IWD's `PowerSaveDisable` driver quirk for this backend and verify live state with `iw`.

### Retry settings while IWD owns autoconnect

`connection.autoconnect-retries=0` does not control IWD's autonomous reconnection while
`wifi.iwd.autoconnect=yes`.
Set `wifi.iwd.autoconnect=no` when NetworkManager profile retry semantics are required.

### Treating recovery errors as the original cause

Association status `30` and reason `15` occurred after the local beacon-loss disconnection.
They describe failures during immediate recovery attempts,
 not why the established connection first vanished.

## Rollback

To restore power saving,
 remove only this entry from `/etc/iwd/main.conf` and restart IWD:

```ini
PowerSaveDisable=rtw89_8852ce
```

To return autoconnection to IWD,
 set:

```ini
wifi.iwd.autoconnect=yes
```

Then restart NetworkManager.
When IWD owns autoconnection,
 do not expect NetworkManager's per-profile priority and retry settings to apply.

## Upstream status

No new upstream report was filed during this diagnosis.
The issue is actionable locally,
 but an upstream report still lacks reproduction on an unmodified current upstream
kernel and isolation from access-point behavior.

The closest known report is `lwfinger/rtw89` issue `121` for RTL8852AE.
It describes strong-signal beacon loss across access points,
 attributes the behavior to power-save disagreement,
and recommends disabling driver power saving.
It is a related predecessor,
 not an exact duplicate for RTL8852CE.
That repository directs in-kernel driver reports to Linux wireless maintainers rather than using it as the canonical
issue tracker.

The inspected OGC kernel already includes RTL8852C beacon-loss count version 1,
 beacon tracking,
 beacon timeout
calculation,
 and beacon diagnostics.
Those mechanisms reduce known failure modes but did not prevent this captured event with power saving enabled.

A future report should go to `linux-wireless@vger.kernel.org`,
 copying the `rtw89` maintainer listed in
`MAINTAINERS`.
Before sending it,
 reproduce on a current upstream kernel and include:

- exact adapter PCI ID,
   driver,
   firmware,
   kernel,
   IWD,
   and NetworkManager versions;
- access-point model,
   firmware,
   security mode,
   band,
   channel,
   and beacon interval;
- correlated kernel,
   IWD,
   NetworkManager,
   `iw event -t -f`,
   gateway,
   and internet timelines;
- live power-save state before and after the workaround;
- results with power saving on and off across the same scan trigger;
- whether the failure reproduces across more than one access point;
- `rtw89` `beacon_info` and `phy_info` debugfs output when available;
- kernel taint state and a statement that the same issue reproduces without out-of-tree modules.

## Sources

- [IWD `3.10` configuration reference][iwd-config]
- [IWD `3.10` beacon-loss handling][iwd-beacon]
- [NetworkManager `1.56.1` IWD backend configuration][nm-iwd-config]
- [OGC Linux `v7.1.3-ogc5` RTL8852C driver source][ogc-rtw89]
- [OGC Linux `v7.1.3-ogc5` mac80211 connection polling][ogc-connection-poll]
- [`rtw89` issue `121`][rtw89-issue-121]
- [Linux wireless issue-reporting guide][linux-reporting]

[iwd-config]: https://kernel.googlesource.com/pub/scm/network/wireless/iwd/+/refs/tags/3.10/src/iwd.config.rst
[iwd-beacon]: https://kernel.googlesource.com/pub/scm/network/wireless/iwd/+/refs/tags/3.10/src/station.c
[nm-iwd-config]: https://github.com/NetworkManager/NetworkManager/blob/1.56.1/man/NetworkManager.conf.xml
[ogc-rtw89]: https://github.com/OpenGamingCollective/linux/tree/v7.1.3-ogc5/drivers/net/wireless/realtek/rtw89
[ogc-connection-poll]: https://github.com/OpenGamingCollective/linux/blob/v7.1.3-ogc5/net/mac80211/mlme.c
[rtw89-issue-121]: https://github.com/lwfinger/rtw89/issues/121
[linux-reporting]: https://docs.kernel.org/admin-guide/reporting-issues.html
