param(
    [Parameter(Mandatory)][string] $Commit,
    [Parameter(Mandatory)][string] $TargetDirectory,
    [Parameter(Mandatory)][string] $Stage
)

$ErrorActionPreference = 'Stop'
if (!$IsWindows) { throw 'This preflight builds Windows executables.' }

$source = Split-Path -Parent $PSScriptRoot
$head = git -C $source rev-parse HEAD
if ($LASTEXITCODE -ne 0 -or $Commit -cnotmatch '^[0-9a-f]{40}$' -or $head -cne $Commit) {
    throw 'Pass the exact checked-out source commit.'
}
$changes = git -C $source status --porcelain
if ($LASTEXITCODE -ne 0 -or $changes) { throw 'The source checkout must be clean.' }

foreach ($path in @($TargetDirectory, $Stage)) {
    if (![IO.Path]::IsPathFullyQualified($path)) { throw 'Use absolute output paths.' }
    $relative = [IO.Path]::GetRelativePath($source, [IO.Path]::GetFullPath($path))
    if ($relative -ne '..' -and !$relative.StartsWith('..\') -and ![IO.Path]::IsPathFullyQualified($relative)) {
        throw 'Build and staging output must be outside the source checkout.'
    }
}
if (Test-Path -LiteralPath $Stage) { throw 'Use a new staging directory.' }

Push-Location $source
try {
    cargo build --target-dir $TargetDirectory --release --workspace --all-features --bins --locked
    if ($LASTEXITCODE -ne 0) { throw 'Locked workspace build failed.' }
} finally {
    Pop-Location
}

$binaries = Join-Path $TargetDirectory 'release'
$front = Join-Path $binaries 'git-asset.exe'
$surface = & $front describe | ConvertFrom-Json
if ($LASTEXITCODE -ne 0 -or $surface.product -ne 'git-asset') {
    throw 'Front door did not identify a Git-Asset build.'
}
$files = @('git-asset.exe')
foreach ($service in $surface.commands) {
    if ($service -cnotmatch '^[a-z]+$') { throw 'Invalid advertised service name.' }
    $files += "git-asset-$service.exe"
}
foreach ($name in $files) {
    if (!(Test-Path -LiteralPath (Join-Path $binaries $name) -PathType Leaf)) {
        throw "Missing advertised executable: $name"
    }
}

New-Item -ItemType Directory -Path $Stage | Out-Null
foreach ($name in $files) { Copy-Item -LiteralPath (Join-Path $binaries $name) -Destination $Stage }
foreach ($name in @('LICENSE','NOTICE.md','Cargo.lock')) {
    Copy-Item -LiteralPath (Join-Path $source $name) -Destination $Stage
    $files += $name
}
$stagedFront = Join-Path $Stage 'git-asset.exe'
foreach ($service in @('sample','storage','query')) {
    $report = & $stagedFront $service describe | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0 -or !$report.schema) { throw "Staged $service dispatch failed." }
}
$manifest = @($files | Sort-Object | ForEach-Object {
    $file = Join-Path $Stage $_
    [ordered]@{
        path = $_
        bytes = (Get-Item -LiteralPath $file).Length
        sha256 = (Get-FileHash -LiteralPath $file).Hash.ToLowerInvariant()
    }
})
[ordered]@{
    schema = 'git-asset.windows-bundle-preflight.experimental-1'
    source_commit = $Commit
    version = $surface.version
    frontend_schema = $surface.schema
    native_storage_available = ($surface.native_storage_available -eq $true)
    files = $manifest
} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $Stage 'preflight-manifest.json') -Encoding utf8
