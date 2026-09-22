@echo off
setlocal

set "VSDEVCMD=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat"
if not exist "%VSDEVCMD%" (
  echo ERROR: VsDevCmd not found at "%VSDEVCMD%"
  exit /b 1
)

call "%VSDEVCMD%" -no_logo
if errorlevel 1 (
  echo ERROR: Failed to initialize Visual Studio build environment.
  exit /b 1
)

set "PATH=%USERPROFILE%\.cargo\bin;C:\Program Files\qemu;%PATH%"

cd /d "%~dp0"

rustup override set nightly
if errorlevel 1 exit /b 1

rustup component add rust-src llvm-tools-preview
if errorlevel 1 exit /b 1

cargo install bootimage
if errorlevel 1 exit /b 1

cargo +nightly bootimage --target x86_64-ferrumos.json -Zbuild-std=core,alloc,compiler_builtins -Zbuild-std-features=compiler-builtins-mem -Zjson-target-spec
if errorlevel 1 exit /b 1

echo.
echo FerrumOS image built successfully. Starting QEMU...

set "IMAGE_PATH=target\x86_64-ferrumos\debug\bootimage-ferrumos.bin"
if not exist "%IMAGE_PATH%" (
  echo ERROR: Boot image not found at "%IMAGE_PATH%"
  exit /b 1
)

qemu-system-x86_64 -no-reboot -no-shutdown -drive format=raw,file="%IMAGE_PATH%"
