This cli tool enumerates Firefox installs (paths, versions, channels), report whether Firefox is default browser, and emit JSON.
Usage:
```auditor [--pretty] [--machine] [--debug]```

--pretty → pretty JSON.

--machine → terse exit codes (0 ok, 1 none found, 2 not default, 3 error).

--debug → extra diagnostics to stderr.

Output JSON (example):
```
{
  "platform": "windows|macos|linux",
  "default_browser": { "name": "Firefox", "is_ff_default": true, "evidence": "registry/plist/xdg" },
  "installs": [
    { "channel": "release|beta|esr|nightly|unknown",
      "version": "128.0.2",
      "path": "C:\\Program Files\\Mozilla Firefox\\firefox.exe",
      "source": "registry|plist|desktop|pkgmgr" }
  ]
}
```

Project layout:
firefox-install-auditor/
  Cargo.toml
  src/
    main.rs
    model.rs
    probe/
      mod.rs
      windows.rs
      macos.rs
      linux.rs

To build project:
```
Windows: cargo build --release --features windows
macOS: cargo build --release --features macos
Linux: cargo build --release --features linux
```