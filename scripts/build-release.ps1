$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

New-Item -ItemType Directory -Path "$root\releases" -Force | Out-Null

$version = (Select-String -Path "$root\backend\Cargo.toml" -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value

Push-Location "$root\backend"
cargo build --release
Pop-Location

$config = Get-Content "$root\backend\.cargo\config.toml" -Raw
$targets = [regex]::Match($config, 'target = \[([^\]]+)\]').Groups[1].Value -split ',' | ForEach-Object { $_.Trim().Trim('"') }

foreach ($t in $targets) {
    $dir = "$root\backend\target\$t\release"
    $out = "$root\releases\alpanel-$version-$t.tar.gz"
    & tar -czf $out -C $dir alpanel
    Write-Host "Packaged: $out"
}
