@echo off
setlocal
set "GODOT=%LOCALAPPDATA%\Programs\Godot\4.7.2-stable-win64\Godot_v4.7.2-stable_win64.exe"
if not exist "%GODOT%" (
    where godot.exe >nul 2>&1
    if errorlevel 1 (
        echo Godot 4 is required to open this project.
        pause
        exit /b 1
    )
    set "GODOT=godot.exe"
)
if /i "%~1"=="--play" (
    start "" "%GODOT%" --path "%~dp0."
) else (
    start "" "%GODOT%" --editor --path "%~dp0." res://water_lab.tscn
)
