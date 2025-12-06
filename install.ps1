# Install script for Nest CLI tool (Windows PowerShell)

$ErrorActionPreference = "Stop"

# Colors and formatting
function Write-ColorOutput($ForegroundColor, $Message) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    Write-Output $Message
    $host.UI.RawUI.ForegroundColor = $fc
}

# Symbols
$CHECK = "✓"
$CROSS = "✗"
$ARROW = "→"
$INFO = "ℹ"
$WARN = "⚠"
$NEST_ICON = "🪺"

# Detect architecture
$ARCH = $env:PROCESSOR_ARCHITECTURE
if ($ARCH -eq "AMD64") {
    $ARCHITECTURE = "x86_64"
} elseif ($ARCH -eq "ARM64") {
    $ARCHITECTURE = "aarch64"
} else {
    Write-ColorOutput Red "Error: Unsupported architecture: $ARCH"
    exit 1
}

$PLATFORM = "windows"
$BINARY_NAME = "nest.exe"

# GitHub repository
# TODO: Update this to your actual GitHub repository (e.g., "username/nest")
$REPO = "quonaro/nest"
$VERSION = "latest"

# Installation directory
$INSTALL_DIR = "$env:USERPROFILE\.local\bin"
$BINARY_PATH = Join-Path $INSTALL_DIR $BINARY_NAME

# Print header
Write-Host ""
Write-ColorOutput Cyan "╔════════════════════════════════════════╗"
Write-ColorOutput Cyan "║  $NEST_ICON Nest CLI Installer              ║"
Write-ColorOutput Cyan "╚════════════════════════════════════════╝"
Write-Host ""

# Print system information
Write-ColorOutput Cyan "$INFO Detected system:"
Write-Host "   $ARROW Platform: " -NoNewline
Write-ColorOutput White "${PLATFORM}-${ARCHITECTURE}"
Write-Host "   $ARROW Install directory: " -NoNewline
Write-ColorOutput White "${INSTALL_DIR}"
Write-Host ""

# Create install directory if it doesn't exist
Write-ColorOutput Cyan "$INFO Preparing installation..."
if (-not (Test-Path $INSTALL_DIR)) {
    New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null
    Write-ColorOutput Green "   $CHECK Created install directory"
} else {
    Write-ColorOutput Green "   $CHECK Install directory exists"
}

# Download binary
$TEMP_DIR = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
$TEMP_FILE = Join-Path $TEMP_DIR "nest-${PLATFORM}-${ARCHITECTURE}.zip"

if ($VERSION -eq "latest") {
    $URL = "https://github.com/${REPO}/releases/latest/download/nest-${PLATFORM}-${ARCHITECTURE}.zip"
} else {
    $URL = "https://github.com/${REPO}/releases/download/v${VERSION}/nest-${PLATFORM}-${ARCHITECTURE}.zip"
}

Write-Host ""
Write-ColorOutput Cyan "$INFO Downloading Nest CLI..."
Write-Host "   $ARROW $URL"

try {
    # Download with progress
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $URL -OutFile $TEMP_FILE -UseBasicParsing -ErrorAction Stop
    Write-ColorOutput Green "   $CHECK Download completed"
    
    # Verify downloaded file
    Write-ColorOutput Cyan "$INFO Verifying download..."
    if (-not (Test-Path $TEMP_FILE)) {
        Write-ColorOutput Red "$CROSS Error: Downloaded file not found"
        exit 1
    }
    Write-ColorOutput Green "   $CHECK Archive verified"
    
    # Extract archive
    Write-ColorOutput Cyan "$INFO Extracting archive..."
    Expand-Archive -Path $TEMP_FILE -DestinationPath $TEMP_DIR -Force -ErrorAction Stop
    Write-ColorOutput Green "   $CHECK Archive extracted"
    
    # Move binary to install directory
    Write-ColorOutput Cyan "$INFO Installing binary..."
    $ExtractedBinary = Join-Path $TEMP_DIR $BINARY_NAME
    if (Test-Path $ExtractedBinary) {
        Move-Item -Path $ExtractedBinary -Destination $BINARY_PATH -Force -ErrorAction Stop
        Write-ColorOutput Green "   $CHECK Binary installed to $BINARY_PATH"
    } else {
        Write-ColorOutput Red "$CROSS Error: Binary '$BINARY_NAME' not found in archive"
        Write-Host "   Archive contents:"
        Get-ChildItem $TEMP_DIR | ForEach-Object { Write-Host "      $($_.Name)" }
        Remove-Item -Path $TEMP_DIR -Recurse -Force
        exit 1
    }
    
    # Cleanup
    Remove-Item -Path $TEMP_DIR -Recurse -Force
} catch {
    Write-ColorOutput Red "$CROSS Error downloading or extracting binary: $_"
    if (Test-Path $TEMP_DIR) {
        Remove-Item -Path $TEMP_DIR -Recurse -Force -ErrorAction SilentlyContinue
    }
    exit 1
}

