# Runs each native DLL's loader in a separate process: a DLL that crashes
# during initialization must not prevent the remaining probes from running.
#Requires -Version 7
param([Parameter(Mandatory = $true)][string]$Bundle, [string]$Library = '')
$ErrorActionPreference = 'Stop'
$bundlePath = (Resolve-Path $Bundle).Path
if ($Library) {
  Add-Type -Namespace NativeProbe -Name Loader -MemberDefinition @'
[DllImport("kernel32.dll")] public static extern uint SetErrorMode(uint mode);
[DllImport("kernel32.dll")] public static extern IntPtr AddVectoredExceptionHandler(uint first, Handler handler);
[DllImport("kernel32.dll", CharSet = CharSet.Unicode)] public static extern bool GetModuleHandleExW(uint flags, IntPtr address, out IntPtr module);
[DllImport("kernel32.dll", CharSet = CharSet.Unicode)] public static extern uint GetModuleFileNameW(IntPtr module, System.Text.StringBuilder path, int size);
public delegate int Handler(IntPtr pointers);
private static Handler handler = Report;
public static void InstallReporter() { AddVectoredExceptionHandler(1, handler); }
private static int Report(IntPtr pointers) {
  IntPtr record = Marshal.ReadIntPtr(pointers);
  uint code = unchecked((uint)Marshal.ReadInt32(record));
  if (code == 0xc0000005 || code == 0xc000001d) {
    IntPtr address = Marshal.ReadIntPtr(record, 16);
    IntPtr module;
    var path = new System.Text.StringBuilder(1024);
    if (GetModuleHandleExW(6, address, out module)) {
      GetModuleFileNameW(module, path, path.Capacity);
      Console.Error.WriteLine("Native fault 0x{0:X8} in {1} + 0x{2:X}", code, path, address.ToInt64() - module.ToInt64());
    }
  }
  return 0;
}
[DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)] public static extern IntPtr LoadLibraryW(string name);
[DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)] public static extern bool SetDllDirectoryW(string path);
'@
  [void][NativeProbe.Loader]::SetErrorMode(3)
  [NativeProbe.Loader]::InstallReporter()
  [void][NativeProbe.Loader]::SetDllDirectoryW($bundlePath)
  $handle = [NativeProbe.Loader]::LoadLibraryW((Join-Path $bundlePath $Library))
  if ($handle -eq [IntPtr]::Zero) {
    throw "LoadLibrary($Library) failed: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
  }
  Write-Host "Loaded $Library"
  exit 0
}
Write-Host "Probe process architecture: $([Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)"
if ([Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -ne 'Arm64') { throw 'Expected a native ARM64 probe process' }
$pwsh = (Get-Process -Id $PID).Path
$failures = @()
foreach ($dll in Get-ChildItem $bundlePath -Filter *.dll | Where-Object { $_.Name -match "^(av|sw|onnxruntime|sherpa-)" } | Sort-Object Name) {
  Write-Host "Probing $($dll.Name)"
  $info = [Diagnostics.ProcessStartInfo]::new($pwsh)
  foreach ($arg in @('-NoProfile', '-File', $PSCommandPath, '-Bundle', $bundlePath, '-Library', $dll.Name)) { [void]$info.ArgumentList.Add($arg) }
  $info.UseShellExecute = $false
  $info.Environment['PATH'] = "$env:SystemRoot\System32;$env:SystemRoot"
  $process = [Diagnostics.Process]::Start($info)
  if (-not $process.WaitForExit(60000)) { $process.Kill($true); $failures += "$($dll.Name): loader timed out"; continue }
  Write-Host ("{0}: exit 0x{1:X8}" -f $dll.Name, $process.ExitCode)
  if ($process.ExitCode -ne 0) { $failures += ("{0}: 0x{1:X8}" -f $dll.Name, $process.ExitCode) }
}
if ($failures.Count) { throw ($failures -join "`n") }
