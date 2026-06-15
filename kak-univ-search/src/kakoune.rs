use std::process::Stdio;

use tokio::{io, process::Command};

use crate::process;

pub struct KakClient<'a>(pub &'a str);
pub struct KakSession<'a>(pub &'a str);

pub struct Kakoune {
    session: String,
    client: String,
}

impl Kakoune {
    pub fn new(session: KakSession, client: KakClient) -> Self {
        Self {
            session: session.0.into(),
            client: client.0.into(),
        }
    }

    pub async fn run_command(&self, command: &str) -> io::Result<()> {
        println!("Kakoune {}", command);

        let mut kak_command = Command::new("kak");
        let kak_process = process::spawn(
            kak_command
                .arg("-p")
                .arg(&self.session)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null()),
        )?;

        process::write_stdin_and_wait(kak_process, command.as_bytes()).await
    }

    pub async fn run_command_in_client(&self, command: &str) -> io::Result<()> {
        let cmd = format!(
            "evaluate-commands -client {} %{{ {} }}",
            &self.client, command
        );
        self.run_command(&cmd).await
    }
}
