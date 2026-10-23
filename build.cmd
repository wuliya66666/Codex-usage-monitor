@echo off
setlocal
cd /d "%~dp0"
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
cargo test
if errorlevel 1 goto failed
cargo build --release
if errorlevel 1 goto failed
echo.
echo Build succeeded: target\release\codex-usage.exe
start "" explorer.exe "%~dp0target\release"
pause
exit /b 0
:failed
echo.
echo Build failed. Please send a screenshot of the error above.
pause
exit /b 1
