param(
    [string]$UnityEditor = 'C:/Program Files/Unity/Hub/Editor/6000.3.29f1/Editor/Unity.exe'
)

$ErrorActionPreference = 'Stop'
$project = (Resolve-Path -LiteralPath $PSScriptRoot).Path
$repository = (Resolve-Path -LiteralPath (Join-Path $project '../..')).Path
$target = (New-Item -ItemType Directory -Force -Path (Join-Path $repository 'target')).FullName
$release = 'water-lab-release-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8)
$output = Join-Path $target $release
$logs = (New-Item -ItemType Directory -Path (Join-Path $target "$release-checks")).FullName
$editor = (Resolve-Path -LiteralPath $UnityEditor).Path

function Invoke-WaterEditor([string]$method, [string]$logName, [string[]]$options) {
    $log = Join-Path $logs $logName
    $arguments = @('-batchmode', '-projectPath', ('"{0}"' -f $project), '-executeMethod', $method,
        '-logFile', ('"{0}"' -f $log)) + $options
    $process = Start-Process -FilePath $editor -ArgumentList $arguments -WindowStyle Hidden -PassThru -Wait
    if ($process.ExitCode -ne 0) { throw "Unity $method failed. See $log." }
    return $log
}

$previousOutput = $env:MIRA_WATER_HDRP_OUTPUT
$previousCapture = $env:MIRA_WATER_HDRP_CAPTURE
try {
    & (Join-Path $project 'sync-shared.ps1')
    $env:MIRA_WATER_HDRP_OUTPUT = $output
    $env:MIRA_WATER_HDRP_CAPTURE = $null
    $build = Invoke-WaterEditor 'HdrpWaterLabBuild.PrepareAndBuild' 'build.log' @('-nographics', '-quit')
    if (!(Select-String -LiteralPath $build -SimpleMatch 'MIRA HDRP WATER PLAYER OK' -Quiet)) {
        throw "Build did not complete. See $build."
    }
    $env:MIRA_WATER_HDRP_CAPTURE = Join-Path $logs 'captures'
    $verify = Invoke-WaterEditor 'HdrpWaterLabAcceptance.Start' 'acceptance.log' @('-force-d3d12')
    if (!(Select-String -LiteralPath $verify -SimpleMatch 'MIRA WATER RENDERER ROUND TRIP OK' -Quiet)) {
        throw "Water acceptance did not complete. See $verify."
    }
    $env:MIRA_WATER_HDRP_CAPTURE = $null
    foreach ($file in @('WaterLab.exe', 'UnityPlayer.dll', 'WaterLab_Data/globalgamemanagers',
        'WaterLab_Data/Managed/Assembly-CSharp.dll', 'water-lab-inputs.sha256')) {
        if (!(Test-Path -LiteralPath (Join-Path $output $file) -PathType Leaf)) {
            throw "Release is missing $file. See $build."
        }
    }

    $log = Join-Path $logs 'player.log'
    $playerArguments = @('-batchmode', '-force-d3d12', '-logFile', ('"{0}"' -f $log))
    $player = Start-Process -FilePath (Join-Path $output 'WaterLab.exe') -WindowStyle Hidden -PassThru -ArgumentList $playerArguments
    try {
        $deadline = (Get-Date).AddSeconds(30)
        do {
            if ($player.HasExited) { throw "Player exited early. See $log." }
            $loaded = (Test-Path -LiteralPath $log) -and (Select-String -LiteralPath $log -SimpleMatch 'UnloadTime:' -Quiet)
            if (!$loaded) { Start-Sleep -Milliseconds 250 }
        } while (!$loaded -and (Get-Date) -lt $deadline)
        if (!$loaded -or (Select-String -LiteralPath $log -Pattern 'Exception:|Shader error' -Quiet)) {
            throw "Player startup failed. See $log."
        }
    } finally {
        if (!$player.HasExited) { $player.Kill(); $player.WaitForExit() }
    }

    $manifest = Join-Path $target 'water-lab-current.txt'
    $stagedManifest = Join-Path $logs 'current.tmp'
    [IO.File]::WriteAllText($stagedManifest, $release + [Environment]::NewLine)
    if (Test-Path -LiteralPath $manifest) {
        [IO.File]::Replace($stagedManifest, $manifest, (Join-Path $logs 'previous-release.txt'))
    } else {
        [IO.File]::Move($stagedManifest, $manifest)
    }
    Write-Output "Published $output"
    Write-Output "Checks: $logs"
} finally {
    $env:MIRA_WATER_HDRP_OUTPUT = $previousOutput
    $env:MIRA_WATER_HDRP_CAPTURE = $previousCapture
}
