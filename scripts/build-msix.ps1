# build-msix.ps1
# Script para empaquetar AnvilCraft Panel como MSIX para la Microsoft Store

$ProjectRoot = Get-Location
$StagingDir = "$ProjectRoot\msix_stage"
$AssetsDir = "$StagingDir\Assets"
$TauriReleaseDir = "$ProjectRoot\src-tauri\target\release"

# 1. Limpieza y preparación
Write-Host "--- Preparando entorno de empaquetado MSIX ---" -ForegroundColor Cyan
if (Test-Path $StagingDir) { Remove-Item -Recururse -Force $StagingDir }
New-Item -ItemType Directory -Path $AssetsDir

# 2. Copiar archivos de la aplicación
Write-Host "--- Copiando binarios de la aplicación ---"
$OriginalExe = "$TauriReleaseDir\anvil-craft.exe"
if (-Not (Test-Path $OriginalExe)) {
    $OriginalExe = Get-ChildItem -Path $TauriReleaseDir -Filter "*.exe" | Select-Object -ExpandProperty FullName -First 1
}

if (-Not $OriginalExe) {
    Write-Error "No se encontró ningún archivo .exe en $TauriReleaseDir"
    exit 1
}

Copy-Item $OriginalExe "$StagingDir\AnvilCraft Panel.exe"
Write-Host "Copiado y renombrado: $OriginalExe -> AnvilCraft Panel.exe"

# Copiar DLLs necesarias si existen
if (Test-Path "$TauriReleaseDir\*.dll") { Copy-Item "$TauriReleaseDir\*.dll" "$StagingDir\" }

# 3. Copiar Manifiesto
Copy-Item "$ProjectRoot\src-tauri\msix\AppxManifest.xml" "$StagingDir\"

# 4. Copiar Iconos a la carpeta Assets
Write-Host "--- Preparando recursos visuales ---"
$IconsSource = "$ProjectRoot\src-tauri\icons"
$RequiredIcons = @("StoreLogo.png", "Square150x150Logo.png", "Square44x44Logo.png", "Square310x310Logo.png")
foreach ($icon in $RequiredIcons) {
    Copy-Item "$IconsSource\$icon" "$AssetsDir\"
}

# 5. Generar Certificado de Firma Temporal
Write-Host "--- Generando certificado de firma temporal ---" -ForegroundColor Yellow
$CertPath = "$ProjectRoot\temp_cert.pfx"
$CertPassword = $env:MSIX_CERT_PASSWORD
if (-not $CertPassword) {
    Write-Error "MSIX_CERT_PASSWORD no esta configurado. Ejemplo: `$env:MSIX_CERT_PASSWORD = 'tu-password'"
    exit 1
}
$Publisher = "CN=4C5C6D2B-352B-4EDE-B886-2F082C336275"

$secpassword = ConvertTo-SecureString $CertPassword -AsPlainText -Force
$cert = New-SelfSignedCertificate -Type Custom -Subject $Publisher -KeyUsage DigitalSignature -FriendlyName "AnvilCraft Store Cert" -CertStoreLocation "Cert:\CurrentUser\My" -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")
Export-PfxCertificate -cert $cert -FilePath $CertPath -Password $secpassword
Write-Host "Certificado generado en $CertPath"

# 6. Empaquetar con MakeAppx
Write-Host "--- Empaquetando MSIX ---" -ForegroundColor Green
$MSIXPath = "$ProjectRoot\AnvilCraft.msix"
if (Test-Path $MSIXPath) { Remove-Item $MSIXPath }

# Buscar MakeAppx.exe en las rutas estándar de Windows SDK
$sdkPath = "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\makeappx.exe"
$makeappx = Resolve-Path $sdkPath | Select-Object -ExpandProperty Path -First 1

& $makeappx pack /d $StagingDir /p $MSIXPath /o

# 7. Firmar con SignTool
Write-Host "--- Firmando MSIX ---" -ForegroundColor Green
$signtoolPath = "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe"
$signtool = Resolve-Path $signtoolPath | Select-Object -ExpandProperty Path -First 1

& $signtool sign /fd SHA256 /a /f $CertPath /p $CertPassword $MSIXPath

Write-Host "--- PROCESO COMPLETADO ---" -ForegroundColor Cyan
Write-Host "Archivo generado: $MSIXPath"
