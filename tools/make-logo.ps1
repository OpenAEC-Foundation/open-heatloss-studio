<#
.SYNOPSIS
Generate the Open Heatloss Studio source logo as a 1024x1024 PNG.

.DESCRIPTION
Modern flat-design icon: stylized wall cross-section with a thermal
gradient (warm interior -> cool exterior) suggesting laagopbouw +
warmteverlies. Single bold shape, no skeuomorphic shadows.

Colour palette:
  - Background : solid OHS teal (#0F766E)
  - Wall frame : white, no shadow
  - Gradient   : warm coral (#EF4444) -> amber (#F59E0B) -> cool cyan (#06B6D4)
  - Arrow      : white, indicates direction of heat flow

Run once on Windows; commit the resulting src-tauri/icons/source.png.
Then re-run the Tauri icon generator to refresh the icon-set:

    npx @tauri-apps/cli icon src-tauri/icons/source.png
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
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality

# ---------------------------------------------------------------- background
# Solid teal — flat-design, no vertical gradient. iOS/macOS app-icon style
# squircle is added implicitly by Tauri's icon generator when targeting
# those platforms; PNG source stays a square.
$colBg = [System.Drawing.Color]::FromArgb(255, 15, 118, 110)   # #0F766E
$bgBrush = New-Object System.Drawing.SolidBrush $colBg
$g.FillRectangle($bgBrush, 0, 0, $size, $size)

# ---------------------------------------------------------------- wall card
# Centered rounded-rectangle card representing a wall cross-section.
# Three horizontal "layers" inside, each with a thermal gradient color
# (warm at top = interior side, cool at bottom = exterior side).
$cardW = 640
$cardH = 720
$cardLeft = ([int](($size - $cardW) / 2))
$cardTop  = ([int](($size - $cardH) / 2))
$cornerR = 56

function New-RoundedRectPath {
    param([int]$x, [int]$y, [int]$w, [int]$h, [int]$r)
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddArc($x, $y, $r * 2, $r * 2, 180, 90)
    $path.AddArc(($x + $w - $r * 2), $y, $r * 2, $r * 2, 270, 90)
    $path.AddArc(($x + $w - $r * 2), ($y + $h - $r * 2), $r * 2, $r * 2, 0, 90)
    $path.AddArc($x, ($y + $h - $r * 2), $r * 2, $r * 2, 90, 90)
    $path.CloseFigure()
    return $path
}

# White card backing — subtle off-white so the warm/cool stripes pop.
$cardPath = New-RoundedRectPath $cardLeft $cardTop $cardW $cardH $cornerR
$cardBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 245, 245, 245))
$g.FillPath($cardBrush, $cardPath)

# Clip subsequent draws to the rounded card so the stripes stay inside.
$g.SetClip($cardPath)

# ---------------------------------------------------------------- stripes
# Three horizontal layer-stripes that span the card width.
# Each stripe has a soft vertical fade so the boundaries are smooth.
$stripes = @(
    @{ Color = [System.Drawing.Color]::FromArgb(255, 239, 68,  68); HeightFrac = 0.34 },  # coral (interior)
    @{ Color = [System.Drawing.Color]::FromArgb(255, 245, 158, 11); HeightFrac = 0.32 },  # amber (mid)
    @{ Color = [System.Drawing.Color]::FromArgb(255,   6, 182, 212); HeightFrac = 0.34 }  # cyan  (exterior)
)

$stripeY = $cardTop
for ($i = 0; $i -lt $stripes.Count; $i++) {
    $sh = [int]($cardH * $stripes[$i].HeightFrac)
    $rect = New-Object System.Drawing.Rectangle $cardLeft, $stripeY, $cardW, $sh
    $brush = New-Object System.Drawing.SolidBrush $stripes[$i].Color
    $g.FillRectangle($brush, $rect)
    $brush.Dispose()
    $stripeY += $sh
}

# Soft horizontal blend bands at the stripe boundaries — two small gradient
# rects that fade adjacent colors into each other for a smooth transition
# instead of hard color jumps.
$blendH = 60
$boundary1Y = $cardTop + [int]($cardH * $stripes[0].HeightFrac) - [int]($blendH / 2)
$boundary2Y = $cardTop + [int]($cardH * ($stripes[0].HeightFrac + $stripes[1].HeightFrac)) - [int]($blendH / 2)

$blend1Rect = New-Object System.Drawing.Rectangle $cardLeft, $boundary1Y, $cardW, $blendH
$blend1Brush = New-Object System.Drawing.Drawing2D.LinearGradientBrush `
    $blend1Rect, $stripes[0].Color, $stripes[1].Color, 90
$g.FillRectangle($blend1Brush, $blend1Rect)
$blend1Brush.Dispose()

$blend2Rect = New-Object System.Drawing.Rectangle $cardLeft, $boundary2Y, $cardW, $blendH
$blend2Brush = New-Object System.Drawing.Drawing2D.LinearGradientBrush `
    $blend2Rect, $stripes[1].Color, $stripes[2].Color, 90
$g.FillRectangle($blend2Brush, $blend2Rect)
$blend2Brush.Dispose()

$g.ResetClip()

# Thin white border around the card to crisp the outline.
$cardPen = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(255, 245, 245, 245)), 10
$cardPen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
$g.DrawPath($cardPen, $cardPath)

# ---------------------------------------------------------------- heat-flow arrow
# Bold arrow on the left pointing INTO the wall — heat flows from
# warm interior (left/top) toward cool exterior. White, ~120px tall,
# positioned just outside the card at the warm-stripe vertical center.
$arrowCy = $cardTop + [int]($cardH * 0.17)         # vertical center in warm zone
$arrowTipX = $cardLeft - 28
$arrowTailX = $arrowTipX - 160
$arrowHalfH = 38

$arrowPath = New-Object System.Drawing.Drawing2D.GraphicsPath
$arrowPts = New-Object 'System.Drawing.PointF[]' 7
$arrowPts[0] = New-Object System.Drawing.PointF $arrowTipX, $arrowCy
$arrowPts[1] = New-Object System.Drawing.PointF ($arrowTipX - 60), ($arrowCy - $arrowHalfH)
$arrowPts[2] = New-Object System.Drawing.PointF ($arrowTipX - 60), ($arrowCy - 14)
$arrowPts[3] = New-Object System.Drawing.PointF $arrowTailX, ($arrowCy - 14)
$arrowPts[4] = New-Object System.Drawing.PointF $arrowTailX, ($arrowCy + 14)
$arrowPts[5] = New-Object System.Drawing.PointF ($arrowTipX - 60), ($arrowCy + 14)
$arrowPts[6] = New-Object System.Drawing.PointF ($arrowTipX - 60), ($arrowCy + $arrowHalfH)
$arrowPath.AddPolygon($arrowPts)
$arrowPath.CloseFigure()

$arrowBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
$g.FillPath($arrowBrush, $arrowPath)
$arrowBrush.Dispose()
$arrowPath.Dispose()

# ---------------------------------------------------------------- finish
$outPath = 'src-tauri/icons/source.png'
$bmp.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)

$g.Dispose()
$bmp.Dispose()
$bgBrush.Dispose()
$cardBrush.Dispose()
$cardPath.Dispose()
$cardPen.Dispose()

Write-Host "Wrote $outPath ($size x $size)"
