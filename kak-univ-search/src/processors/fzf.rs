use std::{io, process::Stdio, sync::Arc};

use tokio::process::Command;

use crate::{
    all_args_start_with_dash, handle_context::Context, kakoune::Kakoune, searchable_args,
    string_ext::SplitArgs,
};

pub async fn process(context: Arc<Context>, args: &[String], kak: &Kakoune) -> io::Result<()> {
    let fzf_path = context.get_tool("fzf");

    if fzf_path.is_none() {
        kak.run_command("echo -debug Error: fzf is missing").await?;
        Err(io::Error::other("fzf is missing from path, can't process"))?;
    }
    searchable_args(args)?;
    let fzf_path = fzf_path.expect("Just checked");

    let fifo = context.ensure_fifo("fzf").await?;
    let fifo_path = Context::get_fifo_path("fzf");
    kak.run_command_in_client(&format!(
        "edit! -fifo {} *fzf*",
        fifo_path.to_string_lossy()
    ))
    .await?;

    let fifo = fifo.lock().await;
    let fifo_clone = fifo.try_clone()?;

    let splitted_args: Vec<String> = args[0].split_args();
    all_args_start_with_dash(&splitted_args)?;
    let cwd = context.get_cwd().await.expect("Missing CWD");

    let mut cmd = Command::new(fzf_path)
        .arg("--color")
        .arg("-f")
        .args(splitted_args)
        .env("FZF_DEFAULT_COMMAND", "rg --files")
        .current_dir(cwd)
        .stdout(Stdio::from(fifo_clone))
        .spawn()?;

    cmd.wait().await?;
    Ok(())
}
