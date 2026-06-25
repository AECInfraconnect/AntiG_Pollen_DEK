use crate::model::{DiscoveryEvidenceV2, EvidenceSource, PrivacyClass};
use dek_fingerprint_defs::model::{BrowserProcessDef, WebAiSignatureDef};
use std::collections::HashSet;

pub struct BrowserWindow {
    pub pid: u32,
    pub process_name: String,
    pub title: String,
    pub cmdline: String,
}

pub fn scan_browser_windows(
    web_ai: &[WebAiSignatureDef],
    browsers: &[BrowserProcessDef],
) -> Vec<DiscoveryEvidenceV2> {
    let mut out = vec![];
    let windows = enumerate_browser_windows(browsers);
    let mut seen: HashSet<String> = Default::default();

    for w in windows {
        for sig in web_ai {
            let by_title = sig.title_patterns.iter().any(|p| title_match(p, &w.title));
            let by_app = sig
                .app_cmdline_patterns
                .iter()
                .any(|p| glob_match(&p.to_lowercase(), &w.cmdline.to_lowercase()));

            if by_title || by_app {
                let key = format!("webai:{}:{}", sig.id, w.process_name);
                if !seen.insert(key.clone()) {
                    continue;
                }
                out.push(DiscoveryEvidenceV2 {
                    evidence_id: uuid::Uuid::new_v4().to_string(),
                    source: EvidenceSource::BrowserWindow,
                    confidence: if by_app { 0.95 } else { 0.85 },
                    observed_at: chrono::Utc::now().to_rfc3339(),
                    privacy_class: PrivacyClass::InternalMetadata,
                    redacted: true,
                    data: serde_json::json!({
                        "name": sig.name,
                        "vendor": sig.vendor,
                        "domain": sig.domain,
                        "browser": w.process_name,
                        "detected_via": if by_app { "app_cmdline" } else { "window_title" },
                        "capability_tags": sig.capability_tags,
                    }),
                    merge_key: Some(key),
                    source_path_hash: None,
                    source_path_redacted: Some(w.process_name.clone()),
                });
            }
        }
    }
    out
}

fn title_match(pat: &str, title: &str) -> bool {
    let t = title.to_lowercase();
    let p = pat.to_lowercase();
    t.split(['-', '—', '|']).any(|seg| seg.trim() == p) || t.contains(&p)
}

fn glob_match(pat: &str, text: &str) -> bool {
    let pat_regex = pat.replace("*", ".*");
    if let Ok(re) = regex::Regex::new(&pat_regex) {
        re.is_match(text)
    } else {
        text.contains(&pat.replace("*", ""))
    }
}

// ── Windows ──
#[cfg(target_os = "windows")]
fn enumerate_browser_windows(browsers: &[BrowserProcessDef]) -> Vec<BrowserWindow> {
    windows_impl::enumerate(browsers)
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
mod windows_impl {
    use super::*;
    use sysinfo::System;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        IsWindowVisible,
    };

    struct EnumState<'a> {
        browsers: &'a [BrowserProcessDef],
        sys: &'a System,
        out: Vec<BrowserWindow>,
    }

    unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let state = &mut *(lparam.0 as *mut EnumState);

        if IsWindowVisible(hwnd).as_bool() {
            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));

            if pid > 0 {
                // Check if this pid belongs to a browser
                if let Some(process) = state.sys.process(sysinfo::Pid::from_u32(pid)) {
                    let pname = process.name().to_string_lossy().to_ascii_lowercase();
                    let is_browser = state.browsers.iter().any(|b| {
                        b.process_names
                            .iter()
                            .any(|n| n.eq_ignore_ascii_case(&pname))
                    });

                    if is_browser {
                        let len = GetWindowTextLengthW(hwnd);
                        let title = if len > 0 {
                            let mut buf = vec![0u16; (len + 1) as usize];
                            GetWindowTextW(hwnd, &mut buf);
                            String::from_utf16_lossy(&buf[..len as usize])
                        } else {
                            String::new()
                        };

                        let cmdline = process
                            .cmd()
                            .iter()
                            .map(|s| s.to_string_lossy().into_owned())
                            .collect::<Vec<_>>()
                            .join(" ");

                        state.out.push(BrowserWindow {
                            pid,
                            process_name: pname,
                            title,
                            cmdline,
                        });
                    }
                }
            }
        }
        BOOL::from(true)
    }

    pub fn enumerate(browsers: &[BrowserProcessDef]) -> Vec<BrowserWindow> {
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let mut state = EnumState {
            browsers,
            sys: &sys,
            out: Vec::new(),
        };

        unsafe {
            let lparam = LPARAM(&mut state as *mut _ as isize);
            let _ = EnumWindows(Some(enum_window_proc), lparam);
        }

        state.out
    }
}

// ── macOS ──
#[cfg(target_os = "macos")]
fn enumerate_browser_windows(browsers: &[BrowserProcessDef]) -> Vec<BrowserWindow> {
    macos_impl::enumerate(browsers)
}

#[cfg(target_os = "macos")]
mod macos_impl {
    use super::*;
                    .any(|n| n.eq_ignore_ascii_case(&pname))
            });
            if is_browser {
                let cmdline = process
                    .cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(" ");
                out.push(BrowserWindow {
                    pid: pid.as_u32(),
                    process_name: pname,
                    title: String::new(), // Full implementation requires CFDictionary parsing
                    cmdline,
                });
            }
        }
        out
    }
}

// ── Linux ──
#[cfg(target_os = "linux")]
fn enumerate_browser_windows(browsers: &[BrowserProcessDef]) -> Vec<BrowserWindow> {
    linux_impl::enumerate(browsers)
}

#[cfg(target_os = "linux")]
mod linux_impl {
    use super::*;
    pub fn enumerate(browsers: &[BrowserProcessDef]) -> Vec<BrowserWindow> {
        let mut out = vec![];
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        // Placeholder for x11rb _NET_CLIENT_LIST logic
        // We fallback to sysinfo cmdline extraction for Wayland/missing X11
        for (pid, process) in sys.processes() {
            let pname = process.name().to_string_lossy().to_ascii_lowercase();
            let is_browser = browsers.iter().any(|b| {
                b.process_names
                    .iter()
                    .any(|n| n.eq_ignore_ascii_case(&pname))
            });
            if is_browser {
                let cmdline = process
                    .cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(" ");
                out.push(BrowserWindow {
                    pid: pid.as_u32(),
                    process_name: pname,
                    title: String::new(),
                    cmdline,
                });
            }
        }
        out
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn enumerate_browser_windows(_browsers: &[BrowserProcessDef]) -> Vec<BrowserWindow> {
    vec![]
}
