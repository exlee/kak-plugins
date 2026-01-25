use std::{io, path::PathBuf, process::Stdio, sync::Arc};

use tokio::process::Command;

use crate::{handle_context::Context, kakoune::Kakoune, string_ext::SplitArgs};

pub async fn process(context: Arc<Context>, args: &[String], kak: &Kakoune) -> io::Result<()> {
    let rg_path = context.get_tool("rg");

    if rg_path.is_none() {
        kak.run_command("echo -debug Error: rg is missing").await?;
        Err(io::Error::other("rg is missing from path, can't process"))?;
    }
    let rg_path = rg_path.unwrap();
    if args.len() < 2 {
        Err(io::Error::other("Not enough arguments"))?;
    }

    let fifo = context.ensure_fifo("buffer-search").await?;
    let fifo_path = Context::get_fifo_path("buffer-search");

    kak.run_command_in_client(&format!(
        "edit! -fifo {} *buffer-search*",
        fifo_path.to_string_lossy()
    ))
    .await?;

    let fifo = fifo.lock().await;
    let fifo_clone = fifo.try_clone()?;

    let input_file = PathBuf::from(&args[0]);
    let parent_dir = input_file.parent().expect("No parent dir found");
    let search_input = &args[1];

    let mut cmd = Command::new(rg_path)
        .args(search_input.split_args())
        .arg("--color=always")
        .arg("--smart-case")
        .arg("--line-number")
        .arg(&input_file)
        .current_dir(parent_dir)
        .stdout(Stdio::from(fifo_clone))
        .spawn()?;

    cmd.wait().await?;

    Ok(())
}
