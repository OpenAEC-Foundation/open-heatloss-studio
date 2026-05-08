<#
.SYNOPSIS
Generate a 1024x1024 placeholder PNG for the app icon.

.DESCRIPTION
Creates a flat blue square with "I51" centered in white.
Run once on Windows; commit the resulting src-tauri/icons/source.png.
Replace with real branding when available.
#>

$ErrorActionPreference = 'Stop'

if (-not (Test-Path 'src-tauri/icons')) {
    New-Item -ItemType Directory -Path 'src-tauri/icons' | Out-Null
}

Add-Type -AssemblyName System.Drawing

$size = 1024
$bmp = New-Object System.Drawing.Bitmap $size, $size
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAlias

# Background: ISSO-blue
$bgColor = [System.Drawing.Color]::FromArgb(255, 26, 79, 122)
$g.Clear($bgColor)

# Text "I51" centered, white, bold Segoe UI
$brush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
$font = New-Object System.Drawing.Font 'Segoe UI', 280, ([System.Drawing.FontStyle]::Bold)
$format = New-Object System.Drawing.StringFormat
$format.Alignment = [System.Drawing.StringAlignment]::Center
$format.LineAlignment = [System.Drawing.StringAlignment]::Center
$rect = New-Object System.Drawing.RectangleF 0, 0, $size, $size
$g.DrawString('I51', $font, $brush, $rect, $format)

$outPath = 'src-tauri/icons/source.png'
$bmp.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)

$g.Dispose()
$bmp.Dispose()
$brush.Dispose()
$font.Dispose()

Write-Host "Wrote $outPath ($size x $size)"
