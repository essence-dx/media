set shell := ["pwsh.exe", "-c"]

build:
    cargo build --release -j 12
    @$config = if (Test-Path dx-config.toml) { Get-Content dx-config.toml | ConvertFrom-Toml } else { $null }
    @$binDir = if ($config -and $config.workspace_root) { "$($config.workspace_root)\bin" } else { "G:\Dx\bin" }
    @New-Item -ItemType Directory -Force -Path $binDir | Out-Null
    @Copy-Item target\release\media.exe "$binDir\dx-media.exe" -Force






