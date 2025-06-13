@echo off
echo 🚀 Starting Lab Manager Lightweight Version...
echo.

echo 📋 Checking Docker status...
docker ps >nul 2>&1
if errorlevel 1 (
    echo ❌ Docker is not running. Please start Docker Desktop first.
    pause
    exit /b 1
)

echo ✅ Docker is running!
echo.

echo 🛑 Stopping any existing containers...
docker-compose down 2>nul

echo 🏗️  Building lightweight containers...
docker-compose -f docker-compose.lightweight.yml build

echo 🚀 Starting lightweight services...
docker-compose -f docker-compose.lightweight.yml up -d

echo.
echo ✅ Lab Manager Lightweight Version Starting!
echo.
echo 🌐 Frontend: http://localhost:8080
echo 🔧 Backend:  http://localhost:3001
echo 🗄️  Database: localhost:5433
echo.
echo 📊 Checking status...
docker-compose -f docker-compose.lightweight.yml ps

echo.
echo 🎉 Ready! Open http://localhost:8080 in your browser
pause 
