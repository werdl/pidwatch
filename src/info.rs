use std::{path::Path, time::SystemTime};

use sysinfo;


pub struct ProcessData {
    pid: u32,
    name: String,
    exe: String,
    state: String,

    ram: u64,
    virtual_memory: u64,
    total_time: f32,
    start_time: f32,
    cpu_usage: f32,
}

pub struct User {
    name: String,
    uid: f32,
    gid: f32,
    groups: Vec<String>,
}

pub struct SystemSpec {
    os: String,
    hostname: String,
    kernel: String,
    uptime: String,
    users: Vec<User>,
}

pub struct Disk {
    name: String,
    mount: String,
    total: u64,
    used: u64,
    free: u64,
    percent: f32,
    fs_type: String,
    is_removable: bool,
}

pub struct Cpu {
    name: String,
    usage: f32,
    clock_speed: f32,
    vendor: String,
}

pub struct Network {
    name: String,
    address: String,
    netmask: String,
    broadcast: String,
    ptp: String,
    mac: String,
    is_up: bool,
    is_running: bool,
    is_loopback: bool,
    mtu: u32,
    rx: u64,
    tx: u64,
}

pub struct SystemData {
    cpus: Vec<Cpu>,
    memory: u64,
    swap: u64,
    disks: Vec<Disk>,
    total_memory: u64,
    total_swap: u64,
    networks: Vec<Network>,
}

pub struct SystemInfo {
    usage: SystemData,
    processes: Vec<ProcessData>,
    spec: SystemSpec,
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

        self.spec.os = sysinfo::System::os_version().unwrap_or_default().to_string();
        self.spec.hostname = sysinfo::System::host_name().unwrap_or_default().to_string();
        self.spec.kernel = sysinfo::System::kernel_version().unwrap_or_default().to_string();
        self.spec.uptime = sysinfo::System::uptime().to_string();

        let mut users = vec![];

        for user in sysinfo::get_users() {
            users.push(User {
                name: user.name().to_string(),
                uid: user.uid() as f32,
                gid: user.gid() as f32,
                groups: user.groups().iter().map(|x| x.to_string()).collect(),
            });
        }

        self.spec.users = users;
    }
}