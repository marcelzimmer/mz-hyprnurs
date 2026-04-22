# Maintainer: Marcel Zimmer <https://www.marcelzimmer.de>
pkgname=mz-hyprnurs
pkgver=1.0.1
pkgrel=1
pkgdesc="Desktop-App für die pflegerische Übergabe im Krankenhaus"
arch=('x86_64')
url="https://github.com/marcelzimmer/mz-hyprnurs"
license=('MIT')
depends=('gcc-libs')
source=("${pkgname}-${pkgver}.zip::https://github.com/marcelzimmer/mz-hyprnurs/releases/download/v${pkgver}/mz-hyprnurs-linux-x86_64.zip")
sha256sums=('SKIP')

package() {
    cd "$srcdir"

    install -Dm755 mz-hyprnurs "$pkgdir/usr/bin/mz-hyprnurs"

    install -Dm644 icon.png "$pkgdir/usr/share/icons/hicolor/256x256/apps/mz-hyprnurs.png"

    install -Dm644 /dev/stdin "$pkgdir/usr/share/applications/mz-hyprnurs.desktop" << EOF
[Desktop Entry]
Name=MZ-HyprNurs
Comment=Desktop-App für die pflegerische Übergabe im Krankenhaus
Exec=mz-hyprnurs
Icon=mz-hyprnurs
Type=Application
Categories=Office;MedicalSoftware;
Terminal=false
EOF
}
