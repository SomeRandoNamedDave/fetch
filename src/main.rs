use std::{
    env::var,
    ffi::OsStr,
    fs::{read_link, read_to_string, File},
    io::{self, BufRead},
    os::unix::prelude::OsStrExt,
    path::Path
};

use anyhow::{bail, Result};
use chrono::prelude::*;
use nvml_wrapper::{enum_wrappers::device::TemperatureSensor::Gpu, error::NvmlError, Nvml};

const BQ: &str = "[0;38:5:19m!?[0m";
const UNKNOWN: &str = "[0;38:5:19mUnknown![0m";

fn main() {
    // may as well hardcode a lot of this stuff as it's unlikely to change
    let term = format!(
        "[3;90mTerminal: [0mwezterm [3;90mShell:[0m {}",
        var("SHELL").unwrap().split('/').last().unwrap()
    );
    let wm = "awesome";
    let editor = "neovim";
    let browser = "qutebrowser";
    let font = "Iosevka [ [3mVictor Mono[0m ]";
    // wallpaper

    let dt = Local::now();
    let mid = dt.format("%H:%M   %a %d %b");

    // temperatures
    // ram usage
    // disk usage
    // kernel
    let (npkg, mpkg) = match pkgs() {
        Ok(x) => x,
        Err(_) => (BQ.to_string(), BQ.to_string())
    };
    // uptime

    println!(
        "                        [1;38:5:16m`
                       -:
                      .//:[0m                       [1;3;4;38:5:16;58:5:18mEnvironment \
         Highlights...[0m
                     [1;38:5:16m`////-
                    `://///.                     [1;31m  ▐ [0m{}
                    [1;38:5:16m:///////.                    [1;32m󰨇  ▐ [0m{}
                   [1;38:5:16m-/////////.                   [1;33m  ▐ [0m{}
                  [1;38:5:16m`://////////`                  [1;34m󰖟  ▐ [0m{}
                 [1;38:5:16m-:..://///////`                 [1;35m  ▐ [0m{}
                [1;38:5:16m-////:::////////`                [1;36m  ▐ [0m{}
               [1;38:5:16m-/////////////////`
              -//////////++++/////`      [0;38:5:18m┌──────────────────────────┐[0m
             [1;38:5:16m-////[0;38:5:17m++++oooooooooo+++.     [38:5:18m│[38:5:20m   {}   \
         [38:5:18m│[0m
            [38:5:17m-/+++oooooooooooooooooo+.    [38:5:18m└──────────────────────────┘[0m
           [38:5:17m:+oooooooo+-..-/+oooooooo+.
         `/ooooooooo:`     .+oooooooo+.          [1;3;4;38:5:17;58:5:18mSystem Information...[0m
        [38:5:17m`/ooooooooo/        .ooooooooo+-
       `/oooooooooo`         /oooooo++++-        [1;91m󰔄  ▐ [0;3;90mcpu:[0m {} [3;90mgpu: [0m{}
      [38:5:17m`+ooooooooooo`         :oooooo++/-:.       [1;92m  ▐ [0m{}
     [38:5:17m.+ooooooooooo+`         :+oooooooo+/-`      [1;93m  ▐ [0m{}
    [38:5:17m.+oooooo++/:-.`          `..-:/++ooooo+:     [1;94m  ▐ [0m{}
   [38:5:17m.+oo++/-.`                      `.-:++ooo:    [1;95m  ▐ [0;3;90mOfficial:[0m {} \
         [3;90mAUR/Local:[0m {}
  [38:5:17m-++/-.`          [1;4:4;38:5:16;58:5:17mArch Linux[0;38:5:19m™[38:5:17m          \
         `-:++/`  [1;96m󱎫  ▐ [0m{}
  [38:5:17m-++/-.`                               `-:++/`[0m
 [38:5:17m.:.`  [3;38:5:19ma simple, lightweight distribution![0;38:5:17m  .--[0m\n",
        term,
        wm,
        editor,
        browser,
        font,
        wall().unwrap_or_else(|_| UNKNOWN.to_string()),
        mid,
        ctemp().unwrap_or_else(|_| BQ.to_string()),
        gtemp().unwrap_or_else(|_| BQ.to_string()),
        ram().unwrap_or_else(|_| UNKNOWN.to_string()),
        disk().unwrap_or_else(|e| e.to_string()),
        kernel().unwrap_or_else(|e| e.to_string()),
        npkg,
        mpkg,
        uptime().unwrap_or_else(|_| UNKNOWN.to_string())
    );
}

fn wall() -> Result<String> {
    let p = read_link(
        [
            &var("XDG_CONFIG_HOME")
                .unwrap_or_else(|_| [&var("HOME").unwrap(), ".config"].join("/")),
            "awesome",
            "assets",
            "wallpaper"
        ]
        .join("/")
    )?;
    let mut f = p.file_name().unwrap().to_string_lossy().to_string();
    match f.len() <= 28 {
        true => Ok(f),
        false => {
            f.truncate(26);
            Ok([f, "…".to_string()].join(""))
        }
    }
}

fn ctemp() -> Result<String> {
    let input = read_to_string("/sys/devices/platform/coretemp.0/hwmon/hwmon2/temp1_input")?;
    let temp = input.trim().parse::<i32>()?;
    Ok((temp as f32 / 1_000f32).to_string())
}

fn gtemp() -> Result<String, NvmlError> {
    let nvml = Nvml::init()?;
    let device = nvml.device_by_index(0)?;
    let temp = device.temperature(Gpu)?;
    Ok(temp.to_string())
}

fn ram() -> Result<String> {
    let mut lines = io::BufReader::new(File::open("/proc/meminfo")?).lines().take(26);

    let total = lines
        .next()
        .unwrap()?
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse::<u64>()?
        .saturating_mul(1_024);
    let free = lines
        .next()
        .unwrap()?
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse::<u64>()?
        .saturating_mul(1_024);
    let buffers = lines
        .nth(1)
        .unwrap()?
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse::<u64>()?
        .saturating_mul(1_024);
    let cached = lines
        .next()
        .unwrap()?
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse::<u64>()?
        .saturating_mul(1_024);
    let s_reclaimable = lines
        .nth(20)
        .unwrap()?
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse::<u64>()?
        .saturating_mul(1_024);

    let used = total - free - buffers - cached - s_reclaimable;

    Ok(fmt_ram(used, total))
}

fn fmt_ram(u: u64, t: u64) -> String {
    let used = [(u / 1_000_000).to_string().as_str(), "[3;90mMB[0m"].join(" ");
    let mut perc = u as f64 / t as f64 * 100f64;
    perc = perc.round();
    let bar = get_bar(perc as u8);
    [bar, used].join(" ")
}

fn disk() -> Result<String> {
    let mut total = 0;
    let mut available = 0;

    let path = Path::new("/");
    let path_os: &OsStr = path.as_ref();
    let mut c_path = path_os.as_bytes().to_vec();
    c_path.push(0);

    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr() as *const _, &mut stat) == 0 {
            let bsize = stat.f_bsize;
            let blocks = stat.f_blocks;
            let bavail = stat.f_bavail;

            total = bsize.saturating_mul(blocks);
            available = bsize.saturating_mul(bavail);
        }
    }

    if total == 0 {
        bail!(UNKNOWN)
    }

    let bar = disk_bar(available, total);
    let used = (((total - available) as f64 / 1_000_000_000f64) * 10f64).round() / 10f64;

    Ok(format!("{} {:.1} [3;90mGB[0m", bar, used))
}

fn disk_bar(a: u64, t: u64) -> String {
    let u = t - a;
    let mut perc = u as f64 / t as f64 * 100f64;
    perc = perc.round();
    get_bar(perc as u8)
}

fn get_bar(n: u8) -> String {
    let r = match n {
        0..=10 => "[38:5:20m[38:5:18m[0m ",
        11..=20 => "[38:5:20m[38:5:18m[0m ",
        21..=30 => "[38:5:20m[38:5:18m[0m ",
        31..=40 => "[38:5:20m[38:5:18m[0m ",
        41..=50 => "[38:5:20m[38:5:18m[0m ",
        51..=60 => "[38:5:20m[38:5:18m[0m ",
        61..=70 => "[38:5:20m[38:5:18m[0m ",
        71..=80 => "[38:5:20m[38:5:18m[0m ",
        81..=90 => "[38:5:202m[38:5:18m[0m ",
        91..=100 => "[38:5:196m[0m ",
        _ => unreachable!()
    };
    r.to_string()
}

fn kernel() -> Result<String> {
    let mut ptr = std::mem::MaybeUninit::<libc::utsname>::zeroed();
    unsafe {
        match libc::uname(ptr.as_mut_ptr()) == 0 {
            true => {
                let r = ptr.assume_init();
                Ok(r.release
                    .iter()
                    .filter(|c| **c != 0)
                    .map(|c| *c as u8 as char)
                    .collect::<String>())
            }
            false => bail!(UNKNOWN)
        }
    }
}

fn pkgs() -> Result<(String, String)> {
    let mut pkgs = read_to_string(
        [
            &var("XDG_CACHE_HOME").unwrap_or_else(|_| [&var("HOME").unwrap(), ".cache"].join("/")),
            "fetch",
            "pkg_stats"
        ]
        .join("/")
    )?;
    pkgs.pop();
    let mut split = pkgs.split(' ');
    Ok((split.next().unwrap().to_string(), split.next().unwrap().to_string()))
}

fn uptime() -> Result<String> {
    let input = read_to_string("/proc/uptime")?;
    let secs = input.split(' ').take(1).next().unwrap_or("");
    let s = secs.parse::<f64>()?.round() as usize;

    let mut r = vec![];

    let mins = s / 60;
    let hrs = mins / 60;
    let days = hrs / 24;

    match days > 0 {
        true if days == 1 => r.push([days.to_string().as_str(), " day, "].join("")),
        true => r.push([days.to_string().as_str(), " days, "].join("")),
        false => {}
    }

    match hrs > 0 {
        true if hrs == 1 => r.push([(hrs % 24).to_string().as_str(), " hour, "].join("")),
        true if (hrs % 24) == 1 => r.push([(hrs % 24).to_string().as_str(), " hour, "].join("")),
        true => r.push([(hrs % 24).to_string().as_str(), " hours, "].join("")),
        false => {}
    }

    match mins > 0 {
        true if mins == 1 => r.push([(mins % 60).to_string().as_str(), " minute"].join("")),
        true if (mins % 60) == 1 => r.push([(mins % 60).to_string().as_str(), " minute"].join("")),
        true => r.push([(mins % 60).to_string().as_str(), " minutes"].join("")),
        false => r.push([(s % 60).to_string().as_str(), " seconds"].join(""))
    }

    Ok(r.join(""))
}
