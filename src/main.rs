use clap::Parser;
use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry,
    Request,
};
use koibumi_base32 as base32;
use libc::ENOENT;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha1, DEFAULT_STEP};

#[derive(Parser, Debug)]
#[clap(version, long_about = None)]
struct Args {
    #[clap(short, long)]
    mountpoint: PathBuf,

    #[clap(short, long)]
    filename: String,

    #[clap(short, long)]
    username: String,

    #[clap(short, long)]
    secret: String,

    #[clap(short, long)]
    noubc: bool,
}

const TTL: Duration = Duration::from_secs(1);

const ROOT_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

fn file_attr(size: u64) -> FileAttr {
    let now = SystemTime::now();
    FileAttr {
        ino: 2,
        size,
        blocks: 1,
        atime: now,
        mtime: now,
        ctime: now,
        crtime: now,
        kind: FileType::RegularFile,
        perm: 0o666,
        nlink: 1,
        uid: 0,
        gid: 0,
        rdev: 0,
        flags: 0,
        blksize: 512,
    }
}

struct AuthUserPass {
    filename: String,
    username: String,
    secret: String,
    file_attr: FileAttr,
}

impl AuthUserPass {
    fn new(filename: String, username: String, secret: String) -> AuthUserPass {
        let size: u64 = (username.len() + 6 + 2).try_into().unwrap(); // 6 for password + 2 newlines
        AuthUserPass {
            filename,
            username,
            secret,
            file_attr: file_attr(size),
        }
    }
}

impl Filesystem for AuthUserPass {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent == 1 && name.to_str() == Some(&self.filename) {
            reply.entry(&TTL, &self.file_attr, 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match ino {
            1 => reply.attr(&TTL, &ROOT_DIR_ATTR),
            2 => reply.attr(&TTL, &self.file_attr),
            _ => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        if ino == 2 {
            // main TOTP code
            let seconds: u64 = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let totp = totp_custom::<Sha1>(
                DEFAULT_STEP,
                6,
                &base32::decode(&self.secret.trim().to_lowercase()).unwrap(),
                seconds,
            );
            let content = format!("{}\n{}\n", &self.username, &totp);
            reply.data(&content.as_bytes()[offset as usize..]);
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let entries = vec![
            (1, FileType::Directory, "."),
            (1, FileType::Directory, ".."),
            (2, FileType::RegularFile, &self.filename),
        ];

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            // i + 1 means the index of the next entry
            if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                break;
            }
        }
        reply.ok();
    }
}

#[allow(clippy::needless_collect)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut options = vec![
        MountOption::RO,
        MountOption::Sync,
        MountOption::AutoUnmount,
        MountOption::AllowRoot,
        MountOption::FSName(args.filename.clone()),
    ];
    if args.noubc {
        // https://github.com/osxfuse/osxfuse/wiki/Mount-options#noubc
        options.push(MountOption::CUSTOM("noubc".to_string()));
    }
    let fs = AuthUserPass::new(args.filename, args.username, args.secret);
    fuser::mount2(fs, args.mountpoint, &options)?;
    Ok(())
}
