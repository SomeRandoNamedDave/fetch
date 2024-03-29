use std::{
    env::var,
    ffi::OsStr,
    fs::{read_link, read_to_string, File},
    io::{self, BufRead},
    os::unix::prelude::OsStrExt,
    path::Path
};

use chrono::prelude::*;
use nvml_wrapper::{enum_wrappers::device::TemperatureSensor::Gpu, Nvml};

const BQ: &str = "\x1b[0;38:5:19m!?\x1b[0m";
const UNKNOWN: &str = "\x1b[0;38:5:19mUnknown!\x1b[0m";

fn main() {
    // may as well hardcode a lot of this stuff as it's unlikely to change
    let term = format!(
        "\x1b[3;90mTerminal: \x1b[0mWezTerm \x1b[3;90mShell:\x1b[0m {}",
        var("SHELL").unwrap().split('/').last().unwrap()
    );

    let wm = if let Ok(x) = var("XDG_SESSION_TYPE") {
        match x.as_str() {
            "tty" => "awesome",
            "wayland" => "Hyprland",
            _ => UNKNOWN
        }
    } else {
        UNKNOWN
    };

    let editor = "Neovim";
    let browser = "qutebrowser";
    let font = "Iosevka [ \x1b[3mVictor Mono\x1b[0m ]";
    // wallpaper

    let dt = Local::now();
    let mid = dt.format("%H:%M   %a %d %b");

    // temperatures
    // ram usage
    // disk usage
    // kernel
    let (npkg, mpkg) = match pkgs() {
        Some(x) => x,
        None => (BQ.to_string(), BQ.to_string())
    };
    // uptime

    println!(
        "                        \x1b[1;38:5:16m`
                       -:
                      .//:\x1b[0m                       \x1b[1;3;4;38:5:16;58:5:18mEnvironment \
         Highlights...\x1b[0m
                     \x1b[1;38:5:16m`////-
                    `://///.                     \x1b[1;31m  ▐ \x1b[0m{}
                    \x1b[1;38:5:16m:///////.                    \x1b[1;32m󰨇  ▐ \x1b[0m{}
                   \x1b[1;38:5:16m-/////////.                   \x1b[1;33m  ▐ \x1b[0m{}
                  \x1b[1;38:5:16m`://////////`                  \x1b[1;34m󰖟  ▐ \x1b[0m{}
                 \x1b[1;38:5:16m-:..://///////`                 \x1b[1;35m  ▐ \x1b[0m{}
                \x1b[1;38:5:16m-////:::////////`                \x1b[1;36m  ▐ \x1b[0m{}
               \x1b[1;38:5:16m-/////////////////`
              -//////////++++/////`      \x1b[0;38:5:18m┌──────────────────────────┐\x1b[0m
             \x1b[1;38:5:16m-////\x1b[0;38:5:17m++++oooooooooo+++.     \x1b[38:5:18m│\x1b[38:5:20m   {}   \
         \x1b[38:5:18m│\x1b[0m
            \x1b[38:5:17m-/+++oooooooooooooooooo+.    \x1b[38:5:18m└──────────────────────────┘\x1b[0m
           \x1b[38:5:17m:+oooooooo+-..-/+oooooooo+.
         `/ooooooooo:`     .+oooooooo+.          \x1b[1;3;4;38:5:17;58:5:18mSystem Information...\x1b[0m
        \x1b[38:5:17m`/ooooooooo/        .ooooooooo+-
       `/oooooooooo`         /oooooo++++-        \x1b[1;91m󰔄  ▐ \x1b[0;3;90mcpu:\x1b[0m {} \x1b[3;90mgpu: \x1b[0m{}
      \x1b[38:5:17m`+ooooooooooo`         :oooooo++/-:.       \x1b[1;92m  ▐ \x1b[0m{}
     \x1b[38:5:17m.+ooooooooooo+`         :+oooooooo+/-`      \x1b[1;93m  ▐ \x1b[0m{}
    \x1b[38:5:17m.+oooooo++/:-.`          `..-:/++ooooo+:     \x1b[1;94m  ▐ \x1b[0m{}
   \x1b[38:5:17m.+oo++/-.`                      `.-:++ooo:    \x1b[1;95m  ▐ \x1b[0;3;90mOfficial:\x1b[0m {} \
         \x1b[3;90mAUR/Local:\x1b[0m {}
  \x1b[38:5:17m-++/-.`          \x1b[1;4:4;38:5:16;58:5:17mArch Linux\x1b[0;38:5:19m™\x1b[38:5:17m          \
         `-:++/`  \x1b[1;96m󱎫  ▐ \x1b[0m{}
  \x1b[38:5:17m-++/-.`                               `-:++/`\x1b[0m
 \x1b[38:5:17m.:.`  \x1b[3;38:5:19ma simple, lightweight distribution!\x1b[0;38:5:17m  .--\x1b[0m\n",
        term,
        wm,
        editor,
        browser,
        font,
        wall().unwrap_or_else(|| UNKNOWN.to_string()),
        mid,
        ctemp().unwrap_or_else(|| BQ.to_string()),
        gtemp().unwrap_or_else(|| BQ.to_string()),
        ram().unwrap_or_else(|| UNKNOWN.to_string()),
        disk().unwrap_or_else(|| UNKNOWN.to_string()),
        kernel().unwrap_or_else(|| UNKNOWN.to_string()),
        npkg,
        mpkg,
        uptime().unwrap_or_else(|| UNKNOWN.to_string())
    );
}

fn wall() -> Option<String> {
    let p = read_link(
        [
            &var("XDG_CONFIG_HOME")
                .unwrap_or_else(|_| [&var("HOME").unwrap(), ".config"].join("/")),
            "mood",
            "wallpaper"
        ]
        .join("/")
    )
    .ok()?;
    let mut f = p.file_name().unwrap().to_string_lossy().to_string();
    match f.len() <= 28 {
        true => Some(f),
        false => {
            f.truncate(26);
            Some([f, "…".to_string()].join(""))
        }
    }
}

fn ctemp() -> Option<String> {
    let input = read_to_string("/sys/devices/platform/coretemp.0/hwmon/hwmon2/temp1_input").ok()?;
    let temp = input.trim().parse::<i32>().ok()?;
    Some((temp as f32 / 1_000f32).to_string())
}

fn gtemp() -> Option<String> {
    let nvml = Nvml::init().ok()?;
    let device = nvml.device_by_index(0).ok()?;
    let temp = device.temperature(Gpu).ok()?;
    Some(temp.to_string())
}

fn ram() -> Option<String> {
    let mut lines = io::BufReader::new(File::open("/proc/meminfo").ok()?).lines().take(26);

    let total = lines
        .next()
        .unwrap()
        .ok()?
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse::<u64>()
        .ok()?
        .saturating_mul(1_024);
    let free = lines
        .next()
        .unwrap()
        .ok()?
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse::<u64>()
        .ok()?
        .saturating_mul(1_024);
    let buffers = lines
        .nth(1)
        .unwrap()
        .ok()?
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse::<u64>()
        .ok()?
        .saturating_mul(1_024);
    let cached = lines
        .next()
        .unwrap()
        .ok()?
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse::<u64>()
        .ok()?
        .saturating_mul(1_024);
    let s_reclaimable = lines
        .nth(20)
        .unwrap()
        .ok()?
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse::<u64>()
        .ok()?
        .saturating_mul(1_024);

    let used = total - free - buffers - cached - s_reclaimable;

    Some(fmt_ram(used, total))
}

fn fmt_ram(u: u64, t: u64) -> String {
    let used = [(u / 1_000_000).to_string().as_str(), "\x1b[3;90mMB\x1b[0m"].join(" ");
    let mut perc = u as f64 / t as f64 * 100f64;
    perc = perc.round();
    let bar = get_bar(perc as u8);
    [bar, used].join(" ")
}

fn disk() -> Option<String> {
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
        return None
    }

    let bar = disk_bar(available, total);
    let used = (((total - available) as f64 / 1_000_000_000f64) * 10f64).round() / 10f64;

    Some(format!("{} {:.1} \x1b[3;90mGB\x1b[0m", bar, used))
}

fn disk_bar(a: u64, t: u64) -> String {
    let u = t - a;
    let mut perc = u as f64 / t as f64 * 100f64;
    perc = perc.round();
    get_bar(perc as u8)
}

fn get_bar(n: u8) -> String {
    let r = match n {
        0..=10 => "\x1b[38:5:20m\x1b[38:5:18m\x1b[0m ",
        11..=20 => "\x1b[38:5:20m\x1b[38:5:18m\x1b[0m ",
        21..=30 => "\x1b[38:5:20m\x1b[38:5:18m\x1b[0m ",
        31..=40 => "\x1b[38:5:20m\x1b[38:5:18m\x1b[0m ",
        41..=50 => "\x1b[38:5:20m\x1b[38:5:18m\x1b[0m ",
        51..=60 => "\x1b[38:5:20m\x1b[38:5:18m\x1b[0m ",
        61..=70 => "\x1b[38:5:20m\x1b[38:5:18m\x1b[0m ",
        71..=80 => "\x1b[38:5:20m\x1b[38:5:18m\x1b[0m ",
        81..=90 => "\x1b[38:5:202m\x1b[38:5:18m\x1b[0m ",
        91..=100 => "\x1b[38:5:196m\x1b[0m ",
        _ => unreachable!()
    };
    r.to_string()
}

fn kernel() -> Option<String> {
    let mut ptr = std::mem::MaybeUninit::<libc::utsname>::zeroed();
    unsafe {
        match libc::uname(ptr.as_mut_ptr()) == 0 {
            true => {
                let r = ptr.assume_init();
                Some(
                    r.release
                        .iter()
                        .filter(|c| **c != 0)
                        .map(|c| *c as u8 as char)
                        .collect::<String>()
                )
            }
            false => None
        }
    }
}

fn pkgs() -> Option<(String, String)> {
    let mut pkgs = read_to_string(
        [
            &var("XDG_CACHE_HOME").unwrap_or_else(|_| [&var("HOME").unwrap(), ".cache"].join("/")),
            "fetch",
            "pkg_stats"
        ]
        .join("/")
    )
    .ok()?;
    pkgs.pop();
    let mut split = pkgs.split(' ');
    Some((split.next().unwrap().to_string(), split.next().unwrap().to_string()))
}

fn uptime() -> Option<String> {
    let input = read_to_string("/proc/uptime").ok()?;
    let secs = input.split(' ').take(1).next().unwrap_or("");
    let s = secs.parse::<f64>().ok()?.round() as usize;

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

    Some(r.join(""))
}
