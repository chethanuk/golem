# Simple Golem Services + Router Start Script for Windows
param(
    [int]$RouterPort = 80
)

Write-Host "Starting Golem services with Router..." -ForegroundColor Green

# Clean up any existing data
if (Test-Path ".\local-run\data\shard-manager") { 
    Remove-Item -Recurse -Force ".\local-run\data\shard-manager" 
}

# Create directories
$null = New-Item -ItemType Directory -Path ".\local-run\data\redis" -Force
$null = New-Item -ItemType Directory -Path ".\local-run\data\shard-manager" -Force  
$null = New-Item -ItemType Directory -Path ".\local-run\logs" -Force
$null = New-Item -ItemType Directory -Path ".\local-run\nginx" -Force

# Environment variables
$env:RUST_BACKTRACE = "1"
$env:GOLEM__TRACING__FILE_DIR = "$PWD\local-run\logs"
$env:GOLEM__TRACING__FILE__ANSI = "true"
$env:GOLEM__TRACING__FILE__ENABLED = "true"
$env:GOLEM__TRACING__FILE__JSON = "false"
$env:GOLEM__TRACING__STDOUT__ENABLED = "false"

$ADMIN_TOKEN = "5c832d93-ff85-4a8f-9803-513950fdfdb1"
$FS_BLOB_STORAGE_DIR = "$PWD\local-run\data\blob_storage"

# Ports
$COMPONENT_SERVICE_HTTP_PORT = 8082
$WORKER_SERVICE_HTTP_PORT = 8085
$CLOUD_SERVICE_HTTP_PORT = 8080

$processes = @()

try {
    # Check for nginx
    $nginxPath = $null
    $nginxPaths = @("nginx", "C:\nginx\nginx.exe")
    foreach ($path in $nginxPaths) {
        try {
            $null = Get-Command $path -ErrorAction Stop
            $nginxPath = $path
            break
        }
        catch {
            # Continue
        }
    }

    # Start Redis
    Write-Host "Starting Redis..."
    $redis = Start-Process -FilePath "redis-server" -ArgumentList "--port 6380" -WindowStyle Hidden -PassThru
    $processes += $redis
    Start-Sleep 2

    # Start Component Service
    Write-Host "Starting Component Service..."
    Push-Location "golem-component-service"
    $env:GOLEM__HTTP_PORT = $COMPONENT_SERVICE_HTTP_PORT
    $env:GOLEM__GRPC_PORT = 9092
    $env:GOLEM__BLOB_STORAGE__TYPE = "LocalFileSystem"
    $env:GOLEM__BLOB_STORAGE__CONFIG__ROOT = $FS_BLOB_STORAGE_DIR
    $env:GOLEM__DB__TYPE = "Sqlite"
    $env:GOLEM__DB__CONFIG__DATABASE = "..\local-run\data\golem_component.db"
    $env:GOLEM__CORS_ORIGIN_REGEX = "http://localhost:3000"
    
    $comp = Start-Process -FilePath "..\target\debug\golem-component-service.exe" -WindowStyle Hidden -PassThru
    $processes += $comp
    Pop-Location
    Start-Sleep 3

    # Start Worker Service  
    Write-Host "Starting Worker Service..."
    Push-Location "golem-worker-service"
    $env:GOLEM__PORT = $WORKER_SERVICE_HTTP_PORT
    $env:GOLEM__WORKER_GRPC_PORT = 9095
    $env:GOLEM__BLOB_STORAGE__TYPE = "LocalFileSystem"
    $env:GOLEM__BLOB_STORAGE__CONFIG__ROOT = $FS_BLOB_STORAGE_DIR
    $env:GOLEM__DB__TYPE = "Sqlite"
    $env:GOLEM__DB__CONFIG__DATABASE = "..\local-run\data\golem_worker.sqlite"
    $env:GOLEM__CORS_ORIGIN_REGEX = "http://localhost:3000"
    
    $worker = Start-Process -FilePath "..\target\debug\golem-worker-service.exe" -WindowStyle Hidden -PassThru
    $processes += $worker
    Pop-Location
    Start-Sleep 3

    # Start Router if nginx available
    $nginx = $null
    if ($nginxPath) {
        Write-Host "Starting Router (nginx)..." -ForegroundColor Yellow
        
        $nginxDir = "$PWD\local-run\nginx"
        $templatePath = "$PWD\golem-router\golem-services.conf.template"
        $configPath = "$nginxDir\nginx.conf"
        
        if (Test-Path $templatePath) {
            # Read template and replace variables
            $template = Get-Content $templatePath -Raw
            $config = $template.Replace('$GOLEM_COMPONENT_SERVICE_HOST', 'localhost')
            $config = $config.Replace('$GOLEM_COMPONENT_SERVICE_PORT', $COMPONENT_SERVICE_HTTP_PORT)
            $config = $config.Replace('$GOLEM_WORKER_SERVICE_HOST', 'localhost')
            $config = $config.Replace('$GOLEM_WORKER_SERVICE_PORT', $WORKER_SERVICE_HTTP_PORT)
            $config = $config.Replace('$GOLEM_COMPONENT_MAX_SIZE_ALLOWED', '50M')
            $config = $config.Replace('resolver 127.0.0.11;', 'resolver 8.8.8.8;')
            
            if ($RouterPort -ne 80) {
                $config = $config.Replace('listen 80;', "listen $RouterPort;")
            }
            
            Set-Content -Path $configPath -Value $config -Encoding UTF8
            $null = New-Item -ItemType Directory -Path "$nginxDir\logs" -Force
            $null = New-Item -ItemType Directory -Path "$nginxDir\temp" -Force
            
            $nginx = Start-Process -FilePath $nginxPath -ArgumentList "-c $configPath -p $nginxDir" -WindowStyle Hidden -PassThru
            $processes += $nginx
            Start-Sleep 2
            
            if (!$nginx.HasExited) {
                Write-Host "Router started on port $RouterPort" -ForegroundColor Green
            }
        }
    }

    Write-Host ""
    Write-Host "Services Status:" -ForegroundColor Cyan
    Write-Host "Redis: $($redis.Id)"
    Write-Host "Component Service: $($comp.Id)"  
    Write-Host "Worker Service: $($worker.Id)"
    if ($nginx -and !$nginx.HasExited) {
        Write-Host "Router: $($nginx.Id)" -ForegroundColor Green
        Write-Host ""
        Write-Host "Router URL: http://localhost:$RouterPort" -ForegroundColor Green
        Write-Host "API: http://localhost:$RouterPort/v1/components" -ForegroundColor White
    }
    Write-Host ""
    Write-Host "Direct URLs:"
    Write-Host "Component Service: http://localhost:$COMPONENT_SERVICE_HTTP_PORT"
    Write-Host "Worker Service: http://localhost:$WORKER_SERVICE_HTTP_PORT"
    Write-Host ""
    Write-Host "Press Ctrl+C to stop all services" -ForegroundColor Yellow
    
    while ($true) {
        Start-Sleep 1
    }
}
catch {
    Write-Host "Error: $_" -ForegroundColor Red
}
finally {
    Write-Host "Stopping services..."
    foreach ($proc in $processes) {
        if ($proc -and !$proc.HasExited) {
            try {
                Stop-Process -Id $proc.Id -Force
                Write-Host "Stopped $($proc.Id)"
            }
            catch {
                Write-Host "Failed to stop $($proc.Id)"
            }
        }
    }
    Write-Host "All services stopped."
}
