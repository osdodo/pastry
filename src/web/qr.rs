use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::process::Command;

use qrcode::QrCode;
use qrcode::render::svg;

pub fn generate_qr_svg(url: &str) -> Option<String> {
    let code = QrCode::new(url).ok()?;
    let svg = code.render::<svg::Color>().min_dimensions(200, 200).build();
    Some(svg)
}

pub fn get_local_ip() -> Option<String> {
    get_local_ip_from_interfaces().or_else(get_local_ip_from_default_route)
}

fn get_local_ip_from_interfaces() -> Option<String> {
    let output = interface_scan_output()?;
    pick_preferred_ipv4(&output).map(|ip| ip.to_string())
}

#[cfg(target_os = "macos")]
fn interface_scan_output() -> Option<String> {
    command_stdout("ifconfig", &[])
}

#[cfg(target_os = "linux")]
fn interface_scan_output() -> Option<String> {
    command_stdout("ip", &["-4", "addr", "show", "scope", "global"])
}

#[cfg(target_os = "windows")]
fn interface_scan_output() -> Option<String> {
    command_stdout("ipconfig", &[])
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn interface_scan_output() -> Option<String> {
    None
}

fn command_stdout(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

fn pick_preferred_ipv4(text: &str) -> Option<Ipv4Addr> {
    let mut first_non_loopback_v4: Option<Ipv4Addr> = None;

    for ip in extract_ipv4_candidates(text) {
        if ip.is_loopback() || is_link_local_ipv4(&ip) {
            continue;
        }

        if is_private_ipv4(&ip) {
            return Some(ip);
        }

        if first_non_loopback_v4.is_none() {
            first_non_loopback_v4 = Some(ip);
        }
    }

    first_non_loopback_v4
}

fn extract_ipv4_candidates(text: &str) -> Vec<Ipv4Addr> {
    text.split_whitespace()
        .filter_map(|token| {
            let cleaned = token.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
            cleaned.parse::<Ipv4Addr>().ok()
        })
        .collect()
}

fn get_local_ip_from_default_route() -> Option<String> {
    use std::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    match addr.ip() {
        IpAddr::V4(v4) => Some(v4.to_string()),
        _ => None,
    }
}

fn is_private_ipv4(ip: &Ipv4Addr) -> bool {
    let [a, b, _, _] = ip.octets();
    a == 10 || (a == 172 && (16..=31).contains(&b)) || (a == 192 && b == 168)
}

fn is_link_local_ipv4(ip: &Ipv4Addr) -> bool {
    let [a, b, _, _] = ip.octets();
    a == 169 && b == 254
}
