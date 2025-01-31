# Pipewire-DBus

This package integrates both with Pipewire and DBus so that there are:

1. `GetVolume/GetMuted/SetVolume()/SetMuted()` DBus methods
3. `VolumeUpdated/MutedUpdated` signals

Exact schema is available in `org.local.PipewireDBus.xml` but you can also retrieve it from locally running daemon:

```sh
$ busctl --user introspect --xml-interface org.local.PipewireDBus /org/local/PipewireDBus org.local.PipewireDBus
```

The service also supports introspection:

```sh
$ busctl --user introspect org.local.PipewireDBus /org/local/PipewireDBus org.local.PipewireDBus
NAME                   TYPE      SIGNATURE RESULT/VALUE FLAGS
.GetMuted              method    -         b            -
.GetVolume             method    -         d            -
.SetMuted              method    b         -            -
.SetVolume             method    d         -            -
.MutedUpdated          signal    b         -            -
.VolumeUpdated         signal    d         -            -
```

## Getting data

```sh
$ busctl --user call org.local.PipewireDBus /org/local/PipewireDBus org.local.PipewireDBus GetVolume
d 0.300003
```

or

```sh
$ busctl --user call org.local.PipewireDBus /org/local/PipewireDBus org.local.PipewireDBus GetMuted
b false
```

## Setting data

```sh
$ busctl --user call org.local.PipewireDBus /org/local/PipewireDBus org.local.PipewireDBus SetVolume "d" 0.5
```

```sh
$ busctl --user call org.local.PipewireDBus /org/local/PipewireDBus org.local.PipewireDBus SetMuted "b" false
```

## Signals

```sh
$ busctl --user monitor org.local.PipewireDBus
```

... and try changing volume/mute your device. You'll ses a stream of events in your terminal.

## Running as systemd system service

That's technically possible, you can define a rule somewhere in `/usr/share/dbus-1/system.d/org.local.PipewireDBus.conf` that allows acquiring a DBus name, then make sure that pipewire runs as sudo by modifying `/usr/lib/systemd/user/pipewire.service` and finally write a service that depends on both `dbus.service` and `pipewire.service`.

In practice, it's easier to run it as your user.
