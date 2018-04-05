#![feature(iterator_find_map)]
extern crate regex;
#[macro_use] extern crate failure;
#[macro_use] extern crate lazy_static;

use failure::Error;
use std::process::Command;
use regex::Regex;

lazy_static! {
    static ref IS_PROTO: Regex = Regex::new(r"(?i)^\s*(tcp|udp)").unwrap();
    static ref EXTRACT_PORT : Regex = Regex::new(r"[.:](\d+)$").unwrap();
}
#[cfg(target_os="macos")]
static COLS:(usize, usize) = (3,8);
#[cfg(target_os="linux")]
static COLS:(usize, usize) = (4,6);
#[cfg(target_os="windows")]
static COLS:(usize, usize) = (1,4);

fn get_cmd() -> Result<Command, Error> {
    if cfg!(target_os = "macos") {
        let mut cmd = Command::new("netstat");
        cmd.args(&["-anvp", "tcp"]);
        Ok(cmd)
    } else if cfg!(target_os="linux") {
        let mut cmd = Command::new("ss");
        cmd.arg("-tunkp");
        Ok(cmd)
    } else if cfg!(target_os="windows") {
        let mut cmd = Command::new("netstat");
        cmd.arg("-ano");
        Ok(cmd)
    } else {
        Err(format_err!("unknown platform"))
    }
}

pub fn pid_from_port(p: u16) -> Result<u32, Error> {
    let out = get_cmd()?.output()?;
    if !out.status.success() {
        return Err(format_err!("Error running command: {:?}", out.status))
    }
    let strings  = String::from_utf8_lossy(&out.stdout);
    for line in strings.lines() {
        if !IS_PROTO.is_match(line) {
            continue
        }
        let mut columns = line.split_whitespace();

        let port = columns.nth(COLS.0)
            .and_then(|port| EXTRACT_PORT.captures(port))
            .and_then(|capts| capts.get(1))
            .map(|port_capt| port_capt.as_str())
            .ok_or_else(|| format_err!("Could not find port {} in {}", p, line))
            .and_then(|port_str| port_str.parse::<u16>().map_err(|_| format_err!("Parse error parsing {:?}", port_str)));

        if let Ok(port) = port {
            if port == p {
                return columns.nth(COLS.1 - COLS.0 - 1)
                    .ok_or_else(|| format_err!("No PID found in line {} at column {}", line, COLS.1 - COLS.0 - 1))
                    .and_then(|pid| pid.parse::<u32>().map_err(|_| format_err!("Parse error")));
            }
        }
    }
    Err(format_err!("No process uses '{}'", p))
}

#[cfg(test)]
mod tests {
    use super::pid_from_port;

    #[test]
    fn test_pid_from_port_unused_port() {
        let res = pid_from_port(0);
        assert!(res.is_err())
    }

    #[test]
    fn test_pid_from_port_used_port() {
        let res = pid_from_port(22);
        assert!(res.is_ok())
    }

    #[test]
    fn test_pid_from_port_is_1_for_port_22() {
        let res = pid_from_port(22);
        assert_eq!(res.unwrap(), 1);
    }

    #[test]
    fn test_finds_own_pid_when_we_run_a_server() {
        #![feature(getpid)] // Will be stable in Rust 1.27.0
        use std::process;
        use std::net::TcpListener;
        const PORT : u16 = 61233;
        let listener = TcpListener::bind(format!("127.0.0.1:{}", PORT)).expect(&*format!("Could not bind to {}", PORT));
        assert_eq!(pid_from_port(PORT).unwrap(), process::id());
        drop(listener);
    }
}
