# Descarga yt-dlp y ffmpeg (build essentials de gyan.dev) en src-tauri/bin,
# comprobando el SHA-256 que publica cada proyecto. Se meten en el .exe al compilar.
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$bin = Join-Path $PSScriptRoot "..\src-tauri\bin"
$tmp = Join-Path ([IO.Path]::GetTempPath()) "tunadl-binarios"
New-Item -ItemType Directory -Force $bin, $tmp | Out-Null

function Comprobar($archivo, $esperado) {
    $real = (Get-FileHash $archivo -Algorithm SHA256).Hash.ToLower()
    if ($real -ne $esperado.Trim().ToLower()) { throw "SHA-256 no coincide en $archivo" }
}

# Algunos servidores mandan los .sha256 como binario: se pasa siempre a texto.
function Texto($url) {
    $c = (Invoke-WebRequest $url -UseBasicParsing).Content
    if ($c -is [byte[]]) { [Text.Encoding]::ASCII.GetString($c) } else { $c }
}

# yt-dlp
$base = "https://github.com/yt-dlp/yt-dlp/releases/latest/download"
Invoke-WebRequest "$base/yt-dlp.exe" -OutFile "$tmp\yt-dlp.exe"
$sumas = Texto "$base/SHA2-256SUMS"
$linea = ($sumas -split "\r?\n") | Where-Object { $_ -match "\syt-dlp\.exe$" }
Comprobar "$tmp\yt-dlp.exe" ($linea -split "\s+")[0]
Copy-Item "$tmp\yt-dlp.exe" $bin -Force

# ffmpeg + ffprobe
$zip = "$tmp\ffmpeg.zip"
Invoke-WebRequest "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip" -OutFile $zip
Comprobar $zip (Texto "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip.sha256")
Expand-Archive $zip "$tmp\ffmpeg" -Force
Get-ChildItem "$tmp\ffmpeg" -Recurse -Include ffmpeg.exe, ffprobe.exe | Copy-Item -Destination $bin -Force

Remove-Item $tmp -Recurse -Force
Get-ChildItem $bin *.exe | Format-Table Name, Length
