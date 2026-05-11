<#
.SYNOPSIS
Generate the Open Heatloss Studio source logo as a 1024x1024 PNG.

.DESCRIPTION
Replaces the I51 placeholder with a thematic icon: stylized house
silhouette with heat-waves escaping the roof — visualising warmteverlies.

Colour palette:
  - Background  : OHS teal (#0F766E) on top, deeper teal (#134E4A) bottom
  - House       : white silhouette, soft drop-shadow
  - Heat waves  : warm coral/amber gradient (#F59E0B → #EF4444)

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
# Vertical teal gradient (top lighter, bottom darker) — gives the icon depth
$bgRect = New-Object System.Drawing.Rectangle 0, 0, $size, $size
$colTop = [System.Drawing.Color]::FromArgb(255, 15, 118, 110)   # #0F766E
$colBot = [System.Drawing.Color]::FromArgb(255, 19, 78, 74)     # #134E4A
$bgBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush $bgRect, $colTop, $colBot, 90
$g.FillRectangle($bgBrush, $bgRect)

# ------------------------------------------------------------- house outline
# Square body + triangle roof, centered. Width 560 px, total height 640 px.
# Use a GraphicsPath so we can both fill and stroke.
$bodyW = 560
$bodyH = 380
$roofH = 220
$cx = [int]($size / 2)
$bodyTop = [int]($size / 2 + 60)    # body starts below center for visual balance
$bodyLeft = $cx - [int]($bodyW / 2)
$bodyRight = $cx + [int]($bodyW / 2)
$bodyBot = $bodyTop + $bodyH
$roofPeak = $bodyTop - $roofH
$roofEaveOverhang = 40              # roof line a bit wider than body

$housePath = New-Object System.Drawing.Drawing2D.GraphicsPath
$pts = New-Object 'System.Drawing.PointF[]' 7
$pts[0] = New-Object System.Drawing.PointF $bodyLeft, $bodyBot
$pts[1] = New-Object System.Drawing.PointF $bodyLeft, $bodyTop
$pts[2] = New-Object System.Drawing.PointF ($bodyLeft - $roofEaveOverhang), $bodyTop
$pts[3] = New-Object System.Drawing.PointF $cx, $roofPeak
$pts[4] = New-Object System.Drawing.PointF ($bodyRight + $roofEaveOverhang), $bodyTop
$pts[5] = New-Object System.Drawing.PointF $bodyRight, $bodyTop
$pts[6] = New-Object System.Drawing.PointF $bodyRight, $bodyBot
$housePath.AddPolygon($pts)
$housePath.CloseFigure()

# Soft drop-shadow under the house — render a blurred-ish dark copy below
$shadowOffset = 14
$shadowPath = [System.Drawing.Drawing2D.GraphicsPath]$housePath.Clone()
$shadowMatrix = New-Object System.Drawing.Drawing2D.Matrix
$shadowMatrix.Translate($shadowOffset, $shadowOffset)
$shadowPath.Transform($shadowMatrix)
$shadowBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(60, 0, 0, 0))
$g.FillPath($shadowBrush, $shadowPath)

# House: white fill so it reads as a strong silhouette
$houseBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
$g.FillPath($houseBrush, $housePath)

# Subtle stroke to crisp the edges
$housePen = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(255, 6, 78, 59)), 6
$housePen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
$g.DrawPath($housePen, $housePath)

# ---------------------------------------------------------------- "warmth"
# Glow + door cut-out: rounded square in warm amber inside the body —
# reads as a lit room, the source of heat.
$glowW = 220
$glowH = 220
$glowLeft = $cx - [int]($glowW / 2)
$glowTop = $bodyBot - $glowH - 40
$glowRect = New-Object System.Drawing.Rectangle $glowLeft, $glowTop, $glowW, $glowH
$glowCenter = New-Object System.Drawing.PointF ($cx + 0), ($glowTop + $glowH / 2 + 0)

$glowPath = New-Object System.Drawing.Drawing2D.GraphicsPath
$cornerR = 32
$glowPath.AddArc($glowLeft, $glowTop, $cornerR * 2, $cornerR * 2, 180, 90)
$glowPath.AddArc(($glowLeft + $glowW - $cornerR * 2), $glowTop, $cornerR * 2, $cornerR * 2, 270, 90)
$glowPath.AddArc(($glowLeft + $glowW - $cornerR * 2), ($glowTop + $glowH - $cornerR * 2), $cornerR * 2, $cornerR * 2, 0, 90)
$glowPath.AddArc($glowLeft, ($glowTop + $glowH - $cornerR * 2), $cornerR * 2, $cornerR * 2, 90, 90)
$glowPath.CloseFigure()

