use std::{
    io::{self, BufReader, Read, Seek, SeekFrom, Write},
    os::{
        fd::AsRawFd,
        unix::fs::{FileTypeExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::io::AsyncBufReadExt;

mod handle_context;
mod kakoune;
mod processors;
mod string_ext;
mod temp_dir;

use crate::kakoune::Kakoune;
use crate::{
    handle_context::Context,
    kakoune::{KakClient, KakSession},
};

type MainResult = Result<(), Box<dyn std::error::Error>>;

const LOCK_PATH: &str = "/tmp/kak-univ-search.lock";

fn main() -> MainResult {
    let args: Vec<String> = std::env::args().collect();
    let should_daemonize = args.iter().any(|a| a == "-d" || a == "--daemon");
    let mut lock_file = std::fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(LOCK_PATH)?;

    let fd = lock_file.as_raw_fd();

    unsafe {
        if libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) != 0 {
            let mut reader = BufReader::new(&lock_file);
            let mut pid_str = String::new();
            let _ = reader.read_to_string(&mut pid_str);

            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                eprintln!("Killing duplicate daemon (PID: {})", pid);
                libc::kill(pid, libc::SIGTERM);
                libc::flock(fd, libc::LOCK_EX);
            }
        }
    }
    lock_file.set_len(0)?;

    let mut writer = &lock_file;
    writer.seek(SeekFrom::Start(0))?;

    if should_daemonize {
        daemonize::Daemonize::new()
            .working_directory("/tmp")
            .start()?;
    } else {
        eprintln!("Running in foreground.");
    }

    lock_file.set_len(0)?;
    (&lock_file).seek(SeekFrom::Start(0))?;
    writeln!(lock_file, "{}", std::process::id())?;

    Box::leak(Box::new(lock_file));

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async_main())
}

async fn async_main() -> MainResult {
    let ctx = Arc::new(
        Context::new(&["rg", "fzf", "fd", "ctags", "global", "ssort"]).expect("Can't find tools"),
    );
    let req_fifo = "/tmp/kak-univ-search";
    ensure_fifo(req_fifo)?;

    let file = tokio::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(req_fifo)
        .await?;

    let mut reader = tokio::io::BufReader::new(file);
    let mut args: Vec<String> = Vec::new();
    let mut buf = Vec::new();

    while reader.read_until(b'\0', &mut buf).await? != 0 {
        eprintln!("Received {} bytes: {:?}", buf.len(), buf); // DEBUG
        if let Some(&b'\0') = buf.last() {
            buf.pop();
        }

        if buf.is_empty() {
            // Empty token
            if !args.is_empty() {
                let task_args = std::mem::take(&mut args);
                let tc = ctx.clone();
                dbg!(&task_args);

                tokio::spawn(async move {
                    if let Err(e) = handle_request(task_args, tc).await {
                        eprintln!("Handler error: {}", e);
                    }
                });
            }
        } else {
            args.push(String::from_utf8_lossy(&buf).into_owned());
        }
        buf.clear();
    }

    Ok(())
}

async fn handle_request(args: Vec<String>, ctx: Arc<Context>) -> io::Result<()> {
    println!("Handling request: {:?}", &args);
    if args.len() < 4 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Insufficient number of args",
        ));
    }
    let cmd = &args[0];
    let kak_session = &args[1];
    let kak_client = &args[2];
    let cwd = PathBuf::from(&args[3]);
    let cmd_args = &args[4..];

    ctx.set_cwd(cwd).await;

    let kakoune = Kakoune::new(KakSession(kak_session), KakClient(kak_client));

    match cmd.as_str() {
        "rg" => processors::rg::process(ctx.clone(), cmd_args, &kakoune).await?,
        "fd" => processors::fd::process(ctx.clone(), cmd_args, &kakoune).await?,
        "fzf" => processors::fzf::process(ctx.clone(), cmd_args, &kakoune).await?,
        "global" => processors::global::process(ctx.clone(), cmd_args, &kakoune).await?,
        "buffer-search" => processors::buffer::process(ctx.clone(), cmd_args, &kakoune).await?,
        "buffer-list-search" => {
            processors::buffer_list::process(ctx.clone(), cmd_args, &kakoune).await?
        }
        "ping" => process_ping(&args, &kakoune).await?,
        _ => eprintln!("Error: Unknown command"),
    }
    Ok(())
}

async fn process_ping(_args: &[String], kak: &Kakoune) -> io::Result<()> {
    kak.run_command_in_client("echo -markup {Error}PONG PONG")
        .await
}

fn searchable_args(args: &[String]) -> Result<(), io::Error> {
    if args.is_empty() || args[0].is_empty() {
        return Err(io::Error::other("Search entry empty"));
    }

    Ok(())
}

fn all_args_start_with_dash(args: &[String]) -> Result<(), io::Error> {
    let non_dashed_arg = args
        .iter()
        .fold(false, |acc, v| if acc { true } else { !v.starts_with("-") });

    if non_dashed_arg {
        Ok(())
    } else {
        return Err(io::Error::other("All args start with dash"));
    }
}

fn ensure_fifo(path: &str) -> io::Result<()> {
    if Path::new(path).exists() {
        if !std::fs::metadata(path)?.file_type().is_fifo() {
            return Err(err("Path exists and is not a fifo"));
        }
    } else {
        use nix::sys::stat;
        use nix::unistd::mkfifo;
        mkfifo(path, stat::Mode::from_bits(0o600).unwrap())?
    }
    Ok(())
}

fn err(msg: &str) -> std::io::Error {
    std::io::Error::other(msg)
}
