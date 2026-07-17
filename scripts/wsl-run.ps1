# scripts/wsl-run.ps1

Set-Location $PSScriptRoot\..\frontend
bun run build
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Set-Location $PSScriptRoot\..\backend
cargo build --target x86_64-unknown-linux-musl
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$name = "alpanel"
$destDir = "/www/server/panel"
$sourcePath = "$PSScriptRoot\..\backend\target\x86_64-unknown-linux-musl\debug\$name"
$distDir = "$PSScriptRoot\..\frontend\dist"

$drive = $sourcePath[0].ToString().ToLower()
$wslPath = "/mnt/$drive" + ($sourcePath.Substring(2) -replace '\\', '/')

wsl mkdir -p $destDir
wsl cp "$wslPath" "$destDir/$name"
wsl chmod +x "$destDir/$name"

# 复制前端 dist 到 WSL 对应目录（无内嵌前端，需外挂）
$distDrive = $distDir[0].ToString().ToLower()
$distWslPath = "/mnt/$distDrive" + ($distDir.Substring(2) -replace '\\', '/')
wsl mkdir -p "$destDir/dist"
wsl -- bash -c "cp -r '$distWslPath'/. '$destDir/dist/'"

wsl -- bash -c "cd $destDir && exec ./$name"
