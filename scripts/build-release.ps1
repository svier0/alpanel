$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

Push-Location "$root\cli"
cargo build --release
Pop-Location

$targets = @(
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl"
)

foreach ($t in $targets) {
    $dir = "$root\cli\target\$t\release"
    $out = "$root\cli\target\$t.tar.gz"
    tar -czf $out -C $dir alp
    Write-Host "Packaged: $out"
}
