# scripts/wsl-run.ps1
Set-Location $PSScriptRoot\..\backend
cargo build --target x86_64-unknown-linux-musl
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$name = "alpanel"
$destDir = "/www/server/panel"
$sourcePath = "$PSScriptRoot\..\backend\target\x86_64-unknown-linux-musl\debug\$name"

$drive = $sourcePath[0].ToString().ToLower()
$wslPath = "/mnt/$drive" + ($sourcePath.Substring(2) -replace '\\', '/')

wsl mkdir -p $destDir
wsl cp "$wslPath" "$destDir/$name"
wsl chmod +x "$destDir/$name"
wsl -- bash -c "cd $destDir && exec ./$name"
