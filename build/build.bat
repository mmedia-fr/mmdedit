@echo off
REM Construction de l'executable Windows MMdedit (PyInstaller, onefile).
REM PyInstaller ne fait pas de compilation croisee : a lancer sous Windows.
REM Sortie : dist\MMdedit.exe (a la racine du projet, pas dans build\).

setlocal
cd /d "%~dp0.."

echo [1/3] Installation des dependances...
python -m pip install --disable-pip-version-check -q -r requirements.txt pyinstaller
if errorlevel 1 goto :erreur

echo [2/3] Nettoyage des sorties precedentes...
if exist dist\MMdedit.exe del /q dist\MMdedit.exe
if exist build\MMdedit rmdir /s /q build\MMdedit

REM Catalogues de traduction Qt : sans eux, les boutons standard des boites de
REM dialogue restent en anglais (« Show Details... ») dans l'executable.
for /f "delims=" %%T in ('python -c "import PySide6,os;print(os.path.join(os.path.dirname(PySide6.__file__),'translations'))"') do set "QT_TR=%%T"

echo [3/3] Construction...
python -m PyInstaller ^
  --noconfirm ^
  --onefile ^
  --windowed ^
  --name MMdedit ^
  --paths src ^
  --icon "%CD%\src\mmdedit\assets\mmdedit.ico" ^
  --add-data "%CD%\src\mmdedit\assets;mmdedit/assets" ^
  --add-data "%CD%\LICENSE;mmdedit" ^
  --add-data "%QT_TR%\qtbase_fr.qm;mmdedit/translations" ^
  --add-data "%QT_TR%\qt_fr.qm;mmdedit/translations" ^
  --workpath build\MMdedit ^
  --specpath build ^
  run_mmdedit.py
if errorlevel 1 goto :erreur

echo.
echo Termine : %CD%\dist\MMdedit.exe

REM Argument « installeur » : enchaine sur la construction de l'installeur.
if /i "%~1"=="installeur" call "%~dp0build_installeur.bat" || goto :erreur

endlocal
exit /b 0

:erreur
echo.
echo ECHEC de la construction.
endlocal
exit /b 1
