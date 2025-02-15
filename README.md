# Pipewire-DBus

This package integrates both with Pipewire and DBus and provides:

1. `Data` DBus property that returns a tuple of volume and muted flag
3. `DataChanged` signal that is triggered with update data tuple

Exact schema is available in `org.local.PipewireDBus.xml` file but you can also retrieve it from locally running daemon:

```sh
$ busctl --user introspect --xml-interface org.local.PipewireDBus /org/local/PipewireDBus org.local.PipewireDBus
```

The service also supports introspection:

```sh
$ busctl --user introspect org.local.PipewireDBus /org/local/PipewireDBus org.local.PipewireDBus
NAME                   TYPE      SIGNATURE RESULT/VALUE FLAGS
.Data                  property  (ub)      34 false     -
.DataChanged           signal    ub        -            -
```

## Getting data

```sh
$ busctl --user get-property org.local.PipewireDBus /org/local/PipewireDBus org.local.PipewireDBus Data
(ub) 34 false
```

## Signals

```sh
$ busctl --user monitor org.local.PipewireDBus
```

... and try changing volume/mute your device. You'll see a stream of events in your terminal.
