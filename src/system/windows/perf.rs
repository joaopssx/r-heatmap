use crate::system::reading::Reading;
use crate::system::windows::adapters;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;
use windows_sys::Win32::System::Performance::{
    PDH_CSTATUS_VALID_DATA, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY,
    PDH_MORE_DATA, PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData,
    PdhGetFormattedCounterArrayW, PdhOpenQueryW,
};
use windows_sys::core::PWSTR;

const COUNTER: &str = r"\GPU Engine(*)\Utilization Percentage";
const WARMUP: Duration = Duration::from_millis(120);

thread_local! {
    static QUERY: RefCell<Option<Query>> = const { RefCell::new(None) };
}

pub fn gpu_usage() -> Vec<Reading> {
    QUERY.with(|query| {
        let mut query = query.borrow_mut();

        if query.is_none() {
            *query = Query::open();
        }

        match query.as_ref() {
            Some(query) => usage(&query.samples(), &query.names),
            None => Vec::new(),
        }
    })
}

fn usage(samples: &[(String, f64)], names: &HashMap<String, String>) -> Vec<Reading> {
    let mut adapters: BTreeMap<String, BTreeMap<String, f64>> = BTreeMap::new();

    for (instance, value) in samples {
        let Some((adapter, engine)) = split_instance(instance) else {
            continue;
        };

        let adapter = adapter.to_ascii_lowercase();

        if !names.is_empty() && !names.contains_key(&adapter) {
            continue;
        }

        *adapters
            .entry(adapter)
            .or_default()
            .entry(engine.to_ascii_lowercase())
            .or_default() += value;
    }

    adapters
        .into_iter()
        .enumerate()
        .map(|(index, (adapter, engines))| {
            let usage = engines.into_values().fold(0.0, f64::max);

            let label = match names.get(&adapter) {
                Some(name) => format!("GPU {index} {name}"),
                None => format!("GPU {index}"),
            };

            Reading::new(label, usage as f32)
        })
        .collect()
}

fn split_instance(instance: &str) -> Option<(&str, &str)> {
    let adapter = instance.find("luid_")?;
    let end = instance.find("_phys_").or_else(|| instance.find("_eng_"))?;
    let engine = instance.find("_engtype_")?;

    if end <= adapter {
        return None;
    }

    Some((
        &instance[adapter..end],
        &instance[engine + "_engtype_".len()..],
    ))
}

struct Query {
    handle: PDH_HQUERY,
    counter: PDH_HCOUNTER,
    names: HashMap<String, String>,
}

impl Query {
    fn open() -> Option<Self> {
        let mut handle: PDH_HQUERY = std::ptr::null_mut();

        if unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut handle) } != 0 {
            log::warn!("Could not open a performance query for {}", COUNTER);
            return None;
        }

        let path = wide(COUNTER);
        let mut counter: PDH_HCOUNTER = std::ptr::null_mut();

        if unsafe { PdhAddEnglishCounterW(handle, path.as_ptr(), 0, &mut counter) } != 0 {
            log::warn!("Counter {} is not available", COUNTER);
            unsafe { PdhCloseQuery(handle) };
            return None;
        }

        unsafe { PdhCollectQueryData(handle) };
        std::thread::sleep(WARMUP);

        Some(Self {
            handle,
            counter,
            names: adapters::names(),
        })
    }

    fn samples(&self) -> Vec<(String, f64)> {
        let status = unsafe { PdhCollectQueryData(self.handle) };

        if status != 0 {
            log::debug!("Collecting {} failed with 0x{:08x}", COUNTER, status);
            return Vec::new();
        }

        let mut size = 0;
        let mut count = 0;

        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &mut size,
                &mut count,
                std::ptr::null_mut(),
            )
        };

        if status != PDH_MORE_DATA || size == 0 {
            log::debug!("Sizing {} failed with 0x{:08x}", COUNTER, status);
            return Vec::new();
        }

        let mut buffer: Vec<u64> = vec![0; (size as usize).div_ceil(8)];
        let items = buffer.as_mut_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>();

        let status = unsafe {
            PdhGetFormattedCounterArrayW(self.counter, PDH_FMT_DOUBLE, &mut size, &mut count, items)
        };

        if status != 0 {
            log::debug!("Reading {} failed with 0x{:08x}", COUNTER, status);
            return Vec::new();
        }

        let items = unsafe { std::slice::from_raw_parts(items, count as usize) };

        items
            .iter()
            .filter(|item| item.FmtValue.CStatus == PDH_CSTATUS_VALID_DATA)
            .filter_map(|item| {
                let name = read_wide(item.szName)?;
                Some((name, unsafe { item.FmtValue.Anonymous.doubleValue }))
            })
            .collect()
    }
}

impl Drop for Query {
    fn drop(&mut self) {
        unsafe { PdhCloseQuery(self.handle) };
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn read_wide(text: PWSTR) -> Option<String> {
    if text.is_null() {
        return None;
    }

    let mut len = 0;
    while unsafe { *text.add(len) } != 0 {
        len += 1;
    }

    String::from_utf16(unsafe { std::slice::from_raw_parts(text, len) }).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_adapter_and_engine_out_of_an_instance() {
        let instance = "pid_16132_luid_0x00000000_0x0000ec9f_phys_0_eng_4_engtype_copy";

        assert_eq!(
            split_instance(instance),
            Some(("luid_0x00000000_0x0000ec9f", "copy"))
        );
    }

    #[test]
    fn ignores_instances_without_an_engine() {
        assert_eq!(split_instance("pid_16132_luid_0x0_phys_0"), None);
    }

    #[test]
    fn sums_processes_per_engine_and_keeps_the_busiest_one() {
        let samples = [
            sample(
                "pid_1_luid_0x00000000_0x0000EC9F_phys_0_eng_0_engtype_3D",
                20.0,
            ),
            sample(
                "pid_2_luid_0x00000000_0x0000EC9F_phys_0_eng_0_engtype_3D",
                30.0,
            ),
            sample(
                "pid_1_luid_0x00000000_0x0000EC9F_phys_0_eng_4_engtype_Copy",
                5.0,
            ),
            sample(
                "pid_1_luid_0x00000000_0x0000FF2D_phys_0_eng_0_engtype_3D",
                99.0,
            ),
        ];

        let mut names = HashMap::new();
        names.insert(
            "luid_0x00000000_0x0000ec9f".to_string(),
            "RTX 4060 Ti".to_string(),
        );

        let gpus = usage(&samples, &names);

        assert_eq!(gpus.len(), 1);
        assert_eq!(gpus[0].label, "GPU 0 RTX 4060 Ti");
        assert_eq!(gpus[0].value, 50.0);
    }

    fn sample(instance: &str, value: f64) -> (String, f64) {
        (instance.to_string(), value)
    }
}
