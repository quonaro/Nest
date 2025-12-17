param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [object[]]$RemainingArgs
)

# Legacy-style static installer name for Windows.
# This script simply delegates to install.ps1, preserving all arguments
# (including -Version if provided).

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$TargetScript = Join-Path $ScriptDir "install.ps1"

if (-not (Test-Path $TargetScript)) {
    Write-Error "Error: install.ps1 not found in $ScriptDir"
    exit 1
}

& $TargetScript @RemainingArgs


