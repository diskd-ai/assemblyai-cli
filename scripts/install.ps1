$ErrorActionPreference = "Stop"

$repo = "diskd-ai/assemblyai-cli"
$binName = "assemblyai-cli.exe"

$tag = $env:ASSEMBLYAI_CLI_TAG
$installDir = $env:ASSEMBLYAI_CLI_INSTALL_DIR

if ([string]::IsNullOrWhiteSpace($installDir)) {
  $installDir = Join-Path $env:LOCALAPPDATA "assemblyai-cli\\bin"
}

function Ensure-Ffmpeg {
  if (Get-Command ffmpeg -ErrorAction SilentlyContinue) {
    return
  }

  Write-Host "ffmpeg not found; attempting to install..."

  if (Get-Command winget -ErrorAction SilentlyContinue) {
    try {
      & winget install --id Gyan.FFmpeg -e --accept-package-agreements --accept-source-agreements | Out-Null
    } catch {
    }
  }

  if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue) -and (Get-Command choco -ErrorAction SilentlyContinue)) {
    try {
      & choco install ffmpeg -y | Out-Null
    } catch {
    }
  }

  if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue) -and (Get-Command scoop -ErrorAction SilentlyContinue)) {
    try {
      & scoop install ffmpeg | Out-Null
    } catch {
    }
  }

  if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
    Write-Warning "ffmpeg still not found after installation attempt; video inputs will not work until ffmpeg is on PATH."
  }
}

if ([string]::IsNullOrWhiteSpace($tag)) {
  $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest" -Headers @{ "Accept" = "application/vnd.github+json" }
  $tag = $release.tag_name
}

if ([string]::IsNullOrWhiteSpace($tag)) {
  throw "Failed to resolve latest release tag"
}

$version = $tag -replace '^v',''
$target = "x86_64-pc-windows-msvc"
$asset = "assemblyai-cli-$version-$target.zip"
$baseUrl = "https://github.com/$repo/releases/download/$tag"

$tmpRoot = Join-Path ([IO.Path]::GetTempPath()) ("assemblyai-cli-" + [Guid]::NewGuid().ToString("n"))
New-Item -ItemType Directory -Force -Path $tmpRoot | Out-Null

try {
  $zipPath = Join-Path $tmpRoot $asset
  $shaPath = "$zipPath.sha256"

  Write-Host "downloading: $asset"
  $iwr = Get-Command Invoke-WebRequest
  $zipParams = @{ Uri = "$baseUrl/$asset"; OutFile = $zipPath }
  $shaParams = @{ Uri = "$baseUrl/$asset.sha256"; OutFile = $shaPath }
  if ($iwr.Parameters.ContainsKey("UseBasicParsing")) {
    $zipParams.UseBasicParsing = $true
    $shaParams.UseBasicParsing = $true
  }
  Invoke-WebRequest @zipParams
  Invoke-WebRequest @shaParams

  $expected = (Get-Content $shaPath -Raw).Split(@(" ", "`t", "`r", "`n"), [StringSplitOptions]::RemoveEmptyEntries)[0].ToLower()
  $actual = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash.ToLower()
  if ($expected -ne $actual) {
    throw "SHA256 mismatch for $asset"
  }

  $extractDir = Join-Path $tmpRoot "extract"
  Expand-Archive -Path $zipPath -DestinationPath $extractDir -Force

  New-Item -ItemType Directory -Force -Path $installDir | Out-Null
  Copy-Item -Path (Join-Path $extractDir $binName) -Destination (Join-Path $installDir $binName) -Force

  Write-Host "installed: $(Join-Path $installDir $binName)"
  Write-Host "note: add $installDir to PATH to run assemblyai-cli without a full path."

  Ensure-Ffmpeg
  Write-Host "next: run 'assemblyai-cli init' (or set ASSEMBLYAI_API_KEY) before transcribing."
} finally {
  Remove-Item -Recurse -Force $tmpRoot -ErrorAction SilentlyContinue
}
