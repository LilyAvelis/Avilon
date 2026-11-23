param(
    [switch]$Explained,
    [switch]$ReplaceKey,
    [ValidateSet('gpt-realtime-2.1-mini', 'gpt-realtime-2.1')]
    [string]$SampleModel,
    [switch]$English
)

$ErrorActionPreference = 'Stop'
$credentialDirectory = Join-Path $env:LOCALAPPDATA 'Mira'
$credentialFile = Join-Path $credentialDirectory 'openai-key.dpapi'
$saveKey = $ReplaceKey -or -not (Test-Path -LiteralPath $credentialFile)
$secret = if ($saveKey) {
    Read-Host 'OpenAI project API key (hidden)' -AsSecureString
} else {
    Get-Content -LiteralPath $credentialFile -Raw | ConvertTo-SecureString
}
$buffer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secret)
try {
    $key = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($buffer)
    if ([string]::IsNullOrWhiteSpace($key)) { throw 'No API key entered.' }
    $mode = if ($SampleModel) { 'sample' } elseif ($Explained) { 'explained' } else { 'short' }
    $arguments = @((Join-Path $PSScriptRoot 'record-voice.cjs'), $mode)
    if ($SampleModel) { $arguments += $SampleModel }
    if ($SampleModel -and $English) { $arguments += 'en' }
    $key.Trim() | node @arguments
    if ($LASTEXITCODE -ne 0) { throw 'Voice generation failed. See the API error above.' }
    if ($saveKey) {
        [void][IO.Directory]::CreateDirectory($credentialDirectory)
        $encrypted = ConvertFrom-SecureString -SecureString $secret
        [IO.File]::WriteAllText($credentialFile, $encrypted)
        Write-Host 'API key saved encrypted for this Windows user.'
    }
} finally {
    [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($buffer)
    Remove-Variable key -ErrorAction SilentlyContinue
    $secret.Dispose()
}
