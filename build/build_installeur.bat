@echo off
REM Construction de l'installeur Windows MMdedit (Inno Setup 6).
REM Prerequis : dist\MMdedit.exe (lancer build\build.bat d'abord)
REM             et Inno Setup 6 (choco install innosetup).
REM Sortie : dist\MMdedit-<version>-setup.exe

setlocal
cd /d "%~dp0.."

if not exist "dist\MMdedit.exe" (
  echo ECHEC : dist\MMdedit.exe absent. Lancez d'abord build\build.bat
  endlocal
  exit /b 1
)

set "ISCC="
for %%P in (
  "%LOCALAPPDATA%\Programs\InnoSetup6\ISCC.exe"
  "%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe"
  "%ProgramFiles%\Inno Setup 6\ISCC.exe"
) do if exist %%P set "ISCC=%%~P"

if not defined ISCC (
  where ISCC.exe >nul 2>&1 && for /f "delims=" %%I in ('where ISCC.exe') do set "ISCC=%%I"
)

if not defined ISCC (
  echo ECHEC : Inno Setup 6 introuvable. Installez-le : choco install innosetup -y
  endlocal
  exit /b 1
)

echo Compilation de l'installeur avec "%ISCC%"...
"%ISCC%" /Q "build\mmdedit.iss"
if errorlevel 1 (
  echo.
  echo ECHEC de la compilation de l'installeur.
  endlocal
  exit /b 1
)

echo.
echo Termine. Installeur genere dans %CD%\dist\ :
dir /b "dist\MMdedit-*-setup.exe"
endlocal
exit /b 0
