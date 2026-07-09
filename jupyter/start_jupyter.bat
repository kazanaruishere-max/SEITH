@echo off
title AI SEITH Jupyter Lab
cd /d "%~dp0.."
echo [AI SEITH] Starting Jupyter Lab...
echo [AI SEITH] Open http://localhost:8888 in browser
echo.
call jupyter lab --no-browser --port=8888 --notebook-dir="%CD%\jupyter"
pause
