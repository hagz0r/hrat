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
    pub use_tls: bool,
}
impl Connection {
    pub fn from(ip: String, port: i32, use_tls: bool) -> Self {
        Self { ip, port, use_tls }
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

pub fn validate_tls_connection(ip: &str, port: i32) -> bool {
    // Simple validation to check if TLS is appropriate
    // In production, you might want more sophisticated validation
    if ip == "localhost" || ip == "127.0.0.1" {
        // For localhost, TLS might not be necessary in development
        return true;
    }

    // For remote connections, TLS is strongly recommended
    true
}

pub fn get_connection_info(use_tls: bool) -> String {
    if use_tls {
        "Using secure WebSocket connection (WSS) with TLS encryption".to_string()
    } else {
        "WARNING: Using plain WebSocket connection (WS) without encryption".to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ip_addresses() {
        assert!(is_valid_ip("127.0.0.1"));
        assert!(is_valid_ip("192.168.1.1"));
        assert!(is_valid_ip("10.0.0.1"));
        assert!(is_valid_ip("localhost"));
        assert!(!is_valid_ip("256.1.1.1")); // Invalid octet
        assert!(!is_valid_ip("192.168..1")); // Double dots
        assert!(!is_valid_ip("not.an.ip"));
    }

    #[test]
    fn test_valid_ports() {
        assert!(is_port_valid("80"));
        assert!(is_port_valid("443"));
        assert!(is_port_valid("8080"));
        assert!(is_port_valid("65535"));
        assert!(is_port_valid("0"));
        assert!(!is_port_valid("65536")); // Out of range
        assert!(!is_port_valid("-1")); // Negative
        assert!(!is_port_valid("not_a_port")); // Invalid format
    }

    #[test]
    fn test_tls_validation() {
        assert!(validate_tls_connection("127.0.0.1", 8080));
        assert!(validate_tls_connection("localhost", 443));
        assert!(validate_tls_connection("192.168.1.100", 9443));
    }

    #[test]
    fn test_connection_info() {
        let secure_info = get_connection_info(true);
        assert!(secure_info.contains("secure"));
        assert!(secure_info.contains("TLS"));

        let plain_info = get_connection_info(false);
        assert!(plain_info.contains("WARNING"));
        assert!(plain_info.contains("plain"));
    }

    #[test]
    fn test_connection_struct() {
        let conn = Connection::from("127.0.0.1".to_string(), 8080, true);
        assert_eq!(conn.ip, "127.0.0.1");
        assert_eq!(conn.port, 8080);
        assert!(conn.use_tls);

        let conn_plain = Connection::from("localhost".to_string(), 3000, false);
        assert_eq!(conn_plain.ip, "localhost");
        assert_eq!(conn_plain.port, 3000);
        assert!(!conn_plain.use_tls);
    }
}
