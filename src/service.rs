//! Runs the pop-launcher process and exchanges line-based JSON with it.

use adw::gio;
use adw::gio::prelude::*;
use adw::glib;

use pop_launcher::{Request, Response};

pub struct Service {
    stdin: gio::OutputStream,
    stdout: gio::InputStream,
}

impl Service {
    pub fn spawn() -> Result<Self, glib::Error> {
        let process = gio::Subprocess::newv(
            &[std::ffi::OsStr::new("pop-launcher")],
            gio::SubprocessFlags::STDIN_PIPE | gio::SubprocessFlags::STDOUT_PIPE,
        )?;
        Ok(Self {
            stdin: process.stdin_pipe().expect("stdin is piped"),
            stdout: process.stdout_pipe().expect("stdout is piped"),
        })
    }

    /// Calls `on_response` on the main loop for each message from pop-launcher.
    pub fn listen(&self, on_response: impl Fn(Response) + 'static) {
        let reader = gio::DataInputStream::new(&self.stdout);
        glib::spawn_future_local(async move {
            loop {
                match reader.read_line_utf8_future(glib::Priority::DEFAULT).await {
                    Ok(Some(line)) => match serde_json::from_str(&line) {
                        Ok(response) => on_response(response),
                        Err(_) => eprintln!("ignoring message: {line}"),
                    },
                    Ok(None) => {
                        eprintln!("pop-launcher exited");
                        break;
                    }
                    Err(error) => {
                        eprintln!("failed to read from pop-launcher: {error}");
                        break;
                    }
                }
            }
        });
    }

    pub fn send(&self, request: &Request) {
        let mut line = serde_json::to_string(request).expect("requests always serialize");
        line.push('\n');
        if let Err(error) = self
            .stdin
            .write_all(line.as_bytes(), gio::Cancellable::NONE)
        {
            eprintln!("failed to write to pop-launcher: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real lines from pop-launcher 1.2.7, to catch the git dependency drifting from the binary.
    #[test]
    fn decodes_messages_from_the_installed_service() {
        let update = r#"{"Update":[{"id":0,"name":"Firefox","description":"System",
            "icon":{"Name":"firefox"},"category_icon":{"Name":"new-window-symbolic"}}]}"#;
        assert!(matches!(
            serde_json::from_str(update),
            Ok(Response::Update(results)) if results[0].name == "Firefox"
        ));
        assert!(matches!(
            serde_json::from_str(r#"{"Fill":"= 4"}"#),
            Ok(Response::Fill(text)) if text == "= 4"
        ));
        let entry = r#"{"DesktopEntry":{"path":"/a/b.desktop",
            "gpu_preference":"Default","action_name":null}}"#;
        assert!(matches!(
            serde_json::from_str(entry),
            Ok(Response::DesktopEntry { .. })
        ));
    }
}
