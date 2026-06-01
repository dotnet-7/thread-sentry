@echo off
setlocal enabledelayedexpansion

echo ========================================
echo Thread-Sentry Performance Benchmark
echo ========================================
echo.

REM Create directories
if not exist "results\data" mkdir results\data
if not exist "results\charts" mkdir results\charts

REM Build test program
echo Building performance test program...
cargo build --release --example performance_test
if errorlevel 1 (
    echo Build failed!
    pause
    exit /b 1
)
echo Build completed successfully.
echo.

REM Run latency tests
echo Running latency tests...
echo This will take about 30 seconds...
target\release\examples\performance_test.exe latency > results\data\latency.csv
echo Latency tests completed.
echo.

REM Run throughput tests
echo Running throughput tests...
echo This will take about 60 seconds...
target\release\examples\performance_test.exe throughput > results\data\throughput.csv
echo Throughput tests completed.
echo.

REM Generate charts
echo Generating performance charts...
python scripts\generate_charts.py
if errorlevel 1 (
    echo Chart generation failed!
    echo Please ensure Python and matplotlib are installed.
    pause
    exit /b 1
)
echo Charts generated successfully.
echo.

REM Show results location
echo ========================================
echo Performance tests completed!
echo ========================================
echo.
echo Results:
echo   Data files: results\data\
echo   Charts:     results\charts\
echo.
echo Please check PERFORMANCE_REPORT.md for analysis.
echo.

pause