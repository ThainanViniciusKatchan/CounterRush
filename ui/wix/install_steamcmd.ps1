param(
    [string]$DownloadUrl = "https://client-update.steamstatic.com/installer/steamcmd.zip",
    [string]$InstallDir = "C:\steamcmd",
    [int]$VerifyTimeoutSeconds = 180,
    [switch]$Force
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$script:LogPath = [System.IO.Path]::Combine(
    [System.IO.Path]::GetTempPath(),
    "CounterRush_SteamCMD_Install.log"
)

function Write-InstallLog {
    param([string]$Message)

    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $line = "[$timestamp] $Message"
    Write-Output $line
    try {
        Add-Content -LiteralPath $script:LogPath -Value $line -Encoding UTF8 -ErrorAction SilentlyContinue
    } catch {
    }
}

function Enable-ModernTls {
    foreach ($protocolName in @("Tls12", "Tls13")) {
        try {
            $protocolValue = [System.Net.SecurityProtocolType]::$protocolName
            [System.Net.ServicePointManager]::SecurityProtocol = `
                [System.Net.ServicePointManager]::SecurityProtocol -bor $protocolValue
        } catch {
        }
    }
}

function Ensure-SteamCmdDirectory {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Container)) {
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
    }
    if (-not (Test-Path -LiteralPath $Path -PathType Container)) {
        throw "Nao foi possivel criar a pasta do SteamCMD em '$Path'."
    }
}

function Test-ValidZipFile {
    param([string]$ZipPath)

    if (-not (Test-Path -LiteralPath $ZipPath -PathType Leaf)) {
        return $false
    }
    $info = Get-Item -LiteralPath $ZipPath
    if ($info.Length -lt 4) {
        return $false
    }

    $stream = [System.IO.File]::OpenRead($ZipPath)
    try {
        $header = New-Object byte[] 2
        $read = $stream.Read($header, 0, 2)
        return ($read -eq 2 -and $header[0] -eq 0x50 -and $header[1] -eq 0x4B)
    } finally {
        $stream.Dispose()
    }
}

function Download-SteamCmdZip {
    param(
        [string]$Url,
        [string]$DestinationZip
    )

    Enable-ModernTls
    $maxAttempts = 3
    for ($attempt = 1; $attempt -le $maxAttempts; $attempt++) {
        try {
            if (Test-Path -LiteralPath $DestinationZip) {
                Remove-Item -LiteralPath $DestinationZip -Force -ErrorAction SilentlyContinue
            }
            Write-InstallLog "Baixando SteamCMD de '$Url' (tentativa $attempt/$maxAttempts)..."
            Invoke-WebRequest -Uri $Url -OutFile $DestinationZip -UseBasicParsing -TimeoutSec 60

            if (-not (Test-ValidZipFile -ZipPath $DestinationZip)) {
                throw "O arquivo baixado nao e um pacote ZIP valido."
            }

            $size = (Get-Item -LiteralPath $DestinationZip).Length
            Write-InstallLog "Download concluido ($size bytes)."
            return
        } catch {
            Write-InstallLog "Falha no download (tentativa $attempt): $($_.Exception.Message)"
            if ($attempt -ge $maxAttempts) {
                throw
            }
            Start-Sleep -Seconds 2
        }
    }
}

function Expand-SteamCmdZip {
    param(
        [string]$ZipPath,
        [string]$DestinationDir
    )

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $rootFull = [System.IO.Path]::GetFullPath($DestinationDir)
    $rootWithSep = if ($rootFull.EndsWith([System.IO.Path]::DirectorySeparatorChar.ToString())) {
        $rootFull
    } else {
        $rootFull + [System.IO.Path]::DirectorySeparatorChar
    }

    $archive = [System.IO.Compression.ZipFile]::OpenRead($ZipPath)
    try {
        if ($archive.Entries.Count -eq 0) {
            throw "O pacote '$ZipPath' esta vazio."
        }

        $extractedFiles = 0
        foreach ($entry in $archive.Entries) {
            $relativePath = $entry.FullName -replace '/', '\'
            $targetPath = [System.IO.Path]::GetFullPath(
                [System.IO.Path]::Combine($rootFull, $relativePath)
            )
            if (-not $targetPath.StartsWith($rootWithSep, [System.StringComparison]::OrdinalIgnoreCase)) {
                throw "Entrada invalida no pacote ZIP: '$($entry.FullName)'."
            }

            if ([string]::IsNullOrEmpty($entry.Name)) {
                Ensure-SteamCmdDirectory -Path $targetPath
                continue
            }

            $parentDir = [System.IO.Path]::GetDirectoryName($targetPath)
            if ($parentDir) {
                Ensure-SteamCmdDirectory -Path $parentDir
            }
            [System.IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $targetPath, $true)
            $extractedFiles++
        }

        if ($extractedFiles -eq 0) {
            throw "Nenhum arquivo foi extraido de '$ZipPath'."
        }
        Write-InstallLog "Extracao concluida em '$DestinationDir' ($extractedFiles arquivo(s))."
    } finally {
        $archive.Dispose()
    }
}

function Invoke-SteamCmdQuitOnce {
    param(
        [string]$ExePath,
        [string]$WorkingDir,
        [int]$TimeoutSeconds
    )

    $id = [Guid]::NewGuid().ToString("N")
    $stdoutFile = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), "steamcmd_stdout_$id.log")
    $stderrFile = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), "steamcmd_stderr_$id.log")

    try {
        $proc = Start-Process -FilePath $ExePath `
            -ArgumentList "+quit" `
            -WorkingDirectory $WorkingDir `
            -WindowStyle Hidden `
            -RedirectStandardOutput $stdoutFile `
            -RedirectStandardError $stderrFile `
            -PassThru
        $null = $proc.Handle

        if (-not $proc.WaitForExit($TimeoutSeconds * 1000)) {
            try { $proc.Kill() } catch {}
            throw "Tempo limite excedido (${TimeoutSeconds}s) ao verificar '$ExePath'."
        }
        $proc.WaitForExit()

        $stdoutText = if (Test-Path -LiteralPath $stdoutFile) {
            Get-Content -LiteralPath $stdoutFile -Raw -ErrorAction SilentlyContinue
        } else {
            ""
        }

        return @{
            ExitCode = [int]$proc.ExitCode
            Stdout   = $stdoutText
        }
    } finally {
        Remove-Item -LiteralPath $stdoutFile -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $stderrFile -Force -ErrorAction SilentlyContinue
    }
}

function Test-SteamCmdWorking {
    param(
        [string]$InstallDir,
        [int]$TimeoutSeconds,
        [bool]$AllowBootstrapRetry = $true
    )

    $exePath = Join-Path $InstallDir "steamcmd.exe"
    if (-not (Test-Path -LiteralPath $exePath -PathType Leaf)) {
        return $false
    }
    if ((Get-Item -LiteralPath $exePath).Length -eq 0) {
        return $false
    }

    $maxRuns = if ($AllowBootstrapRetry) { 3 } else { 1 }
    for ($run = 1; $run -le $maxRuns; $run++) {
        Write-InstallLog "Verificando funcionamento do SteamCMD ('$exePath +quit', execucao $run/$maxRuns)..."
        $result = Invoke-SteamCmdQuitOnce -ExePath $exePath -WorkingDir $InstallDir -TimeoutSeconds $TimeoutSeconds
        $exitCode = $result.ExitCode
        $stdout = if ($null -eq $result.Stdout) { "" } else { $result.Stdout }

        if ($exitCode -eq 0) {
            if ($stdout -match 'Steam Console Client|Loading Steam API|type ''quit'' to exit') {
                Write-InstallLog "SteamCMD verificado com sucesso (codigo de saida 0)."
                return $true
            }
            Write-InstallLog "Aviso: SteamCMD retornou 0, porem saida esperada nao foi detectada."
            return $false
        }

        # No Windows, a primeira execucao do stub baixa os binarios e encerra com codigo 7.
        if ($AllowBootstrapRetry -and $exitCode -eq 7 -and $run -lt $maxRuns) {
            Write-InstallLog "Auto-atualizacao inicial do SteamCMD concluida (codigo 7). Revalidando..."
            Start-Sleep -Seconds 2
            continue
        }

        Write-InstallLog "Verificacao do SteamCMD retornou codigo $exitCode."
        return $false
    }

    return $false
}

function Install-AndVerifySteamCmd {
    Write-InstallLog "Iniciando configuracao do SteamCMD em '$InstallDir'..."
    Ensure-SteamCmdDirectory -Path $InstallDir

    $exePath = Join-Path $InstallDir "steamcmd.exe"
    if (-not $Force -and (Test-Path -LiteralPath $exePath -PathType Leaf)) {
        Write-InstallLog "SteamCMD encontrado em '$exePath'. Validando instalacao existente..."
        if (Test-SteamCmdWorking -InstallDir $InstallDir -TimeoutSeconds $VerifyTimeoutSeconds -AllowBootstrapRetry:$false) {
            Write-InstallLog "SteamCMD existente esta funcionando corretamente."
            return
        }
        Write-InstallLog "Instalacao existente nao passou na verificacao; sera reinstalada."
    }

    $tempZip = [System.IO.Path]::Combine(
        [System.IO.Path]::GetTempPath(),
        "steamcmd_" + [Guid]::NewGuid().ToString("N") + ".zip"
    )

    try {
        Download-SteamCmdZip -Url $DownloadUrl -DestinationZip $tempZip
        Expand-SteamCmdZip -ZipPath $tempZip -DestinationDir $InstallDir
    } finally {
        Remove-Item -LiteralPath $tempZip -Force -ErrorAction SilentlyContinue
    }

    if (-not (Test-SteamCmdWorking -InstallDir $InstallDir -TimeoutSeconds $VerifyTimeoutSeconds -AllowBootstrapRetry:$true)) {
        throw "Falha ao verificar o funcionamento do SteamCMD em '$InstallDir'."
    }
}

try {
    Install-AndVerifySteamCmd
    exit 0
} catch {
    Write-InstallLog "ERRO: $($_.Exception.Message)"
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 1
}
