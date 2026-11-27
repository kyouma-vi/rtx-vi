//! Experimental 60 FPS target for one verified title build.
//! Only short signatures and per-build RVAs are kept here (asset policy).

use crate::prepare::PrepareError;
use victory_pe::ImageInfo;

const BUILD: &str = "b99f876b74d4a77f03af5f878223b27bd647b5c35a79d5decd9f716504239e93";
const FPS: u32 = 60;
const LEN: usize = 6;

// Both paths load the configured FPS before converting it to a period in
// microseconds. Keep the original clamp, automatic refresh divisor, and
// time-based pacing calculations. The paths cover initial setup and a
// later configuration update. Do not change shared float constants.
const SITES: [(u32, [u8; LEN]); 2] = [
    (0x224359a, [0x8b, 0x86, 0x10, 0x02, 0x00, 0x00]),
    (0x1e0fc62, [0x8b, 0x87, 0x10, 0x02, 0x00, 0x00]),
];

fn replacement() -> [u8; LEN] {
    // mov eax, FPS; nop. Same length, register and flag behaviour as the
    // original load, with no new branches, stack use or relocations.
    let mut code = [0xb8, 0, 0, 0, 0, 0x90];
    code[1..5].copy_from_slice(&FPS.to_le_bytes());
    code
}

pub(crate) fn apply(
    bytes: &mut [u8],
    info: &ImageInfo,
    hash: &blake3::Hash,
) -> Result<(), PrepareError> {
    if hash.to_hex().as_str() != BUILD || info.is_dll() {
        return Err(PrepareError::Patch("60 FPS: unsupported executable build; omit --60fps"));
    }
    let mut sites = [(0, [0; LEN]); 2];
    for (slot, (rva, expect)) in sites.iter_mut().zip(SITES) {
        let at = info.rva_to_file_offset(rva)
            .ok_or(PrepareError::Patch("60 FPS: patch site is not file-backed"))? as usize;
        let end = info.rva_to_file_offset(rva + LEN as u32 - 1);
        if end.map(|n| n as usize) != at.checked_add(LEN - 1) {
            return Err(PrepareError::Patch("60 FPS: patch site crosses a section boundary"));
        }
        *slot = (at, expect);
    }
    apply_checked(bytes, &sites)
}

fn apply_checked(bytes: &mut [u8], sites: &[(usize, [u8; LEN]); 2]) -> Result<(), PrepareError> {
    // Validate both before writing either. No partially patched image.
    for (at, expect) in sites {
        let end = at.checked_add(LEN)
            .ok_or(PrepareError::Patch("60 FPS: patch offset overflow"))?;
        if bytes.get(*at..end) != Some(expect.as_slice()) {
            return Err(PrepareError::Patch("60 FPS: unexpected instruction bytes; no patch applied"));
        }
    }
    let new = replacement();
    for (at, _) in sites {
        bytes[*at..*at + LEN].copy_from_slice(&new);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Vec<u8>, [(usize, [u8; LEN]); 2]) {
        let sites = [(4, SITES[0].1), (20, SITES[1].1)];
        let mut bytes = vec![0xcc; 32];
        for (at, expect) in sites {
            bytes[at..at + LEN].copy_from_slice(&expect);
        }
        (bytes, sites)
    }

    #[test]
    fn both_paths_use_sixty_and_other_bytes_are_unchanged() {
        let (mut bytes, sites) = fixture();
        let mut expected = bytes.clone();
        let new = [0xb8, 60, 0, 0, 0, 0x90];
        for (at, _) in sites {
            expected[at..at + LEN].copy_from_slice(&new);
        }
        apply_checked(&mut bytes, &sites).unwrap();
        assert_eq!(bytes, expected);
    }

    #[test]
    fn mismatch_or_out_of_bounds_never_partially_patches() {
        let (mut bytes, mut sites) = fixture();
        bytes[20] ^= 1;
        let before = bytes.clone();
        assert!(apply_checked(&mut bytes, &sites).is_err());
        assert_eq!(bytes, before);
        sites[1].0 = usize::MAX;
        assert!(apply_checked(&mut bytes, &sites).is_err());
        assert_eq!(bytes, before);
    }

    #[test]
    fn supplied_build_optional() {
        let Some(path) = std::env::var_os("VICTORY_TEST_60FPS_EXE") else { return };
        let original = std::fs::read(path).unwrap();
        let mut prepared = crate::prepare(&original).unwrap();
        let normal_name = prepared.file_name();
        let mut expected = prepared.bytes.clone();
        for (rva, _) in SITES {
            let at = prepared.info.rva_to_file_offset(rva).unwrap() as usize;
            expected[at..at + LEN].copy_from_slice(&replacement());
        }
        prepared.enable_60fps().unwrap();
        assert_eq!(prepared.bytes, expected);
        assert_ne!(prepared.file_name(), normal_name);
        assert_eq!(prepared.original_hash, blake3::hash(&original));
        prepared.enable_60fps().unwrap();
        assert_eq!(prepared.bytes, expected);
    }
}
