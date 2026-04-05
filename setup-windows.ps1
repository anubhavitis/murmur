# Murmur Windows Build Setup
# Run in PowerShell as Administrator:
#   Set-ExecutionPolicy Bypass -Scope Process; .\setup-windows.ps1

$ErrorActionPreference = "Stop"

Write-Host "`n=== Murmur Windows Build Setup ===" -ForegroundColor Cyan

# 1. Rust toolchain
if (Get-Command rustup -ErrorAction SilentlyContinue) {
    Write-Host "[OK] Rust already installed" -ForegroundColor Green
    rustup update stable
} else {
    Write-Host "[INSTALLING] Rust toolchain..." -ForegroundColor Yellow
    winget install Rustlang.Rustup --accept-source-agreements --accept-package-agreements
    # Refresh PATH for current session
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
}

# 2. Visual Studio Build Tools (MSVC C/C++ compiler + Windows SDK)
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsInstall = & $vsWhere -latest -property installationPath 2>$null
    if ($vsInstall) {
        Write-Host "[OK] Visual Studio Build Tools found at $vsInstall" -ForegroundColor Green
    } else {
        Write-Host "[INSTALLING] Visual Studio Build Tools..." -ForegroundColor Yellow
        winget install Microsoft.VisualStudio.2022.BuildTools --accept-source-agreements --accept-package-agreements --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
    }
} else {
    Write-Host "[INSTALLING] Visual Studio Build Tools..." -ForegroundColor Yellow
    winget install Microsoft.VisualStudio.2022.BuildTools --accept-source-agreements --accept-package-agreements --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
}

# 3. CMake (required by whisper-rs-sys to compile whisper.cpp)
if (Get-Command cmake -ErrorAction SilentlyContinue) {
    Write-Host "[OK] CMake already installed" -ForegroundColor Green
} else {
    Write-Host "[INSTALLING] CMake..." -ForegroundColor Yellow
    winget install Kitware.CMake --accept-source-agreements --accept-package-agreements
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
}

# 4. LLVM/Clang (required by bindgen in whisper-rs-sys)
if (Get-Command clang -ErrorAction SilentlyContinue) {
    Write-Host "[OK] LLVM/Clang already installed" -ForegroundColor Green
} else {
    Write-Host "[INSTALLING] LLVM..." -ForegroundColor Yellow
    winget install LLVM.LLVM --accept-source-agreements --accept-package-agreements
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
}

# 5. Git
if (Get-Command git -ErrorAction SilentlyContinue) {
    Write-Host "[OK] Git already installed" -ForegroundColor Green
} else {
    Write-Host "[INSTALLING] Git..." -ForegroundColor Yellow
    winget install Git.Git --accept-source-agreements --accept-package-agreements
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
}

# Set LIBCLANG_PATH if LLVM is installed but env var is missing
if (-not $env:LIBCLANG_PATH) {
    $llvmPaths = @(
        "${env:ProgramFiles}\LLVM\bin",
        "${env:ProgramFiles(x86)}\LLVM\bin"
    )
    foreach ($p in $llvmPaths) {
        if (Test-Path "$p\libclang.dll") {
            [System.Environment]::SetEnvironmentVariable("LIBCLANG_PATH", $p, "User")
            $env:LIBCLANG_PATH = $p
            Write-Host "[SET] LIBCLANG_PATH=$p" -ForegroundColor Yellow
            break
        }
    }
}

Write-Host "`n=== Setup Complete ===" -ForegroundColor Cyan
Write-Host "`nNext steps:" -ForegroundColor White
Write-Host "  1. Close and reopen PowerShell (to pick up PATH changes)" -ForegroundColor White
Write-Host "  2. Clone and build:" -ForegroundColor White
Write-Host "     git clone https://github.com/anubhavitis/murmur.git" -ForegroundColor Gray
Write-Host "     cd murmur" -ForegroundColor Gray
Write-Host "     cargo build --release" -ForegroundColor Gray
Write-Host "  3. Run:" -ForegroundColor White
Write-Host "     .\target\release\murmur.exe" -ForegroundColor Gray
Write-Host ""
