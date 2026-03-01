param(
  [string]$AppName = "Pastry",
  [string]$IconSource = "assets/logo.png"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-CargoVersion {
  param(
    [Parameter(Mandatory = $false)][string]$CargoTomlPath = "Cargo.toml"
  )

  if (-not (Test-Path $CargoTomlPath)) {
    throw "Cargo.toml not found: $CargoTomlPath"
  }

  $inPackageSection = $false
  foreach ($line in Get-Content $CargoTomlPath) {
    if ($line -match '^\s*\[package\]\s*$') {
      $inPackageSection = $true
      continue
    }

    if ($inPackageSection -and $line -match '^\s*\[') {
      break
    }

    if ($inPackageSection -and $line -match '^\s*version\s*=\s*"([^"]+)"') {
      return $Matches[1]
    }
  }

  throw "Failed to read version from $CargoTomlPath"
}

function New-IconFromPng {
  param(
    [Parameter(Mandatory = $true)][string]$PngPath,
    [Parameter(Mandatory = $true)][string]$IcoPath
  )

  if (-not (Test-Path $PngPath)) {
    throw "Icon source file not found: $PngPath"
  }

  Add-Type -AssemblyName System.Drawing

  $sizes = @(16, 32, 48, 64, 128, 256)
  $image = [System.Drawing.Image]::FromFile((Resolve-Path $PngPath))

  $pngBlobs = New-Object System.Collections.Generic.List[byte[]]
  $entries = New-Object System.Collections.Generic.List[object]

  try {
    foreach ($size in $sizes) {
      $bmp = New-Object System.Drawing.Bitmap($size, $size)
      try {
        $graphics = [System.Drawing.Graphics]::FromImage($bmp)
        try {
          $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
          $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
          $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
          $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
          $graphics.DrawImage($image, 0, 0, $size, $size)
        }
        finally {
          $graphics.Dispose()
        }

        $ms = New-Object System.IO.MemoryStream
        try {
          $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
          $data = $ms.ToArray()
          [void]$pngBlobs.Add($data)
          [void]$entries.Add([PSCustomObject]@{
            Width  = if ($size -ge 256) { 0 } else { $size }
            Height = if ($size -ge 256) { 0 } else { $size }
            Bytes  = $data.Length
          })
        }
        finally {
          $ms.Dispose()
        }
      }
      finally {
        $bmp.Dispose()
      }
    }
  }
  finally {
    $image.Dispose()
  }

  $dir = Split-Path -Parent $IcoPath
  if ($dir -and -not (Test-Path $dir)) {
    New-Item -ItemType Directory -Path $dir | Out-Null
  }

  $stream = [System.IO.File]::Open($IcoPath, [System.IO.FileMode]::Create, [System.IO.FileAccess]::Write)
  $writer = New-Object System.IO.BinaryWriter($stream)
  try {
    $writer.Write([UInt16]0)
    $writer.Write([UInt16]1)
    $writer.Write([UInt16]$entries.Count)

    $offset = 6 + (16 * $entries.Count)
    for ($i = 0; $i -lt $entries.Count; $i++) {
      $entry = $entries[$i]
      $writer.Write([byte]$entry.Width)
      $writer.Write([byte]$entry.Height)
      $writer.Write([byte]0)
      $writer.Write([byte]0)
      $writer.Write([UInt16]1)
      $writer.Write([UInt16]32)
      $writer.Write([UInt32]$entry.Bytes)
      $writer.Write([UInt32]$offset)
      $offset += $entry.Bytes
    }

    for ($i = 0; $i -lt $pngBlobs.Count; $i++) {
      $writer.Write($pngBlobs[$i])
    }
  }
  finally {
    $writer.Close()
    $stream.Dispose()
  }
}

Write-Host "Generating icon from $IconSource ..."
New-IconFromPng -PngPath $IconSource -IcoPath "assets/logo.ico"

$version = Get-CargoVersion

Write-Host "Building release executable ..."
cargo build --release

$exeSource = Join-Path "target/release" "pastry.exe"
if (-not (Test-Path $exeSource)) {
  throw "Build failed: executable not found at $exeSource"
}

$distRoot = "target/windows-release"
$appDir = Join-Path $distRoot $AppName
$exeTarget = Join-Path $appDir ($AppName + ".exe")
$iconTarget = Join-Path $appDir "logo.ico"
$archivePath = Join-Path $distRoot ("$AppName-$version-windows.zip")

if (Test-Path $appDir) {
  Remove-Item -Recurse -Force $appDir
}
if (Test-Path $archivePath) {
  Remove-Item -Force $archivePath
}

New-Item -ItemType Directory -Path $appDir | Out-Null
Copy-Item $exeSource $exeTarget
Copy-Item "assets/logo.ico" $iconTarget
Compress-Archive -Path $appDir -DestinationPath $archivePath -Force

Write-Host ""
Write-Host "Done."
Write-Host "Executable: $exeTarget"
Write-Host "Icon file : $iconTarget"
Write-Host "Archive file: $archivePath"
