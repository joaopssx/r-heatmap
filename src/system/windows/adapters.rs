use std::collections::HashMap;
use std::ffi::c_void;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_ROUTINE_FLAGS, RRF_RT_REG_DWORD, RRF_RT_REG_QWORD,
    RRF_RT_REG_SZ, RegCloseKey, RegEnumKeyExW, RegGetValueW, RegOpenKeyExW,
};

const ADAPTERS: &str = r"SOFTWARE\Microsoft\DirectX";
const SOFTWARE_DEVICE: u32 = 0x4;

pub fn names() -> HashMap<String, String> {
    let Some(root) = Key::open(ADAPTERS) else {
        log::warn!("Could not read HKLM\\{}, GPUs will not be named", ADAPTERS);
        return HashMap::new();
    };

    let names: HashMap<String, String> = root
        .children()
        .iter()
        .filter_map(|adapter| {
            let luid = root.qword(adapter, "AdapterLuid")?;
            let description = root.text(adapter, "Description")?;
            let kind = root.dword(adapter, "AdapterType").unwrap_or(0);

            (kind & SOFTWARE_DEVICE == 0).then(|| (luid_key(luid), description))
        })
        .collect();

    for (luid, name) in &names {
        log::debug!("adapter {} is {}", luid, name);
    }

    names
}

fn luid_key(luid: u64) -> String {
    format!("luid_0x{:08x}_0x{:08x}", (luid >> 32) as u32, luid as u32)
}

struct Key(HKEY);

impl Key {
    fn open(path: &str) -> Option<Self> {
        let path = wide(path);
        let mut key: HKEY = std::ptr::null_mut();

        let status =
            unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, path.as_ptr(), 0, KEY_READ, &raw mut key) };

        (status == ERROR_SUCCESS).then_some(Self(key))
    }

    fn children(&self) -> Vec<String> {
        let mut children = Vec::new();
        let mut index = 0;

        loop {
            let mut name = [0u16; 256];
            let mut len = name.len() as u32;

            let status = unsafe {
                RegEnumKeyExW(
                    self.0,
                    index,
                    name.as_mut_ptr(),
                    &raw mut len,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            };

            if status != ERROR_SUCCESS {
                return children;
            }

            children.push(String::from_utf16_lossy(&name[..len as usize]));
            index += 1;
        }
    }

    fn value(
        &self,
        subkey: &str,
        name: &str,
        flags: REG_ROUTINE_FLAGS,
        buffer: &mut [u8],
    ) -> Option<usize> {
        let subkey = wide(subkey);
        let name = wide(name);
        let mut size = buffer.len() as u32;

        let status = unsafe {
            RegGetValueW(
                self.0,
                subkey.as_ptr(),
                name.as_ptr(),
                flags,
                std::ptr::null_mut(),
                buffer.as_mut_ptr().cast::<c_void>(),
                &raw mut size,
            )
        };

        (status == ERROR_SUCCESS).then_some(size as usize)
    }

    fn qword(&self, subkey: &str, name: &str) -> Option<u64> {
        let mut buffer = [0u8; 8];
        self.value(subkey, name, RRF_RT_REG_QWORD, &mut buffer)?;

        Some(u64::from_le_bytes(buffer))
    }

    fn dword(&self, subkey: &str, name: &str) -> Option<u32> {
        let mut buffer = [0u8; 4];
        self.value(subkey, name, RRF_RT_REG_DWORD, &mut buffer)?;

        Some(u32::from_le_bytes(buffer))
    }

    fn text(&self, subkey: &str, name: &str) -> Option<String> {
        let mut buffer = [0u8; 512];
        let size = self.value(subkey, name, RRF_RT_REG_SZ, &mut buffer)?;

        let text: Vec<u16> = buffer[..size]
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();

        let text = String::from_utf16_lossy(&text);
        let text = text.trim_end_matches('\0').trim();

        (!text.is_empty()).then(|| text.to_string())
    }
}

impl Drop for Key {
    fn drop(&mut self) {
        unsafe { RegCloseKey(self.0) };
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_a_luid_the_way_the_counters_do() {
        assert_eq!(luid_key(60575), "luid_0x00000000_0x0000ec9f");
    }
}
