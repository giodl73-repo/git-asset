param(
    [Parameter(Mandatory)][string] $Commit,
    [Parameter(Mandatory)][string] $TargetDirectory,
    [Parameter(Mandatory)][string] $Stage,
    [string] $Archive
)

$ErrorActionPreference = 'Stop'
if (!$IsWindows) { throw 'This preflight builds Windows executables.' }

$source = Split-Path -Parent $PSScriptRoot
function Confirm-Source {
    $head = git -C $source rev-parse HEAD
    if ($LASTEXITCODE -ne 0 -or $Commit -cnotmatch '^[0-9a-f]{40}$' -or $head -cne $Commit) {
        throw 'Pass the exact checked-out source commit.'
    }
    $changes = git -C $source status --porcelain
    if ($LASTEXITCODE -ne 0 -or $changes) { throw 'The source checkout must be clean.' }
}
Confirm-Source

foreach ($path in @($TargetDirectory, $Stage, $Archive) | Where-Object { $_ }) {
    if (![IO.Path]::IsPathFullyQualified($path)) { throw 'Use absolute output paths.' }
    $relative = [IO.Path]::GetRelativePath($source, [IO.Path]::GetFullPath($path))
    if ($relative -ne '..' -and !$relative.StartsWith('..\') -and ![IO.Path]::IsPathFullyQualified($relative)) {
        throw 'Build and staging output must be outside the source checkout.'
    }
}
if (Test-Path -LiteralPath $Stage) { throw 'Use a new staging directory.' }
if ($Archive) {
    $parent = Split-Path -Parent $Archive
    if ([IO.Path]::GetExtension($Archive) -ne '.zip' -or (Test-Path -LiteralPath $Archive) -or
        !(Test-Path -LiteralPath $parent -PathType Container)) {
        throw 'Use a new ZIP path in an existing directory.'
    }
}


Push-Location $source
try {
    $rustc = rustc -vV
    if ($LASTEXITCODE -ne 0 -or !$rustc) { throw 'Cannot record the Rust compiler identity.' }
    $cargo = cargo --version
    if ($LASTEXITCODE -ne 0 -or !$cargo) { throw 'Cannot record the Cargo version.' }
    cargo build --target-dir $TargetDirectory --release --workspace --all-features --bins --locked
    if ($LASTEXITCODE -ne 0) { throw 'Locked workspace build failed.' }
} finally {
    Pop-Location
}

Confirm-Source
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
    rustc = @($rustc)
    cargo = $cargo
    frontend_schema = $surface.schema
    native_storage_available = ($surface.native_storage_available -eq $true)
    files = $manifest
} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $Stage 'preflight-manifest.json') -Encoding utf8

if ($Archive) {
    $entries = @($files) + 'preflight-manifest.json'
    $paths = @($entries | ForEach-Object { Join-Path $Stage $_ })
    $pending = Join-Path (Split-Path -Parent $Archive) ("git-asset-" + [Guid]::NewGuid() + ".partial.zip")
    try {
        Compress-Archive -LiteralPath $paths -DestinationPath $pending
        $zip = [IO.Compression.ZipFile]::OpenRead($pending)
        try {
            if ($zip.Entries.Count -ne $entries.Count) { throw 'ZIP entry count differs from staging.' }
            foreach ($name in $entries) {
                $entry = $zip.GetEntry($name)
                $staged = Join-Path $Stage $name
                if (!$entry -or $entry.Length -ne (Get-Item -LiteralPath $staged).Length) {
                    throw "ZIP entry differs in length: $name"
                }
                $stream = $entry.Open()
                try {
                    $actual = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($stream))
                    if ($actual -ne (Get-FileHash -LiteralPath $staged).Hash) {
                        throw "ZIP entry differs in content: $name"
                    }
                } finally {
                    $stream.Dispose()
                }
            }
        } finally {
            $zip.Dispose()
        }
        [IO.File]::Move($pending, $Archive)
    } finally {
        if (Test-Path -LiteralPath $pending) { Remove-Item -LiteralPath $pending }
    }
    Get-FileHash -LiteralPath $Archive
}
