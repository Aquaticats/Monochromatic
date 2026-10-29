/**
 Windows PE settings pass for Autounattend.xml generation.
 Produces the `windowsPE` section with internationalization, disk
 partitioning, image selection, and VirtIO driver loading configuration.
 */

import {
  COMPONENT_ATTRS,
  pnpDriverPaths,
} from './autounattend-virtio.ts';

/**
 Generates the windowsPE settings pass for Autounattend.xml.
 Configures locale, MBR disk partitioning, WIM image selection by index,
 and PnP driver paths via {@link pnpDriverPaths} for VirtIO storage and
 network during WinPE.
 
 @param imageIndex - WIM image index for OS edition selection
 
 @returns XML string for the complete `<settings pass="windowsPE">` block
 
 @example
 ```ts
 windowsPeSection({ imageIndex: 1 }); // => '  <settings pass="windowsPE">...'
 ```
 */
export function windowsPeSection({ imageIndex, }: { readonly imageIndex: number; },): string {
  return `  <settings pass="windowsPE">
    <component name="Microsoft-Windows-International-Core-WinPE" ${COMPONENT_ATTRS}>
      <SetupUILanguage>
        <UILanguage>en-US</UILanguage>
      </SetupUILanguage>
      <InputLocale>en-US</InputLocale>
      <SystemLocale>en-US</SystemLocale>
      <UILanguage>en-US</UILanguage>
      <UserLocale>en-US</UserLocale>
    </component>
    <component name="Microsoft-Windows-Setup" ${COMPONENT_ATTRS}>
      <DiskConfiguration>
        <WillShowUI>OnError</WillShowUI>
        <Disk wcm:action="add">
          <DiskID>0</DiskID>
          <WillWipeDisk>true</WillWipeDisk>
          <CreatePartitions>
            <CreatePartition wcm:action="add">
              <Order>1</Order>
              <Size>100</Size>
              <Type>Primary</Type>
            </CreatePartition>
            <CreatePartition wcm:action="add">
              <Order>2</Order>
              <Extend>true</Extend>
              <Type>Primary</Type>
            </CreatePartition>
          </CreatePartitions>
          <ModifyPartitions>
            <ModifyPartition wcm:action="add">
              <Order>1</Order>
              <PartitionID>1</PartitionID>
              <Active>true</Active>
              <Format>NTFS</Format>
              <Label>System</Label>
            </ModifyPartition>
            <ModifyPartition wcm:action="add">
              <Order>2</Order>
              <PartitionID>2</PartitionID>
              <Format>NTFS</Format>
              <Label>Windows</Label>
              <Letter>C</Letter>
            </ModifyPartition>
          </ModifyPartitions>
        </Disk>
      </DiskConfiguration>
      <ImageInstall>
        <OSImage>
          <InstallFrom>
            <MetaData wcm:action="add">
              <Key>/IMAGE/INDEX</Key>
              <Value>${String(imageIndex,)}</Value>
            </MetaData>
          </InstallFrom>
          <InstallTo>
            <DiskID>0</DiskID>
            <PartitionID>2</PartitionID>
          </InstallTo>
        </OSImage>
      </ImageInstall>
      <UserData>
        <AcceptEula>true</AcceptEula>
      </UserData>
    </component>
${pnpDriverPaths()}
  </settings>`;
}
