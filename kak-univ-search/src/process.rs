use std::{io, time::Duration};

use tokio::{
    io::AsyncWriteExt,
    process::{Child, Command},
    time::timeout,
};

#[cfg(not(test))]
const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
#[cfg(test)]
const COMMAND_TIMEOUT: Duration = Duration::from_millis(50);

pub fn spawn(command: &mut Command) -> io::Result<Child> {
    command.kill_on_drop(true);

    #[cfg(unix)]
    command.process_group(0);

    command.spawn()
}

pub fn supervise(child: Child) {
    tokio::spawn(async move {
        if let Err(error) = wait(child).await {
            eprintln!("Command error: {error}");
        }
    });
}

async fn wait(mut child: Child) -> io::Result<()> {
    match timeout(COMMAND_TIMEOUT, child.wait()).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => {
            terminate(&mut child).await;
            Err(error)
        }
        Err(_) => {
            terminate(&mut child).await;
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "command exceeded the 5 second timeout",
            ))
        }
    }
}

pub async fn write_stdin_and_wait(mut child: Child, input: &[u8]) -> io::Result<()> {
    let result = timeout(COMMAND_TIMEOUT, async {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("command stdin is not piped"))?;
        stdin.write_all(input).await?;
        drop(stdin);
        child.wait().await?;
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => {
            terminate(&mut child).await;
            Err(error)
        }
        Err(_) => {
            terminate(&mut child).await;
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "command exceeded the 5 second timeout",
            ))
        }
    }
}

async fn terminate(child: &mut Child) {
    #[cfg(unix)]
    if let Some(pid) = child.id() {
        // Each command is spawned in its own process group, so this also stops descendants.
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
    }

    let _ = child.kill().await;
    let _ = child.wait().await;
}

#[cfg(test)]
mod tests {
    use std::{process::Stdio, time::Instant};

    use super::*;

    #[tokio::test]
    async fn kills_and_reaps_timed_out_command() {
        let mut command = Command::new("sleep");
        command.arg("30").stdout(Stdio::null());
        let child = spawn(&mut command).unwrap();
        let pid = child.id().unwrap();
        let started = Instant::now();

        let error = wait(child).await.unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_eq!(unsafe { libc::kill(pid as i32, 0) }, -1);
    }
}
