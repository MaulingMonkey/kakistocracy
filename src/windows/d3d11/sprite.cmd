@pushd "%~dp0"
@set PATH=C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64;%PATH%
:: COMMON                  VARYING                                 COMMON
fxc /Zi /Zss /O3 /nologo   /T ps_4_0 /E ps /Fo sprite.bin.ps_4_0   sprite.hlsl
fxc /Zi /Zss /O3 /nologo   /T vs_4_0 /E vs /Fo sprite.bin.vs_4_0   sprite.hlsl
@popd
