use std::process::Stdio;

use tokio::{io::{self, AsyncWriteExt}, process::Command};

pub struct KakClient<'a>(pub &'a str);
pub struct KakSession<'a>(pub &'a str);

pub struct Kakoune {
  session: String,
  client: String,
}

impl Kakoune {
  pub fn new(session: KakSession, client: KakClient) -> Self {
    Self { session: session.0.into(), client: client.0.into() }
  }

  pub async fn run_command(&self, command: &str) -> io::Result<()>{
    println!("Kakoune {}", command);

    let mut kak_process = Command::new("kak")
      .arg("-p")
      .arg(&self.session)
      .stdin(Stdio::piped())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()?;

    let mut kak_stdin = kak_process.stdin.take().expect("Failed to open kak stdin");
    kak_stdin.write_all(command.as_bytes()).await?;

    Ok(())

  }

  pub async fn run_command_in_client(&self, command: &str) -> io::Result<()> {
    let cmd = format!("evaluate-commands -client {} %{{ {} }}", &self.client, command);
    self.run_command(&cmd).await
  }
}
