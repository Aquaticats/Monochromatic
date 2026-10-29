import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  COMPONENT_ATTRS,
  virtioInstallCommand,
} from './autounattend-virtio.ts';
import { windowsPeSection, } from './autounattend-winpe.ts';
import { createIso, } from './iso9660.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Autounattend XML generation

/**
 Generates a complete Windows Autounattend.xml answer file for unattended
 installation of Windows Server from an evaluation ISO on KVM with VirtIO.
 
 The answer file configures:
 - VirtIO storage and network driver loading during WinPE, via {@link windowsPeSection}
 - MBR disk partitioning (system reserved + OS partition) for BIOS boot
 - Unattended OS image selection by WIM index
 - Locale and timezone settings
 - Administrator account with auto-logon
 - QEMU guest agent installation via FirstLogonCommands, running {@link virtioInstallCommand}
 - OOBE bypass for fully automated setup
 
 @param hostname - VM hostname for the specialize pass
 
 @param imageIndex - WIM image index for edition selection
 
 @returns Complete Autounattend.xml content string
 
 @example
 ```ts
 const xml = generateAutounattend({ imageIndex: 1, hostname: 'template-setup' });
 ```
 */
export function generateAutounattend({
  hostname,
  imageIndex,
}: {
  readonly hostname: string;
  readonly imageIndex: number;
},): string {
  return `<?xml version="1.0" encoding="utf-8"?>
<unattend xmlns="urn:schemas-microsoft-com:unattend">
${windowsPeSection({ imageIndex, },)}
  <settings pass="specialize">
    <component name="Microsoft-Windows-Shell-Setup" ${COMPONENT_ATTRS}>
      <ComputerName>${hostname}</ComputerName>
      <TimeZone>UTC</TimeZone>
    </component>
    <component name="Microsoft-Windows-TerminalServices-LocalSessionManager" ${COMPONENT_ATTRS}>
      <fDenyTSConnections>false</fDenyTSConnections>
    </component>
    <component name="Networking-MPSSVC-Svc" ${COMPONENT_ATTRS}>
      <FirewallGroups>
        <FirewallGroup wcm:action="add" wcm:keyValue="RemoteDesktop">
          <Active>true</Active>
          <Group>Remote Desktop</Group>
          <Profile>all</Profile>
        </FirewallGroup>
      </FirewallGroups>
    </component>
  </settings>
  <settings pass="oobeSystem">
    <component name="Microsoft-Windows-Shell-Setup" ${COMPONENT_ATTRS}>
      <OOBE>
        <HideEULAPage>true</HideEULAPage>
        <HideLocalAccountScreen>true</HideLocalAccountScreen>
        <HideOnlineAccountScreens>true</HideOnlineAccountScreens>
        <HideWirelessSetupInOOBE>true</HideWirelessSetupInOOBE>
        <ProtectYourPC>3</ProtectYourPC>
      </OOBE>
      <UserAccounts>
        <AdministratorPassword>
          <Value>mvm</Value>
          <PlainText>true</PlainText>
        </AdministratorPassword>
      </UserAccounts>
      <AutoLogon>
        <Enabled>true</Enabled>
        <Username>Administrator</Username>
        <Password>
          <Value>mvm</Value>
          <PlainText>true</PlainText>
        </Password>
        <LogonCount>5</LogonCount>
      </AutoLogon>
      <FirstLogonCommands>
        <SynchronousCommand wcm:action="add">
          <Order>1</Order>
          <CommandLine>${virtioInstallCommand()}</CommandLine>
          <Description>Install VirtIO drivers and QEMU guest agent via pnputil and MSI</Description>
        </SynchronousCommand>
        <SynchronousCommand wcm:action="add">
          <Order>2</Order>
          <CommandLine>powershell -NoProfile -Command "Set-Service -Name QEMU-GA -StartupType Automatic -ErrorAction SilentlyContinue; Start-Service QEMU-GA -ErrorAction SilentlyContinue"</CommandLine>
          <Description>Enable and start QEMU guest agent service</Description>
        </SynchronousCommand>
        <SynchronousCommand wcm:action="add">
          <Order>3</Order>
          <CommandLine>powershell -NoProfile -Command "Set-Service -Name VirtioFsSvc -StartupType Automatic -ErrorAction SilentlyContinue; Start-Service VirtioFsSvc -ErrorAction SilentlyContinue"</CommandLine>
          <Description>Enable and start VirtIO FS service for host-guest file sharing</Description>
        </SynchronousCommand>
        <SynchronousCommand wcm:action="add">
          <Order>4</Order>
          <CommandLine>powershell -NoProfile -Command "Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled False"</CommandLine>
          <Description>Disable Windows Firewall for VM networking</Description>
        </SynchronousCommand>
      </FirstLogonCommands>
    </component>
  </settings>
</unattend>`;
}

//endregion Autounattend XML generation

//region Autounattend ISO generation

/**
 Creates an ISO9660 image, via {@link createIso}, containing the
 Autounattend.xml answer file rendered by {@link generateAutounattend}.
 Windows PE automatically searches removable media (including CDROMs)
 for the answer file during setup. The ISO is attached as a secondary
 CDROM during Windows template creation.
 
 @param hostname - VM hostname
 
 @param imageIndex - WIM image index for edition selection
 
 @returns ISO9660 image bytes ready to write to disk
 
 @example
 ```ts
 const iso = createAutounattendIso({ imageIndex: 1, hostname: 'setup' });
 ```
 */
export function createAutounattendIso({
  hostname,
  imageIndex,
}: {
  readonly hostname: string;
  readonly imageIndex: number;
},): Uint8Array {
  /**
   Tagged logger so ISO-creation messages name the call site.
   */
  const rl = tagged({
    tag: createAutounattendIso.name,
    l,
  },);
  /**
   Rendered ahead of ISO packing so the encoder operates on a final string.
   */
  const xml = generateAutounattend({
    hostname,
    imageIndex,
  },);
  /**
   Reused for the single XML payload below.
   */
  const encoder = new TextEncoder();

  /**
   Captured before the success log so the bytes are returned after announcement.
   */
  const iso = createIso({
    files: [
      {
        data: encoder.encode(xml,),
        name: 'Autounattend.xml',
      },
    ],
    volumeId: 'OEMDRV',
  },);

  rl.info('created Autounattend ISO',);
  return iso;
}

//endregion Autounattend ISO generation
