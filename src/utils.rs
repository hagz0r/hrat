use sysinfo::System;

pub fn is_valid_ip(ip: &str) -> bool {
	if ip == "localhost" { return true; }
	ip.split('.').filter_map(|s| s.parse::<u8>().ok()).count() == 4 && !ip.contains("..")
}


pub fn is_port_valid(port: &str) -> bool {
	(0..=65535).contains(&port.parse::<i32>().unwrap())
}


pub struct SystemInformation {
	host_name: String,
	os_version: String,
	long_os_version: String,
	kernel_version: String,
	cpu_model: String,
	memory_total: u64,
}

impl SystemInformation {
	pub fn get() -> SystemInformation {
		let mut sys = System::new_all();
		sys.refresh_all();

		let cpu_model = match sys.global_cpu_info().brand().to_string().as_str() {
			"" => "Unknown".to_string(),
			s => s.to_string(),
		};

		SystemInformation {
			host_name: System::host_name().unwrap_or("Unknown".into()),
			os_version: System::os_version().unwrap_or("Unknown".into()),
			long_os_version: System::long_os_version().unwrap_or("Unknown".into()),
			kernel_version: System::kernel_version().unwrap_or("Unknown".into()),
			cpu_model,
			memory_total: sys.total_memory(),
		}
	}

	pub fn to_string(&self) -> String {
		format!(
			"{},{},{},{},{},{}",
			self.host_name,
			self.os_version,
			self.long_os_version,
			self.kernel_version,
			self.cpu_model,
			self.memory_total,
		)
	}
}