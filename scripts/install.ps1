#!/usr/bin/env pwsh
param(
  [String]$Version = "latest",
  # Skips adding the luma.exe directory to the user's %PATH%
  [Switch]$NoPathUpdate = $false,
  # Skips adding luma to the list of installed programs
  [Switch]$NoRegisterInstallation = $false,

  # Debugging: Always download with 'Invoke-RestMethod' instead of 'curl.exe'
  [Switch]$DownloadWithoutCurl = $false
);

# filter out 32 bit + ARM
if (-not ((Get-CimInstance Win32_ComputerSystem)).SystemType -match "x64-based") {
  Write-Output "Install Failed:"
  Write-Output "Luma for Windows is currently only available for x86 64-bit Windows.`n"
  return 1
}

$ErrorActionPreference = "Stop"

# These three environment functions are roughly copied from https://github.com/prefix-dev/pixi/pull/692
# They are used instead of `SetEnvironmentVariable` because of unwanted variable expansions.
function Publish-Env {
  if (-not ("Win32.NativeMethods" -as [Type])) {
    Add-Type -Namespace Win32 -Name NativeMethods -MemberDefinition @"
[DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
public static extern IntPtr SendMessageTimeout(
    IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
    uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
"@
  }
  $HWND_BROADCAST = [IntPtr] 0xffff
  $WM_SETTINGCHANGE = 0x1a
  $result = [UIntPtr]::Zero
  [Win32.NativeMethods]::SendMessageTimeout($HWND_BROADCAST,
    $WM_SETTINGCHANGE,
    [UIntPtr]::Zero,
    "Environment",
    2,
    5000,
    [ref] $result
  ) | Out-Null
}

function Write-Env {
  param([String]$Key, [String]$Value)

  $RegisterKey = Get-Item -Path 'HKCU:'

  $EnvRegisterKey = $RegisterKey.OpenSubKey('Environment', $true)
  if ($null -eq $Value) {
    $EnvRegisterKey.DeleteValue($Key)
  } else {
    $RegistryValueKind = if ($Value.Contains('%')) {
      [Microsoft.Win32.RegistryValueKind]::ExpandString
    } elseif ($EnvRegisterKey.GetValue($Key)) {
      $EnvRegisterKey.GetValueKind($Key)
    } else {
      [Microsoft.Win32.RegistryValueKind]::String
    }
    $EnvRegisterKey.SetValue($Key, $Value, $RegistryValueKind)
  }

  Publish-Env
}

function Get-Env {
  param([String] $Key)

  $RegisterKey = Get-Item -Path 'HKCU:'
  $EnvRegisterKey = $RegisterKey.OpenSubKey('Environment')
  $EnvRegisterKey.GetValue($Key, $null, [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
}

# The installation of luma is it's own function for better error handling.
# There are also lots of sanity checks out of fear of anti-virus software or other weird Windows things happening.
function Install-Luma {
  param(
    [string]$Version
  );

  # if a semver is given, we need to adjust it to this format: v0.0.0
  if ($Version -match "^\d+\.\d+\.\d+$") {
    $Version = "v$Version"
  }
  elseif ($Version -match "^v\d+\.\d+\.\d+$") {
    $Version = "$Version"
  }

  $Arch = "x64"

  $LumaRoot = if ($env:LUMA_INSTALL) { $env:LUMA_INSTALL } else { "${Home}\.luma" }
  $LumaBin = mkdir -Force "${LumaRoot}\bin"

  try {
    Remove-Item "${LumaBin}\${AppExe}" -Force
  } catch [System.Management.Automation.ItemNotFoundException] {
    # ignore
  } catch [System.UnauthorizedAccessException] {
    $openProcesses = Get-Process -Name luma | Where-Object { $_.Path -eq "${LumaBin}\luma.exe" }
    if ($openProcesses.Count -gt 0) {
      Write-Output "Install Failed - An older installation exists and is open. Please close open Luma processes and try again."
      return 1
    }
    Write-Output "Install Failed - An unknown error occurred while trying to remove the existing installation"
    Write-Output $_
    return 1
  } catch {
    Write-Output "Install Failed - An unknown error occurred while trying to remove the existing installation"
    Write-Output $_
    return 1
  }

  $Target = "windows-${Arch}"
  $BaseURL = "https://github.com/tayadev/luma/releases"
  $URL = "$BaseURL/$(if ($Version -eq "latest") { "latest/download" } else { "download/$Version" })/$Target.zip"

  $ZipPath = "${LumaBin}\$Target.zip"

  $DisplayVersion = $(
    if ($Version -eq "latest") { "Luma" }
    elseif ($Version -match "^v\d+\.\d+\.\d+$") { "Luma $($Version.Substring(7))" }
    else { "Luma tag='${Version}'" }
  )

  $null = mkdir -Force $LumaBin
  Remove-Item -Force $ZipPath -ErrorAction SilentlyContinue

  # curl.exe is faster than PowerShell 5's 'Invoke-WebRequest'
  # note: 'curl' is an alias to 'Invoke-WebRequest'. so the exe suffix is required
  if (-not $DownloadWithoutCurl) {
    curl.exe "-#SfLo" "$ZipPath" "$URL" 
  }
  if ($DownloadWithoutCurl -or ($LASTEXITCODE -ne 0)) {
    Write-Warning "The command 'curl.exe $URL -o $ZipPath' exited with code ${LASTEXITCODE}`nTrying an alternative download method..."
    try {
      # Use Invoke-RestMethod instead of Invoke-WebRequest because Invoke-WebRequest breaks on
      # some machines, see 
      Invoke-RestMethod -Uri $URL -OutFile $ZipPath
    } catch {
      Write-Output "Install Failed - could not download $URL"
      Write-Output "The command 'Invoke-RestMethod $URL -OutFile $ZipPath' exited with code ${LASTEXITCODE}`n"
      return 1
    }
  }

  if (!(Test-Path $ZipPath)) {
    Write-Output "Install Failed - could not download $URL"
    Write-Output "The file '$ZipPath' does not exist. Did an antivirus delete it?`n"
    return 1
  }

  try {
    $lastProgressPreference = $global:ProgressPreference
    $global:ProgressPreference = 'SilentlyContinue';
    Expand-Archive "$ZipPath" "$LumaBin" -Force
    $global:ProgressPreference = $lastProgressPreference
    if (!(Test-Path "${LumaBin}\$Target\luma.exe")) {
      throw "The file '${LumaBin}\$Target\luma.exe' does not exist. Download is corrupt or intercepted Antivirus?`n"
    }
  } catch {
    Write-Output "Install Failed - could not unzip $ZipPath"
    Write-Error $_
    return 1
  }

  Move-Item "${LumaBin}\$Target\luma.exe" "${LumaBin}\luma.exe" -Force

  Remove-Item "${LumaBin}\$Target" -Recurse -Force
  Remove-Item $ZipPath -Force

  $LumaVersion = "$(& "${LumaBin}\luma.exe" --version)"
  if ($LASTEXITCODE -ne 0) {
    Write-Output "Install Failed - could not verify luma.exe"
    Write-Output "The command '${LumaBin}\luma.exe --version' exited with code ${LASTEXITCODE}`n"
    return 1
  }

  $DisplayVersion = $LumaVersion

  $C_RESET = [char]27 + "[0m"
  $C_GREEN = [char]27 + "[1;32m"

  Write-Output "${C_GREEN}Luma ${DisplayVersion} was installed successfully!${C_RESET}"
  Write-Output "The binary is located at ${LumaBin}\luma.exe`n"

  $hasExistingOther = $false;
  try {
    $existing = Get-Command luma -ErrorAction SilentlyContinue
    if ($existing -and ($existing.Source -ne "${LumaBin}\luma.exe")) {
      Write-Warning "Note: Another luma.exe is already in %PATH% at $($existing.Source)`nTyping 'luma' in your terminal will not use what was just installed.`n"
      $hasExistingOther = $true;
    }
  } catch {}

  if (-not $NoRegisterInstallation) {
    $rootKey = $null
    try {
      $RegistryKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Luma"  
      $rootKey = New-Item -Path $RegistryKey -Force
      New-ItemProperty -Path $RegistryKey -Name "DisplayName" -Value "Luma" -PropertyType String -Force | Out-Null
      New-ItemProperty -Path $RegistryKey -Name "InstallLocation" -Value "${LumaRoot}" -PropertyType String -Force | Out-Null
      New-ItemProperty -Path $RegistryKey -Name "DisplayIcon" -Value $LumaBin\luma.exe -PropertyType String -Force | Out-Null
      New-ItemProperty -Path $RegistryKey -Name "UninstallString" -Value "powershell -c `"& `'$LumaRoot\uninstall.ps1`' -PauseOnError`" -ExecutionPolicy Bypass" -PropertyType String -Force | Out-Null
    } catch {
      if ($rootKey -ne $null) {
        Remove-Item -Path $RegistryKey -Force
      }
    }
  }

  if(!$hasExistingOther) {
    # Only try adding to path if there isn't already a luma.exe in the path
    $Path = (Get-Env -Key "Path") -split ';'
    if ($Path -notcontains $LumaBin) {
      if (-not $NoPathUpdate) {
        $Path += $LumaBin
        Write-Env -Key 'Path' -Value ($Path -join ';')
        $env:PATH = $Path -join ';'
      } else {
        Write-Output "Skipping adding '${LumaBin}' to the user's %PATH%`n"
      }
    }

    Write-Output "To get started, restart your terminal/editor, then type `"luma`"`n"
  }

  $LASTEXITCODE = 0;
}

Install-Luma -Version $Version