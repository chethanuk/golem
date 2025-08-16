# Golem Router (nginx) Start Script for Windows
# This script starts nginx with the Golem services configuration

param(
    [int]$RouterPort = 80,
    [string]$ComponentServiceHost = "localhost",
    [int]$ComponentServicePort = 8082,
    [string]$WorkerServiceHost = "localhost", 
    [int]$WorkerServicePort = 8085,
    [string]$ComponentMaxSize = "50M"
)

Write-Host "Starting Golem Router (nginx) on Windows..."

# Check if nginx is available
$nginxPath = ""
$possiblePaths = @(
    "nginx",
    "C:\nginx\nginx.exe",
    "C:\Program Files\nginx\nginx.exe",
    "$env:ProgramFiles\nginx\nginx.exe"
)

foreach ($path in $possiblePaths) {
    try {
        $null = Get-Command $path -ErrorAction Stop
        $nginxPath = $path
        Write-Host "Found nginx at: $nginxPath"
        break
    }
    catch {
        # Continue searching
    }
}

if ([string]::IsNullOrEmpty($nginxPath)) {
    Write-Host "ERROR: nginx not found! Please install nginx for Windows." -ForegroundColor Red
    Write-Host "Download from: https://nginx.org/en/download.html" -ForegroundColor Yellow
    Write-Host "Or install via Chocolatey: choco install nginx" -ForegroundColor Yellow
    exit 1
}

# Create nginx working directory
$nginxWorkDir = "$PWD\local-run\nginx"
New-Item -ItemType Directory -Path $nginxWorkDir -Force | Out-Null

# Set environment variables for nginx config template
$env:GOLEM_COMPONENT_SERVICE_HOST = $ComponentServiceHost
$env:GOLEM_COMPONENT_SERVICE_PORT = $ComponentServicePort
$env:GOLEM_WORKER_SERVICE_HOST = $WorkerServiceHost
$env:GOLEM_WORKER_SERVICE_PORT = $WorkerServicePort
$env:GOLEM_COMPONENT_MAX_SIZE_ALLOWED = $ComponentMaxSize

# Generate nginx.conf from template
$templatePath = "$PWD\golem-router\golem-services.conf.template"
$configPath = "$nginxWorkDir\nginx.conf"

if (!(Test-Path $templatePath)) {
    Write-Host "ERROR: nginx config template not found at: $templatePath" -ForegroundColor Red
    exit 1
}

Write-Host "Generating nginx configuration..."

# Read template and substitute environment variables
$template = Get-Content $templatePath -Raw

# Replace environment variables in the template
$config = $template
$config = $config -replace '\$GOLEM_COMPONENT_SERVICE_HOST', $env:GOLEM_COMPONENT_SERVICE_HOST
$config = $config -replace '\$GOLEM_COMPONENT_SERVICE_PORT', $env:GOLEM_COMPONENT_SERVICE_PORT
$config = $config -replace '\$GOLEM_WORKER_SERVICE_HOST', $env:GOLEM_WORKER_SERVICE_HOST
$config = $config -replace '\$GOLEM_WORKER_SERVICE_PORT', $env:GOLEM_WORKER_SERVICE_PORT
$config = $config -replace '\$GOLEM_COMPONENT_MAX_SIZE_ALLOWED', $env:GOLEM_COMPONENT_MAX_SIZE_ALLOWED

# Update resolver for Windows (use local DNS)
$config = $config -replace 'resolver 127\.0\.0\.11;', 'resolver 8.8.8.8;'

# Update listen port if not 80
if ($RouterPort -ne 80) {
    $config = $config -replace 'listen 80;', "listen $RouterPort;"
}

# Write the generated config
Set-Content -Path $configPath -Value $config -Encoding UTF8

Write-Host "Generated nginx configuration at: $configPath"

# Create nginx directory structure
New-Item -ItemType Directory -Path "$nginxWorkDir\logs", "$nginxWorkDir\temp" -Force | Out-Null

# Start nginx
Write-Host "Starting nginx on port $RouterPort..."
Write-Host "Routing to Component Service: ${ComponentServiceHost}:${ComponentServicePort}"
Write-Host "Routing to Worker Service: ${WorkerServiceHost}:${WorkerServicePort}"

try {
    # Start nginx with the generated config
    $nginxProcess = Start-Process -FilePath $nginxPath -ArgumentList "-c", $configPath, "-p", $nginxWorkDir -PassThru -NoNewWindow
    
    # Wait a moment for nginx to start
    Start-Sleep -Seconds 2
    
    if ($nginxProcess.HasExited) {
        Write-Host "ERROR: nginx failed to start!" -ForegroundColor Red
        Write-Host "Check nginx error logs in: $nginxWorkDir\logs\error.log" -ForegroundColor Yellow
        exit 1
    }
    
    Write-Host "✅ Golem Router started successfully!" -ForegroundColor Green
    Write-Host "Router URL: http://localhost:$RouterPort" -ForegroundColor Cyan
    Write-Host "Process ID: $($nginxProcess.Id)" -ForegroundColor Gray
    Write-Host ""
    Write-Host "API endpoints available through router:"
    Write-Host "  - Components: http://localhost:$RouterPort/v1/components"
    Write-Host "  - Workers: http://localhost:$RouterPort/v1/components/{id}/workers"
    Write-Host "  - Invoke: http://localhost:$RouterPort/v1/components/{id}/invoke"
    Write-Host ""
    Write-Host "Press Ctrl+C to stop the router..."
    
    # Wait for user interrupt
    while (!$nginxProcess.HasExited) {
        Start-Sleep -Seconds 1
    }
}
catch {
    Write-Host "Error starting nginx: $_" -ForegroundColor Red
}
finally {
    # Cleanup
    if ($nginxProcess -and !$nginxProcess.HasExited) {
        Write-Host "Stopping nginx..."
        Stop-Process -Id $nginxProcess.Id -Force
    }
    Write-Host "Router stopped."
}
