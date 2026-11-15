//! Runs the pop-launcher process and exchanges line-based JSON with it.

use adw::gio;
use adw::gio::prelude::*;
use adw::glib;

use pop_launcher::{Request, Response};

/// What [`Service::listen`] reports.
#[derive(Debug)]
pub enum Event {
    /// pop-launcher sent a message.
    Response(Response),
    /// pop-launcher exited, or reading from it failed. Nothing follows this.
    Closed,
}

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

    /// Calls `on_event` on the main loop for each message from pop-launcher, and a last time
    /// when there won't be any more.
    pub fn listen(&self, on_event: impl Fn(Event) + 'static) {
        let reader = gio::DataInputStream::new(&self.stdout);
        glib::spawn_future_local(async move {
            loop {
                match reader.read_line_utf8_future(glib::Priority::DEFAULT).await {
                    Ok(Some(line)) => match serde_json::from_str(&line) {
                        Ok(response) => on_event(Event::Response(response)),
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
            on_event(Event::Closed);
        });
    }

    pub fn send(&self, request: &Request) {
        let line = encode(request);
        let result = self
            .stdin
            .write_all(line.as_bytes(), gio::Cancellable::NONE);
        // A write that fails partway is not an `Err`: its error comes with the byte count.
        if let Err(error) | Ok((_, Some(error))) = result {
            eprintln!("failed to write to pop-launcher: {error}");
        }
    }
}

/// pop-launcher reads one JSON request per line.
fn encode(request: &Request) -> String {
    let mut line = serde_json::to_string(request).expect("requests always serialize");
    line.push('\n');
    line
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

    /// The lines pop-launcher 1.2.7 accepts, for the same reason.
    #[test]
    fn encodes_the_requests_the_launcher_sends() {
        assert_eq!(
            encode(&Request::Search("fire".into())),
            "{\"Search\":\"fire\"}\n"
        );
        assert_eq!(encode(&Request::Activate(3)), "{\"Activate\":3}\n");
        assert_eq!(encode(&Request::Exit), "\"Exit\"\n");
    }
}
