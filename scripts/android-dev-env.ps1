
# Must be dot-sourced so env changes persist in the caller
if ($MyInvocation.InvocationName -ne ".") {
    throw "Dot-source this script: . .\scripts\android-dev-env.ps1"
}

$env:ANDROID_NDK_HOME = "$env:LOCALAPPDATA\Android\Ndk\r27d"
$env:PATH = "$env:ANDROID_NDK_HOME\toolchains\llvm\prebuilt\windows-x86_64\bin;$env:PATH"

Write-Host "Android NDK environment enabled"
