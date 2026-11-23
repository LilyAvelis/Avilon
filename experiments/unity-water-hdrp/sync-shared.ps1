$ErrorActionPreference = 'Stop'
$source = Join-Path (Split-Path -Parent $PSScriptRoot) 'unity-water-lab/Assets'
$assets = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot 'Assets')).Path
$destination = Join-Path $assets 'Shared'
$scratch = Join-Path $PSScriptRoot ('Library/MiraSharedSync/' + [guid]::NewGuid().ToString('N'))
$staged = Join-Path $scratch 'new'
$previous = Join-Path $scratch 'previous'
New-Item -ItemType Directory -Force -Path $staged | Out-Null
Get-ChildItem -LiteralPath $source -Force | Where-Object { $_.Name -notin @('Editor', 'Editor.meta') } | ForEach-Object {
    Copy-Item -LiteralPath $_.FullName -Destination $staged -Recurse -Force
}
if (Test-Path -LiteralPath $destination) {
    if ((Resolve-Path -LiteralPath $destination).Path -ne $destination) { throw 'Unexpected Shared location.' }
    Move-Item -LiteralPath $destination -Destination $previous
}
try {
    Move-Item -LiteralPath $staged -Destination $destination
} catch {
    if (Test-Path -LiteralPath $previous) { Move-Item -LiteralPath $previous -Destination $destination }
    throw
}
