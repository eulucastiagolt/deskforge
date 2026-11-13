#!/bin/bash
set -euo pipefail
root=$1
lib="$root/usr/lib"
mkdir -p "$lib" "$root/usr/share/icons" "$root/usr/share/glib-2.0/schemas"
seeds=("$root/usr/bin/deskforge")
for loader in /usr/lib/*/gdk-pixbuf-2.0/*/loaders/libpixbufloader-svg.so /usr/lib/gdk-pixbuf-2.0/*/loaders/libpixbufloader-svg.so; do
    if [[ -f $loader ]]; then
        install -m644 "$loader" "$lib/libpixbufloader-svg.so"
        seeds+=("$loader")
        break
    fi
done

host_pattern='^(ld-linux[^/]*|lib(c|m|mvec|dl|rt|pthread|resolv|util|nss_[^/]+|EGL|GLX|GLdispatch|GL|gbm|drm[^/]*|vulkan|udev|systemd))\.so(\.|$)'
for seed in "${seeds[@]}"; do
    dependencies=$(ldd "$seed")
    while read -r name arrow path _; do
        if [[ $arrow != '=>' || $name == /* || $name =~ $host_pattern ]]; then
            continue
        fi
        if [[ $path == not ]]; then
            printf 'Biblioteca ausente: %s\n' "$name" >&2
            exit 1
        fi
        install -m644 "$path" "$lib/$name"
    done <<< "$dependencies"
done

cp -a /usr/share/icons/Adwaita "$root/usr/share/icons/"
for schema in /usr/share/glib-2.0/schemas/org.gtk.gtk4.*.xml; do
    install -m644 "$schema" "$root/usr/share/glib-2.0/schemas/"
done
glib-compile-schemas --strict "$root/usr/share/glib-2.0/schemas"
