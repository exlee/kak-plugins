use std::{io, process::Stdio, sync::Arc};

use tokio::process::Command;

use crate::{
    all_args_start_with_dash, handle_context::Context, kakoune::Kakoune, process, searchable_args,
    string_ext::SplitArgs,
};

pub async fn process(context: Arc<Context>, args: &[String], kak: &Kakoune) -> io::Result<()> {
    let rg_path = context.get_tool("rg");

    if rg_path.is_none() {
        kak.run_command("echo -debug Error: rg is missing").await?;
        Err(io::Error::other("rg is missing from path, can't process"))?;
    }
    searchable_args(args)?;
    let rg_path = rg_path.expect("Just checked");

    let fifo = context.ensure_fifo("rg").await?;
    let fifo_path = Context::get_fifo_path("rg");
    kak.run_command_in_client(&format!("edit! -fifo {} *rg*", fifo_path.to_string_lossy()))
        .await?;

    let fifo = fifo.lock().await;
    let fifo_clone = fifo.try_clone()?;
    let fifo_clone2 = fifo.try_clone()?;

    let splitted_args: Vec<String> = args[0].split_args();
    all_args_start_with_dash(&splitted_args)?;

    let cwd = context.get_cwd().await.expect("Missing CWD");

    let mut command = Command::new(rg_path);
    let child = process::spawn(
        command
            .args(splitted_args)
            .arg("--color=always")
            .arg("--line-number")
            .arg("--smart-case")
            .current_dir(cwd)
            .stdout(Stdio::from(fifo_clone))
            .stderr(Stdio::from(fifo_clone2)),
    )?;

    process::supervise(child);
    Ok(())
}
