$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

New-Item -ItemType Directory -Path "$root\releases" -Force | Out-Null

$version = (Select-String -Path "$root\backend\Cargo.toml" -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value

# 1. 编译前端
Push-Location "$root\frontend"
pnpm run build
Pop-Location

# 2. 复制前端 dist/* 完整到 panel/dist/*
$panelDir = "$root\panel"
New-Item -ItemType Directory -Path "$panelDir\dist" -Force | Out-Null
Remove-Item -Path "$panelDir\dist\*" -Recurse -Force -ErrorAction SilentlyContinue
Copy-Item -Path "$root\frontend\dist\*" -Destination "$panelDir\dist\" -Recurse -Force
Write-Host "Copied frontend dist -> panel/dist"

# 3. targets 从 .cargo/config.toml 读取
$config = Get-Content "$root\backend\.cargo\config.toml" -Raw
$targets = [regex]::Match($config, '(?m)^\s*target = \[([^\]]+)\]').Groups[1].Value -split ',' | ForEach-Object { $_.Trim().Trim('"') }

# 4. for target: 编译后端 + 复制 alpanel + 打包 panel 整个目录
foreach ($t in $targets) {
    Write-Host "Building backend for $t"
    Push-Location "$root\backend"
    cargo build --release --target $t
    Pop-Location

    $bin = "$root\backend\target\$t\release\alpanel"
    Copy-Item -Path $bin -Destination "$panelDir\alpanel" -Force

    $out = "$root\releases\alpanel-$version-$t.tar.gz"
    & tar -czf $out -C $panelDir .
    Write-Host "Packaged: $out"
}
