project('pipewire-dbus')

subdir('data')
subdir('systemd')

custom_target(
    'pipewire-dbus',
    command: [
        files('meson-build-rust.sh'),
        get_option('buildtype'),
        meson.project_source_root(),
        '@OUTPUT@',
    ],
    output: 'pipewire-dbus',
    build_always_stale: true,
    build_by_default: true,
    install: true,
    install_dir: join_paths(
        get_option('prefix'),
        get_option('bindir'),
    ),
)
