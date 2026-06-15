use std::{io, process::Stdio, sync::Arc};

use tokio::process::Command;

use crate::{
    all_args_start_with_dash, handle_context::Context, kakoune::Kakoune, process, searchable_args,
    string_ext::SplitArgs,
};

pub async fn process(context: Arc<Context>, args: &[String], kak: &Kakoune) -> io::Result<()> {
    let global_path = context.get_tool("global");

    if global_path.is_none() {
        kak.run_command("echo -debug Error: global is missing")
            .await?;
        Err(io::Error::other(
            "global is missing from path, can't process",
        ))?;
    }
    searchable_args(args)?;

    let global_path = global_path.expect("Just checked");

    let fifo = context.ensure_fifo("global").await?;
    let fifo_path = Context::get_fifo_path("global");
    kak.run_command_in_client(&format!(
        "edit! -fifo {} *global*",
        fifo_path.to_string_lossy()
    ))
    .await?;

    let fifo = fifo.lock().await;
    let fifo_clone = fifo.try_clone()?;
    let fifo_clone2 = fifo.try_clone()?;

    let splitted_args: Vec<String> = args[0].split_args();
    all_args_start_with_dash(&splitted_args)?;

    let cwd = context.get_cwd().await.expect("Missing CWD");

    let mut command = Command::new(global_path);
    let child = process::spawn(
        command
            .args(splitted_args)
            .args(["--color=always"])
            .args(["--result=grep"])
            .current_dir(cwd)
            .stdout(Stdio::from(fifo_clone))
            .stderr(Stdio::from(fifo_clone2)),
    )?;

    process::supervise(child);
    Ok(())
}
