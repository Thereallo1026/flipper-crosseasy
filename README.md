# CrossEasy for Flipper Zero

> [!NOTE]
> Learn more in my blog post: [Reverse Engineering Hong Kong's New Traffic Lights](https://thereallo.dev/blog/reverse-engineering-hong-kong-traffic-lights).

Sends the same BLE beacon Hong Kong's *HKeMobility* app uses to turn up the audible signal at accessible crossings (eATS), from a Flipper instead of a phone.

It's an accessibility feature. If you're someone it's for, this is just a handier way to invoke it. Don't spam crossings you don't need.

## The signal

Non connectable manufacturer advertisement, company id 2619 (`0x0A3B`):

```
1B FF 3B 0A 47 61 4C 69 4C 65 4F 20 <16-byte AES block>
```

`1B` length, `FF` manufacturer data, `3B 0A` company id little endian, the ASCII header `"GaLiLeO "`, then a 16 byte `AES-128-ECB` block built on device each broadcast:

```
key = "GalileoLH0000852"
plaintext = "TeCh>Z" + nonce[0..6] + 0x70 + {0x58, 0x79, 0x11}
```

The nonce is per session. The app takes the low 48 bits of `System.currentTimeMillis()` in `AdvertiserService.onCreate`; here it's the Flipper RTC (Unix ms) plus the tick counter.

Both modes send the same bytes. Receivers tell a hold (amplify) from a tap (trigger) by how long the advert stays up, so the mode is just how long it broadcasts.

Goes out through the firmware's extra beacon slot (`flipperzero::bluetooth::beacon`). Stock firmware has it, just new enough to include the API, or Momentum / Xtreme.

## Build and install

```sh
cargo build --release
```

The bundle lands at `target/thumbv7em-none-eabihf/release/crosseasy.fap`. Drop it on the Flipper under `apps/Bluetooth/`, or `ufbt launch APPSRC=…/crosseasy.fap`.

GPL-3.0. Dolphin splash from [flipperzero-firmware](https://github.com/flipperdevices/flipperzero-firmware).
