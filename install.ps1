# Install script for Nest CLI tool (Windows PowerShell)

$ErrorActionPreference = "Stop"

# Colors for output
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

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

Write-ColorOutput Green "Installing Nest CLI..."
Write-Host "Platform: ${PLATFORM}-${ARCHITECTURE}"
Write-Host "Install directory: ${INSTALL_DIR}"

# Create install directory if it doesn't exist
if (-not (Test-Path $INSTALL_DIR)) {
    New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null
}

# Download binary
$TEMP_DIR = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
$TEMP_FILE = Join-Path $TEMP_DIR "nest-${PLATFORM}-${ARCHITECTURE}.zip"

if ($VERSION -eq "latest") {
    $URL = "https://github.com/${REPO}/releases/latest/download/nest-${PLATFORM}-${ARCHITECTURE}.zip"
} else {
    $URL = "https://github.com/${REPO}/releases/download/v${VERSION}/nest-${PLATFORM}-${ARCHITECTURE}.zip"
}

Write-ColorOutput Yellow "Downloading from: ${URL}"

try {
    Invoke-WebRequest -Uri $URL -OutFile $TEMP_FILE -UseBasicParsing
    
    # Extract archive
    Expand-Archive -Path $TEMP_FILE -DestinationPath $TEMP_DIR -Force
    
    # Move binary to install directory
    $ExtractedBinary = Join-Path $TEMP_DIR $BINARY_NAME
    if (Test-Path $ExtractedBinary) {
        Move-Item -Path $ExtractedBinary -Destination $BINARY_PATH -Force
    } else {
        Write-ColorOutput Red "Error: Binary not found in archive"
        exit 1
    }
    
    # Cleanup
    Remove-Item -Path $TEMP_DIR -Recurse -Force
} catch {
    Write-ColorOutput Red "Error downloading or extracting binary: $_"
    exit 1
}

# Check if install directory is in PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$INSTALL_DIR*") {
    Write-ColorOutput Yellow "Warning: ${INSTALL_DIR} is not in your PATH."
    Write-Host "Add this directory to your PATH:"
    Write-ColorOutput Green "`$env:Path += `";${INSTALL_DIR}`""
    Write-Host ""
    Write-Host "Or add it permanently via System Properties > Environment Variables"
}

Write-ColorOutput Green "✓ Nest CLI installed successfully!"
Write-Host "Run 'nest --version' to verify installation."