# Warm amber gradient: brighter at top, deeper coral at bottom (flame-like)
$glowBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush `
    (New-Object System.Drawing.Rectangle $glowLeft, $glowTop, $glowW, $glowH), `
    ([System.Drawing.Color]::FromArgb(255, 251, 191, 36)), `
    ([System.Drawing.Color]::FromArgb(255, 239, 68, 68)), `
    90
$g.FillPath($glowBrush, $glowPath)

# ---------------------------------------------------------------- heat waves
# Three RISING flame-wisps escaping above the roof — each a vertical
# sinuous ribbon that tapers toward the top (heat-loss reads upward).
# Offset the three wisps horizontally so they don't stack.
$waveBaseY = $roofPeak - 20          # just above the roof peak
$baseOffsets = @(-110, 0, 110)        # left, center, right
$lengths     = @(360, 440, 360)       # how tall each wisp is
$widthsBot   = @(34, 44, 34)          # bottom thickness (warm root)
$widthsTop   = @(8, 10, 8)            # tip thickness (cool/fading)
$alphasBot   = @(255, 255, 255)
$alphasTop   = @(80, 110, 80)
$swayAmp     = @(34, 42, 34)          # horizontal sway
$swayPhase   = @(0.0, 0.4, 0.8)       # different phase per wisp

$colHot  = [System.Drawing.Color]::FromArgb(255, 239, 68, 68)   # coral
$colWarm = [System.Drawing.Color]::FromArgb(255, 251, 191, 36)  # amber

for ($i = 0; $i -lt 3; $i++) {
    $segments = 24
    $xBase = $cx + $baseOffsets[$i]
    $len   = $lengths[$i]
    $amp   = $swayAmp[$i]
    $phase = $swayPhase[$i]

    # Polygon: trace the right edge upward (sway + taper), then back down
    # the left edge so we get a closed ribbon shape that can be filled
    # with a vertical gradient.
    $pts = New-Object 'System.Drawing.PointF[]' (($segments + 1) * 2)

    for ($j = 0; $j -le $segments; $j++) {
        $t = $j / [double]$segments         # 0 at base, 1 at tip
        $y = $waveBaseY - $t * $len
        # Width tapers linearly from widthsBot[i] down to widthsTop[i]
        $w = $widthsBot[$i] * (1 - $t) + $widthsTop[$i] * $t
        # Horizontal sway: stronger near the top, plus a phase per wisp
        $sway = [Math]::Sin($t * [Math]::PI * 1.8 + $phase) * $amp * $t
        $cxLocal = $xBase + $sway
        $pts[$j] = New-Object System.Drawing.PointF ($cxLocal + $w / 2), $y
        $pts[($segments * 2 + 1) - $j] = New-Object System.Drawing.PointF ($cxLocal - $w / 2), $y
    }

    $wispPath = New-Object System.Drawing.Drawing2D.GraphicsPath
    $wispPath.AddCurve($pts, 0.5)
    $wispPath.CloseFigure()

    # Vertical gradient: hot amber at base → coral at mid → soft red fade at tip
    $wispRect = New-Object System.Drawing.Rectangle ($xBase - 60), ($waveBaseY - $len - 10), 220, ($len + 30)
    $cBot = [System.Drawing.Color]::FromArgb($alphasBot[$i], $colWarm.R, $colWarm.G, $colWarm.B)
    $cTop = [System.Drawing.Color]::FromArgb($alphasTop[$i], $colHot.R, $colHot.G, $colHot.B)
    $wispBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush $wispRect, $cBot, $cTop, 270

    $g.FillPath($wispBrush, $wispPath)
    $wispBrush.Dispose()
    $wispPath.Dispose()
}

# ---------------------------------------------------------------- finish
$outPath = 'src-tauri/icons/source.png'
$bmp.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)

$g.Dispose()
$bmp.Dispose()
$bgBrush.Dispose()
$shadowBrush.Dispose()
$houseBrush.Dispose()
$housePen.Dispose()
$glowBrush.Dispose()

Write-Host "Wrote $outPath ($size x $size)"
