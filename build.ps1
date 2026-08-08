# Build DevProjectsCleaner and copy the binary into .\dist.
#
#   powershell -ExecutionPolicy Bypass -File .\build.ps1
$ErrorActionPreference = "Stop"

$HOST = (rustc -vV | Select-String '^host:').ToString().Split(' ')[1]

New-Item -ItemType Directory -Force -Path "dist" | Out-Null

Write-Host "==> Building for $HOST"
cargo build --release --target $HOST

$BIN = "DevProjectsCleaner"
if ($HOST -like "*windows*") { $BIN = "DevProjectsCleaner.exe" }

Copy-Item "target\$HOST\release\$BIN" "dist\DevProjectsCleaner-$HOST$([IO.Path]::GetExtension($BIN))" -Force
Write-Host "    -> dist\DevProjectsCleaner-$HOST"
Write-Host "Done. Binary is in .\dist"
