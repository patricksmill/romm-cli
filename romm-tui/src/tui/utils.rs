use std::{io, process::Command};

/// Open a URL in the system default browser.
pub fn open_in_browser(url: &str) -> io::Result<std::process::Child> {
    let mut cmd = browser_command_for_url(url)?;
    cmd.spawn()
}

fn unsupported_browser_url(url: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("cover URL must use http or https: {url}"),
    )
}

fn is_supported_browser_url(url: &str) -> io::Result<()> {
    let parsed = reqwest::Url::parse(url).map_err(|_| unsupported_browser_url(url))?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        _ => Err(unsupported_browser_url(url)),
    }
}

fn browser_command_for_url(url: &str) -> io::Result<Command> {
    is_supported_browser_url(url)?;

    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("rundll32.exe");
        cmd.args(["url.dll,FileProtocolHandler", url]);
        Ok(cmd)
    }

    #[cfg(target_os = "macos")]
    {
        let mut cmd = Command::new("open");
        cmd.arg(url);
        Ok(cmd)
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let mut cmd = Command::new("xdg-open");
        cmd.arg(url);
        Ok(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::is_supported_browser_url;

    #[test]
    fn browser_url_validation_allows_http_and_https() {
        assert!(is_supported_browser_url("http://example.com/cover.png").is_ok());
        assert!(is_supported_browser_url("https://example.com/cover.png").is_ok());
    }

    #[test]
    fn browser_url_validation_rejects_local_or_executable_schemes() {
        for url in [
            "file:///C:/Windows/System32/calc.exe",
            "javascript:alert(1)",
            "data:text/html,<script>alert(1)</script>",
        ] {
            assert!(is_supported_browser_url(url).is_err(), "{url}");
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn browser_command_does_not_invoke_cmd_shell() {
        let cmd = super::browser_command_for_url("https://example.com/cover.png").unwrap();
        let program = cmd.get_program().to_string_lossy().to_ascii_lowercase();
        assert_ne!(program, "cmd");
        assert_ne!(program, "cmd.exe");
    }
}
