setup build:
    meson setup builddir --buildtype={{build}}

compile:
    meson compile -C builddir

clean:
    rm -rf builddir

dev:
    @just compile
    RUST_LOG=info ./builddir/pipewire-dbus

test-install:
    @just clean
    rm -rf test-install
    meson setup builddir --buildtype=release --prefix=$PWD/test-install/usr
    meson compile -C builddir
    meson install -C builddir
    tree -C test-install
