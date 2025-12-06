@echo off
title GenXLink Portable v1.0.0
echo =====================================
echo    GenXLink Remote Desktop v1.0.0
echo         (Portable Edition)
echo =====================================
echo.
echo Starting servers...
echo.

echo [1/2] Starting API Server...
start "GenXLink API Server" cmd /k "title GenXLink API Server && api-server.exe"

timeout /t 2 /nobreak >nul

echo [2/2] Starting Signaling Server...
start "GenXLink Signaling Server" cmd /k "title GenXLink Signaling Server && signaling-server.exe"

timeout /t 2 /nobreak >nul

echo.
echo =====================================
echo     GenXLink Servers Started
echo =====================================
echo.
echo API Server:      http://127.0.0.1:8000
echo Signaling Server: http://127.0.0.1:8081
echo Health Check:    http://127.0.0.1:8000/health
echo.
echo Press any key to open servers in browser...
pause >nul

start http://127.0.0.1:8000/health
start http://127.0.0.1:8081/health

echo.
echo GenXLink Portable is running!
echo Ready for remote desktop connections
echo.
pause
