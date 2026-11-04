
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class CliGitRestartManager {
  [StructLayout(LayoutKind.Sequential)]
  public struct UniqueProcess { public int ProcessId; public System.Runtime.InteropServices.ComTypes.FILETIME StartTime; }
  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
  public struct ProcessInfo {
    public UniqueProcess Process;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 256)] public string AppName;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 64)] public string ServiceShortName;
    public int ApplicationType;
    public uint AppStatus;
    public uint TSSessionId;
    [MarshalAs(UnmanagedType.Bool)] public bool Restartable;
  }
  [DllImport("rstrtmgr.dll", CharSet = CharSet.Unicode)]
  public static extern int RmStartSession(out uint session, int flags, System.Text.StringBuilder key);
  [DllImport("rstrtmgr.dll")]
  public static extern int RmEndSession(uint session);
  [DllImport("rstrtmgr.dll", CharSet = CharSet.Unicode)]
  public static extern int RmRegisterResources(uint session, uint fileCount, string[] files, uint applicationCount, IntPtr applications, uint serviceCount, string[] services);
  [DllImport("rstrtmgr.dll")]
  public static extern int RmGetList(uint session, out uint needed, ref uint count, [In, Out] ProcessInfo[] processes, ref uint rebootReasons);
  public static string Query(string path) {
    uint session;
    int status = RmStartSession(out session, 0, new System.Text.StringBuilder(64));
    if (status != 0) { throw new Exception("RmStartSession failed: " + status); }
    try {
      status = RmRegisterResources(session, 1, new string[] { path }, 0, IntPtr.Zero, 0, null);
      if (status != 0) { throw new Exception("RmRegisterResources failed: " + status); }
      uint needed = 0;
      uint count = 0;
      uint reasons = 0;
      status = RmGetList(session, out needed, ref count, null, ref reasons);
      if (status == 0) { return ""; }
      if (status != 234) { throw new Exception("RmGetList failed: " + status); }
      ProcessInfo[] processes = new ProcessInfo[needed];
      count = needed;
      status = RmGetList(session, out needed, ref count, processes, ref reasons);
      if (status != 0) { throw new Exception("RmGetList failed: " + status); }
      System.Text.StringBuilder lines = new System.Text.StringBuilder();
      for (int index = 0; index < count; index++) {
        lines.Append(processes[index].Process.ProcessId).Append('\t').Append(processes[index].AppName).Append('\n');
      }
      return lines.ToString();
    }
    finally { RmEndSession(session); }
  }
}
'@
[Console]::Out.Write([CliGitRestartManager]::Query($env:CLI_GIT_RM_TARGET))
