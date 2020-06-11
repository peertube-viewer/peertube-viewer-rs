#!/bin/sh

release=$(git describe --tags|sed 's/^v//')

ssh-agent
ssh-add "$AUR_DEPLOY_KEY"
git clone 'ssh://aur@aur.archlinux.org/peertube-viewer-rs.git'
cd peertube-viewer-rs
sed "s/^pkgver=$/pkgver=$release/" ../raw_PKGBUILD > PKGBUILD
sed -i "s/^#md5sums=$/$(makepkg -g)/" PKGBUILD
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD
git add .SRCINFO
git commit -m "Release $release"
cat PKGBUILD
cat .SRCINFO
