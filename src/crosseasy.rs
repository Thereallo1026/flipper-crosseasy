// see readme for packet layout

use aes::Aes128;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockEncrypt, KeyInit};

use flipperzero::bluetooth::beacon::{AdPacket, AdType, Beacon};
use flipperzero_sys as sys;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    SoundBoost,
    Trigger,
}

impl Mode {
    pub fn label(self) -> &'static core::ffi::CStr {
        match self {
            Mode::SoundBoost => c"Amplify sound",
            Mode::Trigger => c"Trigger + sound",
        }
    }

    // short label for the narrow panel next to the dolphin
    pub fn short_label(self) -> &'static core::ffi::CStr {
        match self {
            Mode::SoundBoost => c"Amplify",
            Mode::Trigger => c"Trigger",
        }
    }
}

const COMPANY_ID: u16 = 2619;
const KEY: &[u8; 16] = b"GalileoLH0000852";
const HEADER: [u8; 8] = *b"GaLiLeO ";

// company id (le) + "GaLiLeO " header + aes-128-ecb block
// nonce_ms fills the 6 timestamp bytes the app derives from currentTimeMillis at init
fn manufacturer_data(nonce_ms: u64) -> [u8; 26] {
    let mut plain = [0u8; 16];
    plain[..6].copy_from_slice(b"TeCh>Z");
    let n = nonce_ms.to_le_bytes();
    plain[6..12].copy_from_slice(&n[..6]);
    plain[12] = 0x70;
    plain[13] = 0x58;
    plain[14] = 0x79;
    plain[15] = 0x11;

    let mut block = GenericArray::clone_from_slice(&plain);
    Aes128::new(GenericArray::from_slice(KEY)).encrypt_block(&mut block);

    let mut out = [0u8; 26];
    out[..2].copy_from_slice(&COMPANY_ID.to_le_bytes());
    out[2..10].copy_from_slice(&HEADER);
    out[10..26].copy_from_slice(block.as_slice());
    out
}

// per-session nonce: unix ms from the rtc mixed with the tick counter so it
// varies between launches even if the clock is coarse
fn session_nonce() -> u64 {
    let secs = unsafe { sys::furi_hal_rtc_get_timestamp() } as u64;
    let ticks = unsafe { sys::furi_get_tick() } as u64;
    secs.wrapping_mul(1000).wrapping_add(ticks % 1000)
}

pub struct CrossEasyBeacon {
    inner: Beacon,
}

impl CrossEasyBeacon {
    // none if bluetooth is unavailable or the beacon slot is busy
    pub fn acquire() -> Option<Self> {
        Some(Self {
            inner: Beacon::acquire().ok()?,
        })
    }

    // rebuild the packet with a fresh nonce and start advertising
    pub fn start(&mut self) -> bool {
        let data = manufacturer_data(session_nonce());
        let Ok(packet) = AdPacket::empty().with_var(AdType::ManufacturerSpecificData, &data) else {
            return false;
        };
        self.inner.set_data_packet(packet).is_ok() && self.inner.start().is_ok()
    }

    pub fn stop(&mut self) {
        let _ = self.inner.stop();
    }
}

impl Drop for CrossEasyBeacon {
    fn drop(&mut self) {
        self.stop();
    }
}
