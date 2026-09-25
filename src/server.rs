use std::fs::{self, Metadata};
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixStream;
use std::path::Path;

use nix::sys::socket::{getsockopt, sockopt::PeerCredentials};
use nix::unistd::Uid;

/// 目录属主为 root、组为目标用户主组、权限 0770。
/// 0770 让目标用户能在目录里创建自己的 socket；0750 没有组写权限，用户进程无法 bind。
pub const RUNTIME_DIR_MODE: u32 = 0o770;
pub const SOCKET_MODE: u32 = 0o600;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathFacts {
    pub is_symlink: bool,
    pub dir_uid: u32,
    pub dir_gid: u32,
    pub dir_mode: u32,
    pub sock_uid: u32,
    pub sock_mode: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    Symlink,
    Directory,
    Socket,
}

pub fn validate_runtime_path(
    facts: PathFacts,
    target_uid: u32,
    target_gid: u32,
) -> Result<(), PathError> {
    if facts.is_symlink {
        return Err(PathError::Symlink);
    }
    if facts.dir_uid != 0 || facts.dir_gid != target_gid || facts.dir_mode != RUNTIME_DIR_MODE {
        return Err(PathError::Directory);
    }
    if facts.sock_uid != target_uid || facts.sock_mode != SOCKET_MODE {
        return Err(PathError::Socket);
    }
    Ok(())
}

pub fn facts_from_paths(dir: &Path, socket: &Path) -> std::io::Result<PathFacts> {
    let dir_meta = fs::symlink_metadata(dir)?;
    let sock_meta = fs::symlink_metadata(socket)?;
    Ok(PathFacts {
        is_symlink: dir_meta.file_type().is_symlink() || sock_meta.file_type().is_symlink(),
        dir_uid: dir_meta.uid(),
        dir_gid: dir_meta.gid(),
        dir_mode: mode_bits(&dir_meta),
        sock_uid: sock_meta.uid(),
        sock_mode: mode_bits(&sock_meta),
    })
}

fn mode_bits(meta: &Metadata) -> u32 {
    meta.mode() & 0o777
}

pub fn peer_uid(stream: &UnixStream) -> std::io::Result<u32> {
    let creds = getsockopt(stream, PeerCredentials).map_err(std::io::Error::other)?;
    Ok(creds.uid())
}

pub fn peer_is_target(stream: &UnixStream, target_uid: u32) -> bool {
    peer_uid(stream).ok() == Some(target_uid)
}

pub fn current_uid() -> u32 {
    Uid::current().as_raw()
}
