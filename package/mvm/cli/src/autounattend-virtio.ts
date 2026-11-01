/**
 VirtIO driver constants and helpers for Windows unattended installation.
 Generates WinPE driver loading paths and PowerShell commands for
 installing VirtIO drivers and the QEMU guest agent.
 */

/**
 Windows component XML attribute boilerplate shared by all unattend components.
 Contains the processor architecture, public key token, language, version scope,
 and XML namespace declarations required by Windows Setup.
 */
export const COMPONENT_ATTRS =
  'processorArchitecture="amd64" publicKeyToken="31bf3856ad364e35" language="neutral" versionScope="nonSxS" xmlns:wcm="http://schemas.microsoft.com/WMIConfig/2002/State" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"';

/**
 Possible drive letters where the virtio-win ISO may be mounted during WinPE.
 Windows PE assigns letters dynamically based on device enumeration order.
 Including multiple candidates ensures the drivers are found regardless of
 how many CDROMs are attached or in what order.
 */
const VIRTIO_DRIVE_CANDIDATES = [
  'D',
  'E',
  'F',
  'G',
];

/**
 Windows Server 2025 driver subdirectory within the virtio-win ISO.
 Server 2025 shares drivers with Windows 11 (`w11` directory).
 */
const VIRTIO_DRIVER_OS_DIR = 'w11';

/**
 VirtIO driver subdirectories to install from the virtio-win ISO.
 Only the drivers needed for VM operation: storage (disk), network,
 serial (guest agent communication), and QEMU firmware config.
 */
const VIRTIO_DRIVER_DIRS = [
  'viostor',
  'NetKVM',
  'vioserial',
  'viofs',
  'qemufwcfg',
];

/**
 Generates the PnpCustomizationsWinPE component XML for loading VirtIO drivers
 during Windows PE. Searches multiple drive letters for the viostor (storage)
 and NetKVM (network) drivers so the virtio disk is visible during partitioning.
 
 @returns XML string for the PnpCustomizationsWinPE component
 
 @example
 ```ts
 pnpDriverPaths(); // => '<component name="Microsoft-Windows-PnpCustomizationsWinPE" ...>...'
 ```
 */
export function pnpDriverPaths(): string {
  /**
   Storage and network drivers WinPE must load before partitioning becomes possible.
   */
  const driverDirs = [
    'viostor',
    'NetKVM',
  ];
  /**
   Flattened list of `<PathAndCredentials>` entries spanning every driver and candidate letter.
   
   `wcm:keyValue` must be unique per element; derived from the cross-product index of
   driver and letter so each pair gets a deterministic 1-based id.
   */
  const paths = driverDirs.flatMap(function mapDriver(
    driver,
    driverIdx,
  ) {
    return VIRTIO_DRIVE_CANDIDATES.map(function mapLetter(
      letter,
      letterIdx,
    ) {
      /**
       Unique 1-based keyValue derived from the driver/letter cross-product position.
       */
      const keyValue = (driverIdx * VIRTIO_DRIVE_CANDIDATES
        .length) + letterIdx
        + 1;
      return `      <PathAndCredentials wcm:action="add" wcm:keyValue="${
        String(keyValue,)
      }">
        <Path>${letter}:\\${driver}\\${VIRTIO_DRIVER_OS_DIR}\\amd64</Path>
      </PathAndCredentials>`;
    },);
  },);

  return `    <component name="Microsoft-Windows-PnpCustomizationsWinPE" ${COMPONENT_ATTRS}>
      <DriverPaths>
${paths.join('\n',)}
      </DriverPaths>
    </component>`;
}

/**
 Generates a PowerShell command that:
 1. Finds the virtio-win CDROM among candidate drive letters
 2. Imports the Red Hat signing certificate into Trusted Publishers
 
    to suppress driver signing prompts
 3. Installs VirtIO drivers for the correct OS version via `pnputil`
 4. Installs the QEMU guest agent from the standalone MSI
 
 Only installs drivers for the `w11` (Server 2025) platform to avoid
 hash failures from older driver versions.
 
 @returns PowerShell command string for VirtIO driver and guest agent installation
 
 @example
 ```ts
 virtioInstallCommand();
 // => "powershell -NoProfile -Command ..."
 ```
 */
export function virtioInstallCommand(): string {
  /**
   Comma-separated quoted drive letter list passed to `Get-ChildItem -Path` for ISO discovery.
   */
  const driveList = VIRTIO_DRIVE_CANDIDATES
    .map(function formatDriveLetter(d,) {
      return `'${d}:\\'`;
    },)
    .join(',',);
  /**
   pnputil calls for each driver directory, targeting the correct OS version.
   */
  const pnputilCalls = VIRTIO_DRIVER_DIRS
    .map(function formatPnputilCall(dir,) {
      return `pnputil /add-driver (Join-Path $root '${dir}\\${VIRTIO_DRIVER_OS_DIR}\\amd64\\*.inf') /install`;
    },)
    .join('; ',);
  return `powershell -NoProfile -Command "${
    // find the virtio-win CDROM by looking for the viostor directory
    ''}$vd = Get-ChildItem -Path ${driveList} -Directory -Filter 'viostor' -ErrorAction SilentlyContinue | Select-Object -First 1; ${
    // proceed only if virtio-win ISO is found
    ''}if ($vd) { $root = $vd.Parent.FullName; ${
    // import Red Hat certificate to suppress driver signing dialogs
    ''}Get-ChildItem (Join-Path $root 'cert\\*.cer') -Recurse | ForEach-Object { certutil -addstore TrustedPublisher $_.FullName }; ${
    // install VirtIO drivers for Server 2025 (w11) platform only
    ''}${pnputilCalls}; ${
    // install standalone guest agent MSI (provides the QEMU-GA service)
    ''}$ga = Join-Path $root 'guest-agent\\qemu-ga-x86_64.msi'; ${
    // fall back to all-in-one MSI if standalone is absent
    ''}if (-not (Test-Path $ga)) { $ga = Join-Path $root 'virtio-win-gt-x64.msi' }; ${
    // run silent install and wait for completion
    ''}Start-Process msiexec -ArgumentList '/i',$ga,'/qn','/norestart','/log','C:\\virtio-install.log' -Wait }"`;
}
