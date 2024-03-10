use std::{path::Path, time::SystemTime};

use sysinfo::{self, Networks};


#[derive(Debug, Clone)]
pub struct ProcessData {
    pub pid: u32,
    pub name: String,
    pub exe: String,
    pub state: String,

    pub ram: u64,
    pub virtual_memory: u64,
    pub total_time: f32,
    pub start_time: f32,
    pub cpu_usage: f32,
}

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub uid: String,
    pub groups: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SystemSpec {
    pub os: String,
    pub hostname: String,
    pub kernel: String,
    pub uptime: String,
    pub users: Vec<User>,
}

#[derive(Debug, Clone)]
pub struct  Disk {
    pub name: String,
    pub mount: String,
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub percent: f32,
    pub fs_type: String,
    pub is_removable: bool,
}

#[derive(Debug, Clone)]
pub struct Cpu {
    pub name: String,
    pub usage: f32,
    pub clock_speed: f32,
    pub vendor: String,
}

#[derive(Debug, Clone)]
pub struct  Network {
    pub name: String,
    pub mac: String,
    pub total_sent: u64,
    pub total_recv: u64,
    pub total_packets_sent: u64,
    pub total_packets_recv: u64,
}

#[derive(Debug, Clone)]
pub struct SystemData {
    pub cpus: Vec<Cpu>,
    pub memory: u64,
    pub swap: u64,
    pub disks: Vec<Disk>,
    pub total_memory: u64,
    pub total_swap: u64,
    pub networks: Vec<Network>,
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub usage: SystemData,
    pub processes: Vec<ProcessData>,
    pub spec: SystemSpec,
}

impl SystemInfo {
    pub fn new() -> SystemInfo {
        SystemInfo {
            usage: SystemData {
                cpus: vec![],
                memory: 0,
                swap: 0,
                disks: vec![],
                total_memory: 0,
                total_swap: 0,
                networks: vec![],
            },
            processes: Vec::new(),
            spec: SystemSpec {
                os: String::new(),
                hostname: String::new(),
                kernel: String::new(),
                uptime: String::new(),
                users: vec![],
            },
        }
    }

    pub fn populate(&mut self) {
        let mut sys = sysinfo::System::new_all();

        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

        sys.refresh_all();

        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

        let mut cpus = vec![];

        for cpu in sys.cpus() {
            cpus.push(Cpu {
                name: cpu.name().to_string(),
                usage: cpu.cpu_usage() as f32,
                clock_speed: cpu.frequency() as f32,
                vendor: cpu.vendor_id().to_string(),
            });
        }

        self.usage.cpus = cpus;

        self.usage.memory = sys.used_memory();
        self.usage.swap = sys.used_swap();
        self.usage.total_memory = sys.total_memory();
        self.usage.total_swap = sys.total_swap();

        let mut networks = vec![];

        for (name, network) in Networks::new_with_refreshed_list().iter() {
            networks.push(Network {
                name: name.to_string(),
                mac: network.mac_address().to_string(),
                total_sent: network.total_transmitted(),
                total_recv: network.total_received(),
                total_packets_sent: network.total_packets_transmitted(),
                total_packets_recv: network.total_packets_received(),
            });
        }

        self.usage.networks = networks;

        let mut disks = vec![];

        for disk in &sysinfo::Disks::new_with_refreshed_list() {
            disks.push(Disk {
                name: disk.name().to_str().unwrap_or_default().to_string(),
                mount: disk.mount_point().to_str().unwrap_or_default().to_string(),
                total: disk.total_space(),
                used: disk.total_space() - disk.available_space(),
                free: disk.available_space(),
                percent: disk.total_space() as f32 / disk.available_space() as f32,
                fs_type: disk.file_system().to_str().unwrap_or_default().to_string(),
                is_removable: disk.is_removable(),
            });
        }

        self.usage.disks = disks;

        let mut processes = vec![];

        for (pid, process) in sys.processes() {
            let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as f32;

            let total_time = current_time - process.start_time() as f32;
            processes.push(
                ProcessData {
                    pid: pid.as_u32(),
                    name: process.name().to_string(),
                    exe: process.exe().unwrap_or(Path::new("not_found")).as_os_str().to_str().unwrap_or_default().to_string(),
                    state: process.status().to_string(),
                    ram: process.memory(),
                    virtual_memory: process.virtual_memory(),
                    total_time: total_time,
                    start_time: process.start_time() as f32,
                    // divide by number of cpus to get percentage
                    cpu_usage: process.cpu_usage() as f32 / sys.cpus().iter().count() as f32,
                }
            );
        }

        self.processes = processes;

        self.spec.os = format!("{} {}", sysinfo::System::name().unwrap_or("Unknown".to_string()), sysinfo::System::os_version().unwrap_or_default());
        self.spec.hostname = sysinfo::System::host_name().unwrap_or_default().to_string();
        self.spec.kernel = sysinfo::System::kernel_version().unwrap_or_default().to_string();
        self.spec.uptime = sysinfo::System::uptime().to_string();

        let mut users = vec![];

        for user in sysinfo::Users::new_with_refreshed_list().list() {
            users.push(User {
                name: user.name().to_string(),
                uid: user.id().to_string(),
                groups: user.groups().iter().map(|x| x.name().to_string()).collect(),
            });
        }

        self.spec.users = users;
    }
}