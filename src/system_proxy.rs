use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RRF_RT_REG_SZ,
};

use crate::native_interop::wide_str;

pub fn request_proxy() -> Option<ureq::Proxy> {
    // Match ureq's existing environment priority before falling back to Windows.
    for name in [
        "ALL_PROXY",
        "all_proxy",
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
    ] {
        if let Some(proxy) = std::env::var(name)
            .ok()
            .and_then(|value| ureq::Proxy::new(value).ok())
        {
            return Some(proxy);
        }
    }
    windows_proxy()
        .and_then(|value| https_proxy(&value))
        .and_then(|value| ureq::Proxy::new(value).ok())
}

fn windows_proxy() -> Option<String> {
    let path = wide_str(r"Software\Microsoft\Windows\CurrentVersion\Internet Settings");
    let enabled_name = wide_str("ProxyEnable");
    let server_name = wide_str("ProxyServer");
    let mut enabled = 0u32;
    let mut size = std::mem::size_of_val(&enabled) as u32;
    // Read again for each poll so changing the system proxy needs no restart.
    unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(path.as_ptr()),
            PCWSTR(enabled_name.as_ptr()),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut enabled as *mut u32).cast()),
            Some(&mut size),
        )
        .ok()
        .ok()?;
        if enabled == 0 {
            return None;
        }
        let mut server = [0u16; 16384];
        size = std::mem::size_of_val(&server) as u32;
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(path.as_ptr()),
            PCWSTR(server_name.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(server.as_mut_ptr().cast()),
            Some(&mut size),
        )
        .ok()
        .ok()?;
        let end = server.iter().position(|&ch| ch == 0)?;
        Some(String::from_utf16_lossy(&server[..end]))
    }
}

fn https_proxy(server: &str) -> Option<String> {
    // shortcut: static Windows proxy only; add PAC resolution if a user needs PAC.
    let address = if server.contains('=') {
        server.split(';').find_map(|entry| {
            let (scheme, address) = entry.split_once('=')?;
            scheme
                .trim()
                .eq_ignore_ascii_case("https")
                .then_some(address.trim())
        })?
    } else {
        server.trim()
    };
    if address.is_empty() {
        return None;
    }
    Some(if address.contains("://") {
        address.to_owned()
    } else {
        format!("http://{address}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_https_proxy_formats() {
        for (input, expected) in [
            ("http://127.0.0.1:12450", Some("http://127.0.0.1:12450")),
            (" 127.0.0.1:12450 ", Some("http://127.0.0.1:12450")),
            (
                "http=localhost:80; HTTPS = localhost:81 ;socks=localhost:82",
                Some("http://localhost:81"),
            ),
            ("http=localhost:80", None),
            ("https=", None),
            ("", None),
        ] {
            assert_eq!(https_proxy(input).as_deref(), expected, "{input}");
        }
    }
}
