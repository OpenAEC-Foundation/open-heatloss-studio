<#
.SYNOPSIS
Generate the Open Heatloss Studio source logo as a 1024x1024 PNG.

.DESCRIPTION
OpenAEC-stijl icoon: donker grijs achtergrond met amber/oranje
wireframe-tekening van een isometrisch wand-segment. Matcht de
OpenAEC Foundation huisstijl (charcoal background + amber outline,
technical-drawing aesthetic), specifiek voor heat-loss met de
laag-opbouw zichtbaar als binnen-grenzen van de wand-doorsnede en
een kleine warmte-richtingspijl.

Colour palette (afgeleid van OpenAEC org-avatar):
  - Background : OpenAEC charcoal #2A2D31
  - Wireframe  : OpenAEC amber #E67E22
  - Accent     : warmere amber #F39C12 voor heat-arrow

Run once on Windows; commit het resulting src-tauri/icons/source.png.
Regenereer de Tauri icon-set:

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
# Solid OpenAEC charcoal — flat, no gradient.
$colBg = [System.Drawing.Color]::FromArgb(255, 42, 45, 49)       # #2A2D31
$bgBrush = New-Object System.Drawing.SolidBrush $colBg
$g.FillRectangle($bgBrush, 0, 0, $size, $size)

# ---------------------------------------------------------------- isometric wall
# Een 3D-wand getoond in isometrisch perspectief. De wand is een rechte
# kubus (kortere kant naar voren) waar de DOORSNEDE-kant naar voren is
# gedraaid zodat je de laag-opbouw kunt zien als horizontale strepen op
# het front-vlak. Dit pakt zowel "AEC bouwkundig" als "warmteverlies-
# doorsnede" beeldend op in één wireframe-mark.
#
# Isometrische projectie:
#   - Een 3D-punt (x, y, z) projecteert naar 2D als
#       sx = (x − z) * cos(30°)
#       sy = y + (x + z) * sin(30°)
#   met 30° = π/6 radialen
#
# Onze wand is een box: width × height × depth, met
#   - x-as = naar rechts-naar-achter (depth)
#   - y-as = omhoog (height)
#   - z-as = naar voren (width)
# Hierdoor staat de "doorsnede" (z = 0) naar de kijker toe.

$angle = [Math]::PI / 6   # 30°
$cos = [Math]::Cos($angle)
$sin = [Math]::Sin($angle)

# Wand-afmetingen in canvas-units. Vult ~75% van canvas-breedte zodat
# het wireframe-mark dominant op het icoon staat (vergelijkbaar met de
# OpenAEC org-avatar die ~70% gevuld is).
$wallW = 540   # breedte van de doorsnede (z-richting, naar voren)
$wallH = 460   # hoogte
$wallD = 330   # diepte (x-richting, naar achter)

# centerX = canvas-midden. centerY = onderkant van de wand-box,
# gepositioneerd zodanig dat de iso-bounding-box verticaal gecentreerd is.
#
# Iso-projectie geeft bounding-box hoogte = wallH + wallD*sin.
# centerY = canvas-midden + (bbox-hoogte / 2)
$centerX = [int]($size / 2)
$totalH = $wallH + $wallD * $sin
$centerY = [int]($size / 2 + $totalH / 2)

function Project([double]$x, [double]$y, [double]$z) {
    # Iso projection — voor onze oriëntatie:
    #   sx = x*cos + (-z)*cos  →  (x - z) * cos
    #   sy = y + (x + z) * sin   (omgekeerd want canvas y groeit naar onder)
    $sx = $centerX + (($x - $z) * $cos)
    $sy = $centerY - ($y + ($x + $z) * $sin)
    return New-Object System.Drawing.PointF $sx, $sy
}

