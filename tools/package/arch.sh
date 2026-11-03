#!/usr/bin/env bash
# Builds the Arch package from the tarball `make dist` wrote and puts it into a pacman repository of
# its own in dist/repo/ARCH: pchart.db with the package next to it, ready to be served from any web
# folder. Signs both when PCHART_SIGN_KEY names a GPG key; without it they stay unsigned.
# usage: tools/package/arch.sh VERSION
set -euo pipefail

version=${1:?usage: arch.sh VERSION}
root=$(cd "$(dirname "$0")/../.." && pwd)
tarball="$root/dist/pchart-$version.tar.gz"
[[ -f $tarball ]] || { echo "arch.sh: $tarball is missing; make dist writes it" >&2; exit 1; }
for tool in makepkg repo-add; do
  command -v "$tool" >/dev/null || { echo "arch.sh needs $tool (pacman)" >&2; exit 1; }
done

build="$root/dist/arch"
rm -rf "$build"
mkdir -p "$build"
cp "$tarball" "$build/"
sum=$(sha256sum "$tarball" | cut -d' ' -f1)
sed -e "s/^pkgver=.*/pkgver=$version/" -e "s/^sha256sums=.*/sha256sums=('$sum')/" "$root/packaging/arch/PKGBUILD" > "$build/PKGBUILD"

sign=()
[[ -n ${PCHART_SIGN_KEY:-} ]] && sign=(--sign --key "$PCHART_SIGN_KEY")
(cd "$build" && makepkg --cleanbuild --force "${sign[@]}")

arch=$(uname -m)
repo="$root/dist/repo/$arch"
mkdir -p "$repo"
for package in "$build"/pchart-"$version"-*.pkg.tar.zst; do
  cp "$package" "$repo/"
  [[ -f $package.sig ]] && cp "$package.sig" "$repo/"
  repo-add "${sign[@]}" --remove "$repo/pchart.db.tar.zst" "$repo/$(basename "$package")"
done
echo "The repository is in $repo; serve that folder and add to /etc/pacman.conf:"
echo "  [pchart]"
echo "  Server = https://…/\$arch"
[[ ${#sign[@]} -eq 0 ]] && echo "It is unsigned; set PCHART_SIGN_KEY to sign it, or add SigLevel = Optional TrustAll."
exit 0
