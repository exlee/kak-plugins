use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use nix::fcntl::{FcntlArg, OFlag, fcntl};
use tokio::io;
use tokio::sync::Mutex;

pub struct Context {
    pub(crate) tools: HashMap<String, Option<PathBuf>>,
    pub(crate) fifos: tokio::sync::RwLock<HashMap<String, Arc<Mutex<File>>>>,
    cwd: Mutex<Option<PathBuf>>,
}

impl Context {
    pub async fn set_cwd(&self, path: PathBuf) {
        let mut cwd = self.cwd.lock().await;
        (*cwd) = Some(path);
    }
    pub async fn get_cwd(&self) -> Option<PathBuf> {
        self.cwd.lock().await.clone()
    }
    pub fn get_tool(&self, key: &str) -> Option<PathBuf> {
        self.tools.get(key).cloned().unwrap_or(None)
    }
    pub async fn ensure_fifo(&self, key: &str) -> io::Result<Arc<Mutex<File>>> {
        let path = Self::get_fifo_path(key);

        let _ = std::fs::remove_file(&path);
        nix::unistd::mkfifo(&path, nix::sys::stat::Mode::from_bits(0o600).unwrap())?;

        let path_clone = path.clone();
        let file = tokio::task::spawn_blocking(move || {
            std::fs::OpenOptions::new()
                .write(true)
                .read(true)
                .open(path_clone)
        })
        .await
        .map_err(std::io::Error::other)??;

        let mut flags = OFlag::from_bits_truncate(fcntl(&file, FcntlArg::F_GETFL)?);
        flags.remove(OFlag::O_NONBLOCK);
        fcntl(&file, FcntlArg::F_SETFL(flags))?;

        let fifo_arc = Arc::new(Mutex::new(file));
        {
            let mut writable_fifos = self.fifos.write().await;
            writable_fifos.insert(key.into(), fifo_arc.clone());
        }
        Ok(fifo_arc)
    }
    pub fn get_fifo_path(key: &str) -> PathBuf {
        Path::new("/tmp").join(format!("kak-univ-{}", key))
    }

    pub fn new(required_tools: &[&str]) -> Result<Self, String> {
        let mut tools = HashMap::new();
        let fifos = tokio::sync::RwLock::new(HashMap::new());

        for &tool in required_tools {
            tools.insert(tool.to_string(), find_binary(tool));
        }

        Ok(Self {
            tools,
            fifos,
            cwd: Mutex::new(None),
        })
    }
}

fn find_binary(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).find_map(|dir| {
            let full_path = dir.join(name);
            if full_path.is_file() && is_executable(&full_path) {
                Some(full_path)
            } else {
                None
            }
        })
    })
}

fn is_executable(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}