# Definieer de 8 hoekpunten van de wand-box
# Front (z = wallW/2): visible cross-section
# Back (z = -wallW/2): far face
# Left/right (x = ±wallD/2)
# Bottom/top (y = 0/wallH)
$x0 = -$wallD / 2.0   # left
$x1 =  $wallD / 2.0   # right
$y0 = 0.0             # bottom
$y1 = $wallH * 1.0    # top
$z0 = -$wallW / 2.0   # back
$z1 =  $wallW / 2.0   # front

# Punten (numbered)
$p000 = Project $x0 $y0 $z0   # back-left-bottom
$p100 = Project $x1 $y0 $z0   # back-right-bottom
$p110 = Project $x1 $y1 $z0   # back-right-top
$p010 = Project $x0 $y1 $z0   # back-left-top
$p001 = Project $x0 $y0 $z1   # front-left-bottom
$p101 = Project $x1 $y0 $z1   # front-right-bottom
$p111 = Project $x1 $y1 $z1   # front-right-top
$p011 = Project $x0 $y1 $z1   # front-left-top

# Pen voor het wireframe — dikker zodat hij ook leesbaar is op favicon
# size. OpenAEC org-avatar gebruikt ~2-3% van canvas-width als stroke.
$colWire = [System.Drawing.Color]::FromArgb(255, 230, 126, 34)   # #E67E22
$penMain = New-Object System.Drawing.Pen $colWire, 18
$penMain.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
$penMain.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
$penMain.EndCap = [System.Drawing.Drawing2D.LineCap]::Round

# Vlakken te tekenen: alleen ZICHTBARE randen (geen verstopte achterkant
# lijnen, voor schone look).
# Zichtbare randen in dit perspectief:
#   - Top face: 4 randen (p010-p110-p111-p011)
#   - Right face: 4 randen (p100-p101-p111-p110) — sharing met top
#   - Front face: 4 randen (p001-p101-p111-p011) — sharing met top + right
# Onzichtbare (achter): back-left-bottom hoek + bijbehorende randen.

# Front face (de doorsnede)
$g.DrawLine($penMain, $p001, $p101)   # front bottom
$g.DrawLine($penMain, $p101, $p111)   # front right
$g.DrawLine($penMain, $p111, $p011)   # front top
$g.DrawLine($penMain, $p011, $p001)   # front left

# Right face (zijaanzicht naar achter)
$g.DrawLine($penMain, $p101, $p100)   # bottom-right diagonal
$g.DrawLine($penMain, $p100, $p110)   # back-right vertical
$g.DrawLine($penMain, $p110, $p111)   # top-right diagonal

# Top face (dak/bovenkant)
$g.DrawLine($penMain, $p011, $p010)   # left-top diagonal naar back
$g.DrawLine($penMain, $p010, $p110)   # back-top

# ---------------------------------------------------------------- layers
# Op de front face (doorsnede) tekenen we 3-4 horizontale lijnen om de
# layered wall opbouw te suggereren. Dit is het unieke heat-loss element
# binnen de OpenAEC-stijl.
$penLayer = New-Object System.Drawing.Pen $colWire, 10
$penLayer.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
$penLayer.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
$penLayer.EndCap = [System.Drawing.Drawing2D.LineCap]::Round

$layerFractions = @(0.25, 0.50, 0.75)
foreach ($f in $layerFractions) {
    $yL = $wallH * $f
    $pL_left = Project $x0 $yL $z1   # front-left at this height
    $pL_right = Project $x1 $yL $z1   # front-right at this height
    $g.DrawLine($penLayer, $pL_left, $pL_right)
}

# (Heat-arrow boven de cube weggehaald — onleesbaar op favicon-sizes en
#  voegt visuele clutter toe. De gelaagde front-face is genoeg
#  signifier voor "warmteverlies door wandopbouw" + matcht beter de
#  minimalistische OpenAEC-stijl.)

# ---------------------------------------------------------------- finish
$outPath = 'src-tauri/icons/source.png'
$bmp.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)

$g.Dispose()
$bmp.Dispose()
$bgBrush.Dispose()
$penMain.Dispose()
$penLayer.Dispose()

Write-Host "Wrote $outPath ($size x $size)"
