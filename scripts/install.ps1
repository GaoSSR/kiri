$ErrorActionPreference = "Stop"

$Platform = [System.Runtime.InteropServices.RuntimeInformation]::OSDescription
$Architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture

Write-Error @"
Kiri Windows installation is not available yet.

Detected platform: $Platform ($Architecture)

The PowerShell installer entry exists so the release channel is planned, but
Windows artifacts and the Windows platform collector have not shipped yet.
Kiri currently supports macOS first.
"@

exit 1
