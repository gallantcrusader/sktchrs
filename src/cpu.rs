use std::io;
use std::str;

use libc::mach_host_self;

use mach::kern_return::KERN_SUCCESS;
use mach::host_info::{
    host_cpu_load_info_data_t,
    host_info_t,
    HOST_CPU_LOAD_INFO,
    HOST_CPU_LOAD_INFO_COUNT
};
use mach::machine::{
    CPU_STATE_MAX,
    CPU_STATE_IDLE,
    CPU_STATE_USER,
    CPU_STATE_SYSTEM
};
use mach::message::mach_msg_type_number_t;

static TOPPROC: &str = "/bin/ps -Aceo pid,pcpu,comm -r";
static FILTER_PATTERN: &str = "com.apple.";

#[derive()]
pub struct CPU {
    pub host: mach::mach_types::host_t,
    pub count: mach_msg_type_number_t,
    pub load: host_cpu_load_info_data_t,
    pub prev_load: host_cpu_load_info_data_t,
    pub has_prev_load: bool,
    pub command: String,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            host: unsafe{ mach_host_self()},
            count: HOST_CPU_LOAD_INFO_COUNT,
            load: host_cpu_load_info_data_t {
                cpu_ticks: [0; CPU_STATE_MAX as usize],
            },
            prev_load: host_cpu_load_info_data_t {
                cpu_ticks: [0; CPU_STATE_MAX as usize],
            },
            has_prev_load: false,
            command: String::new(),
        }
    }

    pub fn update(&mut self) {
        let error = unsafe {
            libc::host_statistics(
                self.host,
                HOST_CPU_LOAD_INFO as i32,
                &self.load as *const _ as host_info_t,
                &self.count as *const _ as *mut mach_msg_type_number_t,
            )
        };

        if error != KERN_SUCCESS {
            println!("Error: Could not read CPU host statistics.");
            return;
        }

        if self.has_prev_load {
            let delta_user = self.load.cpu_ticks[CPU_STATE_USER as usize]
                - self.prev_load.cpu_ticks[CPU_STATE_USER as usize];

            let delta_system = self.load.cpu_ticks[CPU_STATE_SYSTEM as usize]
                - self.prev_load.cpu_ticks[CPU_STATE_SYSTEM as usize];

            let delta_idle = self.load.cpu_ticks[CPU_STATE_IDLE as usize]
                - self.prev_load.cpu_ticks[CPU_STATE_IDLE as usize];

            let user_perc = delta_user as f64 / (delta_system as f64 + delta_user as f64 + delta_idle as f64);
            let sys_perc = delta_system as f64 / (delta_system as f64 + delta_user as f64 + delta_idle as f64);
            let total_perc = user_perc + sys_perc;

            let top_proc = match get_top_process(TOPPROC, FILTER_PATTERN) {
                Ok(process) => process,
                Err(_) => String::new(),
            };

            let color = if total_perc >= 0.7 {
                getenv("RED")
            } else if total_perc >= 0.3 {
                getenv("ORANGE")
            } else if total_perc >= 0.1 {
                getenv("YELLOW")
            } else {
                getenv("LABEL_COLOR")
            };

            self.command = format!(
                "--push cpu.sys {:.2} --push cpu.user {:.2} --set cpu.percent label={:.0}% label.color={} --set cpu.top label=\"{}\"",
                sys_perc,
                user_perc,
                total_perc * 100.0,
                color,
                top_proc
            );
        } else {
            self.command = String::new();
        }

        self.prev_load = self.load;
        self.has_prev_load = true;
    }
}

fn get_top_process(command: &str, filter_pattern: &str) -> Result<String, io::Error> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    let top_process = output_str.lines().nth(2).unwrap_or("");
    let top_process = top_process
        .splitn(2, filter_pattern)
        .last()
        .unwrap_or(top_process);

    Ok(top_process.to_string())
}

fn getenv(env_var: &str) -> String {
    match std::env::var(env_var) {
        Ok(val) => val,
        Err(_) => String::new(),
    }
}

