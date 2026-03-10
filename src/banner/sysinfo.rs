use std::env;

pub struct SystemInfo {
    pub hostname: String,
    pub os: &'static str,
    pub arch: &'static str,
    pub date: String,
    pub load: String,
    pub memory: String,
}

impl SystemInfo {
    pub fn gather() -> Self {
        Self {
            hostname: get_hostname(),
            os: env::consts::OS,
            arch: env::consts::ARCH,
            date: get_date(),
            load: get_load(),
            memory: get_memory(),
        }
    }
}

fn get_hostname() -> String {
    let mut buf = [0u8; 256];
    let ret = unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len()) };
    if ret == 0 {
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        String::from_utf8_lossy(&buf[..len])
            .split('.')
            .next()
            .unwrap_or("unknown")
            .to_string()
    } else {
        "unknown".to_string()
    }
}

fn get_date() -> String {
    let mut t: libc::time_t = 0;
    unsafe { libc::time(&raw mut t) };
    let tm = unsafe { libc::localtime(&raw const t) };
    if tm.is_null() {
        return "????-??-??".to_string();
    }
    let tm = unsafe { &*tm };
    format!(
        "{:04}-{:02}-{:02}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
    )
}

#[cfg(target_os = "macos")]
fn get_load() -> String {
    let mut buf = [0u8; 256];
    let mut len: libc::size_t = buf.len();
    let name = c"vm.loadavg";
    let ret = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            buf.as_mut_ptr().cast(),
            &raw mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret == 0 && len >= 4 {
        // macOS struct loadavg { u32 ldavg[3]; long fscale; }
        let scale_offset = 3 * std::mem::size_of::<u32>();
        if len > scale_offset {
            let ldavg: [u32; 3] = [
                u32::from_ne_bytes(buf[0..4].try_into().unwrap_or([0; 4])),
                u32::from_ne_bytes(buf[4..8].try_into().unwrap_or([0; 4])),
                u32::from_ne_bytes(buf[8..12].try_into().unwrap_or([0; 4])),
            ];
            let fscale_bytes = &buf[scale_offset..scale_offset + std::mem::size_of::<i64>()];
            #[allow(clippy::cast_precision_loss)]
            let fscale =
                i64::from_ne_bytes(fscale_bytes.try_into().unwrap_or([0; 8])).max(1) as f64;
            return format!("{:.2}", f64::from(ldavg[0]) / fscale);
        }
    }
    "?".to_string()
}

#[cfg(target_os = "macos")]
fn get_memory() -> String {
    let mut memsize: u64 = 0;
    let mut len: libc::size_t = std::mem::size_of::<u64>();
    let name = c"hw.memsize";
    let ret = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            (&raw mut memsize).cast(),
            &raw mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret == 0 {
        format!("{}GB", memsize / 1_073_741_824)
    } else {
        "?GB".to_string()
    }
}

#[cfg(target_os = "linux")]
fn get_load() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next().map(String::from))
        .unwrap_or_else(|| "?".to_string())
}

#[cfg(target_os = "linux")]
fn get_memory() -> String {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("MemTotal:"))
                .and_then(|l| {
                    l.split_whitespace()
                        .nth(1)
                        .and_then(|v| v.parse::<u64>().ok())
                })
                .map(|kb| format!("{}GB", kb / 1_048_576))
        })
        .unwrap_or_else(|| "?GB".to_string())
}
