use std::{io, process::Stdio, sync::Arc};

use tokio::process::Command;

use crate::{
    all_args_start_with_dash, handle_context::Context, kakoune::Kakoune, searchable_args,
    string_ext::SplitArgs,
};

pub async fn process(context: Arc<Context>, args: &[String], kak: &Kakoune) -> io::Result<()> {
    let fd_path = context.get_tool("fd");

    if fd_path.is_none() {
        kak.run_command("echo -debug Error: fd is missing").await?;
        Err(io::Error::other("fd is missing from path, can't process"))?;
    }
    searchable_args(args)?;
    let fd_path = fd_path.expect("Just checked");

    let fifo = context.ensure_fifo("fd").await?;
    let fifo_path = Context::get_fifo_path("fd");
    kak.run_command_in_client(&format!("edit! -fifo {} *fd*", fifo_path.to_string_lossy()))
        .await?;

    let fifo = fifo.lock().await;
    let fifo_clone = fifo.try_clone()?;

    let splitted_args: Vec<String> = args[0].split_args();
    all_args_start_with_dash(&splitted_args)?;
    let cwd = context.get_cwd().await.expect("Missing CWD");

    let mut cmd = Command::new(fd_path)
        .arg("--color=always")
        .arg("-p")
        .args(splitted_args)
        .arg("-tf")
        .current_dir(cwd)
        .stdout(Stdio::from(fifo_clone))
        .spawn()?;

    cmd.wait().await?;
    Ok(())
}