# Check if install directory is in PATH
Write-Host ""
Write-ColorOutput Cyan "$INFO Configuring PATH..."
$CurrentPath = $env:Path
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
$IsInCurrentPath = $CurrentPath -like "*$INSTALL_DIR*"
$IsInUserPath = $UserPath -like "*$INSTALL_DIR*"

# Add to current session PATH
if (-not $IsInCurrentPath) {
    Write-Host "   Adding to PATH for current session..." -NoNewline
    $env:Path += ";$INSTALL_DIR"
    Write-ColorOutput Green " $CHECK"
} else {
    Write-ColorOutput Green "   $CHECK Already in current session PATH"
}

# Add to PowerShell profile
$ProfilePath = $PROFILE.CurrentUserAllHosts
$ProfileContent = ""
$PathLine = "`$env:Path += `";`$env:USERPROFILE\.local\bin`""

if (Test-Path $ProfilePath) {
    $ProfileContent = Get-Content $ProfilePath -Raw -ErrorAction SilentlyContinue
} else {
    # Create profile directory if it doesn't exist
    $ProfileDir = Split-Path -Parent $ProfilePath
    if (-not (Test-Path $ProfileDir)) {
        New-Item -ItemType Directory -Path $ProfileDir -Force | Out-Null
    }
    $ProfileContent = ""
}

# Check if PATH line already exists in profile
$PathLineEscaped = [regex]::Escape($PathLine)
$PathInProfile = $ProfileContent -match $PathLineEscaped

if (-not $PathInProfile) {
    Write-Host "   Adding to PowerShell profile..." -NoNewline
    try {
        if ($ProfileContent -and -not $ProfileContent.EndsWith("`n") -and -not $ProfileContent.EndsWith("`r`n")) {
            $ProfileContent += "`r`n"
        }
        $ProfileContent += "$PathLine`r`n"
        Set-Content -Path $ProfilePath -Value $ProfileContent -Encoding UTF8 -ErrorAction Stop
        Write-ColorOutput Green " $CHECK"
        Write-Host "      Profile: $ProfilePath"
    } catch {
        Write-ColorOutput Yellow " $WARN"
        Write-Host "      Could not write to profile: $_"
    }
} else {
    Write-ColorOutput Green "   $CHECK Already in PowerShell profile"
}

# Add to user PATH environment variable
if (-not $IsInUserPath) {
    Write-Host "   Adding to user PATH environment variable..." -NoNewline
    try {
        $NewUserPath = $UserPath
        if ($NewUserPath -and -not $NewUserPath.EndsWith(";")) {
            $NewUserPath += ";"
        }
        $NewUserPath += $INSTALL_DIR
        [Environment]::SetEnvironmentVariable("Path", $NewUserPath, "User")
        Write-ColorOutput Green " $CHECK"
    } catch {
        Write-ColorOutput Yellow " $WARN"
        Write-Host "      Could not add to user PATH: $_"
    }
} else {
    Write-ColorOutput Green "   $CHECK Already in user PATH environment variable"
}

# Success message
Write-Host ""
Write-ColorOutput Green "╔════════════════════════════════════════╗"
Write-ColorOutput Green "║  $CHECK Nest CLI installed successfully!  ║"
Write-ColorOutput Green "╚════════════════════════════════════════╝"
Write-Host ""
Write-Host "   Run " -NoNewline
Write-ColorOutput White "nest --version"
Write-Host "   to verify installation."
Write-Host ""

