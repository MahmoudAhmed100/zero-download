@echo off
rem Zero Download - One-click Windows Build
rem Double-click this file to build ZeroDownload.exe
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build-windows.ps1"
