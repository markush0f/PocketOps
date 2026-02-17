use crate::executor::ssh::SshExecutor;
use crate::models::ManagedServer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_release: String,
    pub kernel_version: String,
    pub hostname: String,
    pub uptime: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resources {
    pub cpu_usage: String, // simple load avg
    pub memory_usage: String,
    pub disk_usage: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunningService {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryReport {
    pub system_info: SystemInfo,
    pub resources: Resources,
    pub services: Vec<RunningService>,
    pub timestamp: String,
}

pub struct Discovery;

impl Discovery {
    pub fn run(server: &ManagedServer) -> Result<DiscoveryReport, String> {
        // Gather System Info
        let os_release = SshExecutor::execute(
            server,
            "cat /etc/os-release | grep PRETTY_NAME | cut -d= -f2 | tr -d '\"'",
        )
        .unwrap_or_else(|_| "Unknown".to_string());

        let kernel =
            SshExecutor::execute(server, "uname -r").unwrap_or_else(|_| "Unknown".to_string());

        let hostname =
            SshExecutor::execute(server, "hostname").unwrap_or_else(|_| "Unknown".to_string());

        let uptime =
            SshExecutor::execute(server, "uptime -p").unwrap_or_else(|_| "Unknown".to_string());

        let system_info = SystemInfo {
            os_release: os_release.trim().to_string(),
            kernel_version: kernel.trim().to_string(),
            hostname: hostname.trim().to_string(),
            uptime: uptime.trim().to_string(),
        };

        // Gather Resources
        let load_avg = SshExecutor::execute(server, "cat /proc/loadavg | awk '{print $1, $2, $3}'")
            .unwrap_or_else(|_| "Unknown".to_string());

        let memory =
            SshExecutor::execute(server, "free -h | grep Mem | awk '{print $3 \" / \" $2}'")
                .unwrap_or_else(|_| "Unknown".to_string());

        let disk = SshExecutor::execute(
            server,
            "df -h / | tail -n 1 | awk '{print $3 \" / \" $2 \" (\" $5 \")\"}'",
        )
        .unwrap_or_else(|_| "Unknown".to_string());

        let resources = Resources {
            cpu_usage: format!("Load Avg: {}", load_avg.trim()),
            memory_usage: memory.trim().to_string(),
            disk_usage: disk.trim().to_string(),
        };

        // Gather Services (Top 10 running)
        // using systemctl list-units --type=service --state=running
        let services_raw = SshExecutor::execute(server, "systemctl list-units --type=service --state=running --no-pager --plain | head -n 15 | awk '{print $1}'")
             .unwrap_or_else(|_|"".to_string());

        let services: Vec<RunningService> = services_raw
            .lines()
            .filter(|line| line.contains(".service"))
            .map(|line| RunningService {
                name: line.trim().to_string(),
                status: "running".to_string(),
            })
            .collect();

        Ok(DiscoveryReport {
            system_info,
            resources,
            services,
            timestamp: chrono::Local::now().to_string(),
        })
    }
}
