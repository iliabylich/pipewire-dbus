# Pipewire-DBus

This package integrates both with Pipewire and DBus and provides:

1. `Volume` DBus property that returns current volume (as u32)
2. `Muted` DBus property that returns current muted flag (as bool)
3. standard `org.freedesktop.DBus.Properties` interface for receiving notifications when one of these values is changed

Exact schema is available in `org.local.PipewireDBus.xml` file but you can also retrieve it from locally running daemon:

```sh
$ busctl --user introspect --xml-interface org.local.PipewireDBus /org/local/PipewireDBus org.local.PipewireDBus
```

The service also supports introspection:

```sh
$ busctl --user introspect org.local.PipewireDBus /org/local/PipewireDBus org.local.PipewireDBus
NAME                   TYPE      SIGNATURE RESULT/VALUE FLAGS
.Muted                 property  b         false        emits-change
.Volume                property  u         0            emits-change
```

## Getting data

```sh
$ busctl --user get-property org.local.PipewireDBus /org/local/PipewireDBus org.local.PipewireDBus Volume
u 32
```

## Signals

```sh
$ busctl --user monitor org.local.PipewireDBus
```

... and try changing volume/mute your device. You'll see a stream of events in your terminal.
