# Check nginx availability and setup for Golem Router
Write-Host "🔍 Checking nginx setup for Golem Router..." -ForegroundColor Cyan

# Check if nginx is available
$nginxFound = $false
$nginxPath = ""
$possiblePaths = @(
    "nginx",
    "C:\nginx\nginx.exe", 
    "C:\Program Files\nginx\nginx.exe",
    "$env:ProgramFiles\nginx\nginx.exe"
)

Write-Host "`n📋 Checking nginx installation..."
foreach ($path in $possiblePaths) {
    try {
        $cmd = Get-Command $path -ErrorAction Stop
        $nginxPath = $cmd.Source
        $nginxFound = $true
        Write-Host "✅ Found nginx at: $nginxPath" -ForegroundColor Green
        
        # Get nginx version
        $version = & $path -v 2>&1
        Write-Host "   Version: $version" -ForegroundColor Gray
        break
    }
    catch {
        Write-Host "❌ Not found: $path" -ForegroundColor Red
    }
}

if (!$nginxFound) {
    Write-Host "`n⚠️  nginx NOT FOUND!" -ForegroundColor Red
    Write-Host "To install nginx on Windows:" -ForegroundColor Yellow
    Write-Host "1. Download from: https://nginx.org/en/download.html" -ForegroundColor White
    Write-Host "2. Or install via Chocolatey: choco install nginx" -ForegroundColor White
    Write-Host "3. Or install via Scoop: scoop install nginx" -ForegroundColor White
}

# Check router config template
Write-Host "`n📄 Checking router configuration..."
$templatePath = "$PWD\golem-router\golem-services.conf.template"
if (Test-Path $templatePath) {
    Write-Host "✅ Router config template found: $templatePath" -ForegroundColor Green
} else {
    Write-Host "❌ Router config template NOT found: $templatePath" -ForegroundColor Red
}

# Check if services are running
Write-Host "`n🔍 Checking if Golem services are running..."
$servicePorts = @(
    @{Name="Component Service"; Port=8082},
    @{Name="Worker Service"; Port=8085}, 
    @{Name="Cloud Service"; Port=8080}
)

foreach ($service in $servicePorts) {
    try {
        $connection = Test-NetConnection -ComputerName localhost -Port $service.Port -WarningAction SilentlyContinue
        if ($connection.TcpTestSucceeded) {
            Write-Host "✅ $($service.Name) is running on port $($service.Port)" -ForegroundColor Green
        } else {
            Write-Host "❌ $($service.Name) is NOT running on port $($service.Port)" -ForegroundColor Red
        }
    }
    catch {
        Write-Host "❌ $($service.Name) is NOT running on port $($service.Port)" -ForegroundColor Red
    }
}

# Check if router is running
Write-Host "`n🔀 Checking router status..."
$routerPorts = @(80, 8080)
$routerRunning = $false

foreach ($port in $routerPorts) {
    try {
        $connection = Test-NetConnection -ComputerName localhost -Port $port -WarningAction SilentlyContinue
        if ($connection.TcpTestSucceeded) {
            Write-Host "✅ Router/Service is running on port $port" -ForegroundColor Green
            $routerRunning = $true
        }
    }
    catch {
        # Port not in use
    }
}

if (!$routerRunning) {
    Write-Host "❌ No router detected on standard ports (80, 8080)" -ForegroundColor Red
}

# Summary and recommendations
Write-Host "`n📋 SUMMARY:" -ForegroundColor Cyan
Write-Host "----------------------------------------"

if ($nginxFound -and (Test-Path $templatePath)) {
    Write-Host "✅ Router setup is READY" -ForegroundColor Green
    Write-Host "   Run: .\local-run\start-router.ps1 (standalone router)"
    Write-Host "   Or:  .\local-run\start-with-router.ps1 (all services + router)"
} else {
    Write-Host "❌ Router setup is INCOMPLETE" -ForegroundColor Red
    if (!$nginxFound) {
        Write-Host "   → Install nginx for Windows"
    }
    if (!(Test-Path $templatePath)) {
        Write-Host "   → Ensure golem-router directory is present"
    }
}

Write-Host "----------------------------------------"
