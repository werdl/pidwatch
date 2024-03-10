mod info;

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout}, prelude::{CrosstermBackend, Stylize, Terminal}, style::Style, widgets::{Block, Borders, Paragraph, Row, Table}
};
use std::io::{stdout, Result};

use itertools::Itertools;

fn main() -> Result<()> {
    let mut sys = info::SystemInfo::new();
    sys.populate();

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;


    let network_order = sys
        .usage
        .networks
        .iter()
        .map(|n| n.name.clone())
        .collect::<Vec<String>>();

    loop {
        sys.populate();

        // four sections: specs, processes, usage (ram, cpu, disk, swap), network
        // each section has an expandable view, (s, p, u, n)
        // by default, the usage section is expanded

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }

        // now match the widget and display the appropriate section
        // here, have a title bar, for example "Windows 10" or "Debian 13"
        // then have a list of specs, like "Hostname: <hostname>"

        let _ = terminal.draw(|frame| {
            let main_layout = Layout::new(
                Direction::Vertical,
                [
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ],
            )
            .split(frame.size());
            frame.render_widget(
                Block::new()
                    .borders(Borders::TOP)
                    .title(format!("{} {}", sys.spec.os, sys.spec.kernel))
                    .bold(),
                main_layout[0],
            );
            frame.render_widget(
                Block::new().borders(Borders::TOP).title("pidwatch").bold(),
                main_layout[2],
            );

            let inner_layout = Layout::new(
                Direction::Horizontal,
                [Constraint::Percentage(50), Constraint::Percentage(50)],
            )
            .split(main_layout[1]);

            let left_layout = Layout::new(
                Direction::Vertical,
                [Constraint::Percentage(50), Constraint::Percentage(50)],
            )
            .split(inner_layout[0]);

            let right_layout = Layout::new(
                Direction::Vertical,
                [Constraint::Percentage(50), Constraint::Percentage(50)],
            )
            .split(inner_layout[1]);

            // now split each column into two rows, giving us a 2x2 grid
            frame.render_widget(
                Block::default().borders(Borders::ALL).title("CPU").yellow(),
                left_layout[0],
            );
            frame.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Memory")
                    .blue(),
                left_layout[1],
            );

            frame.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Specs/Network")
                    .red(),
                right_layout[0],
            );
            frame.render_widget(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Processes")
                    .magenta(),
                right_layout[1],
            );

            // now we can render the actual data

            // first, create four handles, to each paragraph widget
            let top_left_inner = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(2),
                    Constraint::Min(1),
                ])
                .split(left_layout[0]);

            let top_right_inner = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1)])
                .split(right_layout[0]);

            let top_right_inner_inner = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Min(1),
                ])
                .split(top_right_inner[0]);

            let bottom_left_inner = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(1)])
                .split(left_layout[1]);

            let bottom_right_inner = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(1)])
                .split(right_layout[1]);

            // now we can render the actual data

            let formatted_cpu = format!(
                "Average Usage: {:.2}%\nAverage Clock Speed: {:.2} GHz\n\n",
                // average the usage of all cpus
                sys.usage.cpus.iter().map(|c| c.usage).sum::<f32>() / sys.usage.cpus.len() as f32,
                // average the clock speed of all cpus
                sys.usage.cpus.iter().map(|c| c.clock_speed).sum::<f32>()
                    / sys.usage.cpus.len() as f32,
            );

            let mut formatted_core_data = String::from("\n");

            for cpu in &sys.usage.cpus {
                formatted_core_data.push_str(&format!(
                    "{} ({:.2}%) at {:.2} GHz ({})\n",
                    cpu.name, cpu.usage, cpu.clock_speed, cpu.vendor,
                ));
            }

            frame.render_widget(Paragraph::new(formatted_cpu).bold(), top_left_inner[1]);

            frame.render_widget(Paragraph::new(formatted_core_data), top_left_inner[2]);

            let uptime_days = sys.spec.uptime.parse::<f32>().unwrap_or_default() / 86400.0;

            let uptime_hours = (uptime_days - uptime_days.floor()) * 24.0;

            let uptime_minutes = (uptime_hours - uptime_hours.floor()) * 60.0;

            let uptime_seconds = (uptime_minutes - uptime_minutes.floor()) * 60.0;

            let formatted_uptime = format!(
                "{}d {}h {}m {}s ({}s)",
                uptime_days.floor(),
                uptime_hours.floor(),
                uptime_minutes.floor(),
                uptime_seconds.floor(),
                sys.spec.uptime
            );

            // now, we move on to system specs
            let formatted_specs = format!(
                "Hostname: {}
OS: {}
Kernel: {}
Uptime: {}
Users: {}",
                sys.spec.hostname,
                sys.spec.os,
                sys.spec.kernel,
                formatted_uptime,
                // users where uid > 1000
                sys.spec
                    .users
                    .iter()
                    .filter(|u| u.uid.parse::<u32>().unwrap_or_default() > 1000)
                    .map(|u| u.name.clone())
                    .collect::<Vec<String>>()
                    .len(),
            );

            frame.render_widget(
                Paragraph::new(formatted_specs).bold(),
                top_right_inner_inner[1],
            );

            // now onto memory
            let formatted_ram = format!(
                "Used: {:.2} GB\nTotal: {:.2} GB",
                sys.usage.memory as f32 / 1024.0 / 1024.0 / 1024.0,
                sys.usage.total_memory as f32 / 1024.0 / 1024.0 / 1024.0,
            );

            let formatted_swap = format!(
                "Used: {:.2} GB\nTotal: {:.2} GB",
                sys.usage.swap as f32 / 1024.0 / 1024.0 / 1024.0,
                sys.usage.total_swap as f32 / 1024.0 / 1024.0 / 1024.0,
            );

            let formatted_disk = format!(
                "Used: {:.2} GB\nTotal: {:.2} GB",
                sys.usage.disks.iter().map(|d| d.used).sum::<u64>() as f32
                    / 1024.0
                    / 1024.0
                    / 1024.0,
                sys.usage.disks.iter().map(|d| d.total).sum::<u64>() as f32
                    / 1024.0
                    / 1024.0
                    / 1024.0,
            );

            frame.render_widget(
                Paragraph::new(format!(
                    "RAM:\n{}\n\nSWAP:\n{}\n\nDISK:\n{}\n",
                    formatted_ram, formatted_swap, formatted_disk
                ))
                .bold(),
                bottom_left_inner[1],
            );

            // now, network
            let mut formatted_network = String::new();

            let raw_networks = sys.usage.networks.clone();

            let ordered_networks = network_order
                .iter()
                .map(|n| raw_networks.iter().find(|x| x.name == *n).unwrap())
                .collect::<Vec<&info::Network>>();

            for network in ordered_networks {
                formatted_network.push_str(&format!(
                    "Name: {}\nMAC: {}\nSent/Recieved: {}B/{}B\n\n",
                    network.name,
                    network.mac,
                    network.total_sent,
                    network.total_recv,
                ));
            }

            frame.render_widget(
                Paragraph::new(formatted_network).bold(),
                top_right_inner_inner[2],
            );

            // now for the big one, processes
            // this will be a table, with the headers being "PID", "Name", "CPU", "Memory", "Uptime"
            // importantly, the table will be sorted by CPU usage

            let mut rows = vec![Row::new(vec!["PID", "Name", "CPU", "Memory", "Uptime"]).style(Style::new().on_red())];

            let sorted_by_cpu = sys
                .processes
                .clone()
                .into_iter()
                .sorted_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());

            // now sum any processes with the same name together
            let mut summed_processes: Vec<crate::info::ProcessData> = vec![];

            for process in sorted_by_cpu {
                if let Some(existing) = summed_processes.iter_mut().find(|p| p.name == process.name)
                {
                    existing.cpu_usage += process.cpu_usage;
                    existing.ram += process.ram;
                    existing.total_time += process.total_time;
                } else {
                    summed_processes.push(process);
                }
            }

            for process in summed_processes {
                rows.push(Row::new(vec![
                    process.pid.to_string(),
                    process.name.clone(),
                    format!("{:.2}%", process.cpu_usage),
                    format!("{:.2} MB", process.ram as f32 / 1024.0 / 1024.0),
                    format!("{}s", process.total_time),
                ]));
            }

            let table = Table::new(
                rows,
                [
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ],
            );

            // render the table
            frame.render_widget(table, bottom_right_inner[1]);
        });
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
