use crate::system::reading::Reading;
use std::ffi::c_void;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows_sys::Win32::System::IO::DeviceIoControl;
use windows_sys::Win32::System::Ioctl::{
    IOCTL_STORAGE_QUERY_PROPERTY, PropertyStandardQuery, STORAGE_DEVICE_DESCRIPTOR,
    STORAGE_PROPERTY_ID, STORAGE_PROPERTY_QUERY, STORAGE_TEMPERATURE_DATA_DESCRIPTOR,
    STORAGE_TEMPERATURE_INFO, StorageDeviceProperty, StorageDeviceTemperatureProperty,
};

const MAX_DRIVES: u32 = 32;
const BUFFER_BYTES: usize = 1024;
const SENSORS_OFFSET: usize = 24;

pub fn temperatures() -> Vec<Reading> {
    (0..MAX_DRIVES)
        .filter_map(Drive::open)
        .flat_map(|drive| drive.temperatures())
        .collect()
}

struct Drive {
    handle: HANDLE,
    index: u32,
}

impl Drive {
    fn open(index: u32) -> Option<Self> {
        let path = wide(&format!(r"\\.\PhysicalDrive{index}"));

        let handle = unsafe {
            CreateFileW(
                path.as_ptr(),
                0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut(),
            )
        };

        (handle != INVALID_HANDLE_VALUE).then_some(Self { handle, index })
    }

    fn temperatures(&self) -> Vec<Reading> {
        let mut buffer = Buffer::new();

        if !self.query(StorageDeviceTemperatureProperty, &mut buffer) {
            return Vec::new();
        }

        let bytes = buffer.bytes();
        let descriptor = unsafe { &*bytes.as_ptr().cast::<STORAGE_TEMPERATURE_DATA_DESCRIPTOR>() };

        let room = (BUFFER_BYTES - SENSORS_OFFSET) / size_of::<STORAGE_TEMPERATURE_INFO>();
        let count = (descriptor.InfoCount as usize).min(room);
        if count == 0 {
            return Vec::new();
        }

        let sensors =
            unsafe { std::slice::from_raw_parts(descriptor.TemperatureInfo.as_ptr(), count) };
        let name = self.name();

        sensors
            .iter()
            .filter(|sensor| sensor.Temperature != i16::MIN)
            .map(|sensor| {
                let label = if count == 1 {
                    name.clone()
                } else {
                    format!("{name} temp{}", sensor.Index)
                };

                Reading::new(label, f32::from(sensor.Temperature))
            })
            .collect()
    }

    fn name(&self) -> String {
        let fallback = format!("disk{}", self.index);
        let mut buffer = Buffer::new();

        if !self.query(StorageDeviceProperty, &mut buffer) {
            return fallback;
        }

        let bytes = buffer.bytes();
        let descriptor = unsafe { &*bytes.as_ptr().cast::<STORAGE_DEVICE_DESCRIPTOR>() };

        let model = [descriptor.VendorIdOffset, descriptor.ProductIdOffset]
            .into_iter()
            .filter_map(|offset| text_at(bytes, offset))
            .collect::<Vec<_>>()
            .join(" ");

        if model.is_empty() {
            fallback
        } else {
            format!("disk{} {}", self.index, model)
        }
    }

    fn query(&self, property: STORAGE_PROPERTY_ID, buffer: &mut Buffer) -> bool {
        let query = STORAGE_PROPERTY_QUERY {
            PropertyId: property,
            QueryType: PropertyStandardQuery,
            AdditionalParameters: [0],
        };

        let mut returned = 0;

        unsafe {
            DeviceIoControl(
                self.handle,
                IOCTL_STORAGE_QUERY_PROPERTY,
                (&raw const query).cast(),
                size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                buffer.as_mut_ptr(),
                BUFFER_BYTES as u32,
                &mut returned,
                std::ptr::null_mut(),
            ) != 0
        }
    }
}

impl Drop for Drive {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle) };
    }
}

struct Buffer([u64; BUFFER_BYTES / 8]);

impl Buffer {
    fn new() -> Self {
        Self([0; BUFFER_BYTES / 8])
    }

    fn as_mut_ptr(&mut self) -> *mut c_void {
        self.0.as_mut_ptr().cast()
    }

    fn bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.0.as_ptr().cast(), BUFFER_BYTES) }
    }
}

fn text_at(bytes: &[u8], offset: u32) -> Option<String> {
    let start = offset as usize;
    if start == 0 || start >= bytes.len() {
        return None;
    }

    let end = start + bytes[start..].iter().position(|byte| *byte == 0)?;
    let text = std::str::from_utf8(&bytes[start..end]).ok()?.trim();

    (!text.is_empty()).then(|| text.to_string())
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
