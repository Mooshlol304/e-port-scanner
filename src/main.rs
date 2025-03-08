use std::net::{SocketAddr, TcpStream};
use std::process::Command;
use std::time::Duration;
use rayon::prelude::*;
use colored::*;
use crossterm::{execute, terminal, ExecutableCommand};
use std::io::{self, Write};

// Function to get the router IP
fn get_router_ip() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("ipconfig")
            .output()
            .expect("Failed to execute ipconfig");
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("Default Gateway") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    return Some(parts[1].trim().to_string());
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("ip")
            .args(["route", "show", "default"])
            .output()
            .expect("Failed to execute ip");
        let output_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = output_str.split_whitespace().collect();
        if let Some(index) = parts.iter().position(|&s| s == "via") {
            if let Some(ip) = parts.get(index + 1) {
                return Some(ip.to_string());
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("route")
            .args(["-n", "get", "default"])
            .output()
            .expect("Failed to execute route");
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("gateway:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    return Some(parts[1].to_string());
                }
            }
        }
    }

    None // Return None if we couldn't determine the router IP
}

fn is_port_open(ip: &str, port: u16) -> bool {
    let address: SocketAddr = format!("{}:{}", ip, port).parse().unwrap();
    let timeout = Duration::from_millis(1); // Slightly increased for reliability

    TcpStream::connect_timeout(&address, timeout).is_ok()
}

fn main() {
    // Set up terminal
    let mut stdout = io::stdout();
    execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap(); // Clear screen
    execute!(stdout, terminal::SetTitle("Router IP Port Scanner")).unwrap(); // Set window title

    // Get the router IP dynamically
    let ip = get_router_ip().unwrap_or_else(|| {
        println!("Could not determine the router IP.");
        std::process::exit(1);
    });

    let start_port = 1;
    let end_port = 65535;

    println!("Scanning ports on {}...", ip);

    (start_port..=end_port).into_par_iter().for_each(|port| {
        if is_port_open(&ip, port) {
            println!("Port {} is {}", port, "DEMILITARIZED".green());
        }
    });

    println!("Scan complete.");
}
