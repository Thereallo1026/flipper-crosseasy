# CrossEasy for Flipper Zero

> [!NOTE]
> This app is intended for people with disabilities to invoke the accessible-crossing feature from a Flipper instead of the phone app. The reverse-engineering findings are for educational and interoperability purposes only. Please do not use it to interfere with traffic infrastructure or trigger crossings you do not need.

Broadcasts BLE advertisement used by Hong Kong's *HKeMobility* app to activate accessible pedestrian-crossing receivers (the audible / eATS signals).

Reproduces the beacon emitted by the app's embedded `com.galileo.crosseasylibrary.AdvertiserService`. Built from [`flipperzero-template`](https://github.com/flipperzero-rs/flipperzero-template) against [`flipperzero-rs`](https://github.com/flipperzero-rs/flipperzero-rs).

## BLE signal

Non-connectable manufacturer AD, company id 2619 (`0x0A3B`):

```
1B FF 3B 0A 47 61 4C 69 4C 65 4F 20 <16-byte AES block>
```

`1B` length, `FF` manufacturer data, `3B 0A` company id LE, then the ASCII header `"GaLiLeO "` and a 16-byte `AES-128-ECB` block. The block is encrypted on-device whenever a broadcast starts:

```
key = "GalileoLH0000852"
plaintext = "TeCh>Z" + nonce[0..6] + 0x70 + {0x58, 0x79, 0x11}
```

Nonce is per session. The app takes the low 48 bits of `System.currentTimeMillis()` in `AdvertiserService.onCreate` and writes it to `register.data` as `Verified;galileo;<ts>` — it's a registration nonce, not a constant. Here it comes from the Flipper RTC (Unix ms) mixed with the tick counter. `crosseasy_packet.py` can rebuild the block for any nonce; nonce `0` encrypts to `1E F8 BC B7 …`.

Both modes send the same bytes (`f3827d` / `f3828e`). The phone app's `f3820t` field is what actually changes: `0` on, `1` off, `2` trigger. Receivers tell a long hold (amplify) from a short burst (trigger) by how long the advert stays up, so the mode in this app is just a label for that.

Sent through the firmware extra-beacon slot (`flipperzero::bluetooth::beacon`). Needs OFW with extra-beacon, or Momentum / Xtreme.

## Build and install

```sh
cargo build --release
```

`rustup` installs the pinned toolchain and target automatically.

The bundle is written to `target/thumbv7em-none-eabihf/release/crosseasy.fap`, copy it to the Flipper under `apps/Bluetooth/` (qFlipper) or launch it with `ufbt launch APPSRC=…/crosseasy.fap`.