Name: clipg
Version: 2.15.0
Release: 1
Summary: Aplicació de sincronització de partides guardades de videojocs (Client).

License: MIT
BuildArch: x86_64

%description
Aplicació de sincronització de partides guardades de videojocs (Client).

%install
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/icons/hicolor/256x256/apps

install -m 755 %{_sourcedir}/clipg %{buildroot}/usr/bin/clipg

cp %{_sourcedir}/clipg.desktop %{buildroot}/usr/share/applications/
cp %{_sourcedir}/clipg.png %{buildroot}/usr/share/icons/hicolor/256x256/apps/clipg.png

%files
/usr/bin/clipg
/usr/share/applications/clipg.desktop
/usr/share/icons/hicolor/256x256/apps/clipg.png
