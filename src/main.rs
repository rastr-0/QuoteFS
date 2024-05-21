use std::error::Error;
use std::ffi::OsStr;
use std::process::Command as process_command;
use std::time::{Duration, UNIX_EPOCH};
use std::string::String;
use clap::{Arg, ArgAction, Command, crate_version};
use fuser::{FileAttr, Filesystem, FileType, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request};
use libc::ENOENT;

const TTL: Duration = Duration::from_secs(5);

const QUOTE_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH,
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    // owner can do everything, groups and others can only search
    perm: 0o755,
    // figure out why here should be 2
    nlink: 2,
    uid: 301,
    // root
    gid: 0,
    rdev: 0,
    flags: 0,
    blksize: 512
};

fn get_quote() -> Result<(String, u64), Box<dyn Error>> {
    // run python script for getting a random quote
    let output = process_command::new("python3")
        .arg("get_random_quote.py")
        .output()?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut quote = output_str.trim().to_string();
        quote.push('\n');
        let size = quote.len() as u64;

        Ok((quote, size))
    } else {
        Err("Failed to execute command".into())
    }
}

fn generate_file_attr(file_size: u64) -> FileAttr {
    FileAttr {
        ino: 2,
        size: file_size,
        blocks: 0,
        atime: UNIX_EPOCH,
        mtime: UNIX_EPOCH,
        ctime: UNIX_EPOCH,
        crtime: UNIX_EPOCH,
        kind: FileType::RegularFile,
        // owner can do everything, groups and others can only search
        perm: 0o755,
        // figure out why here should be 2
        nlink: 2,
        uid: 301,
        // root
        gid: 0,
        rdev: 0,
        flags: 0,
        blksize: 512
    }
}

struct QuoteFs {
    file_size: u64,
    file_content: String
}

impl QuoteFs {
    // default values, used when content of the file is not used
    fn new() -> QuoteFs {
        QuoteFs {
            file_size: 25_u64,
            file_content: "This is a default content".to_string()
        }
    }
    // every time user wants to interact with the content it randomly generates
    fn update_file(&mut self) -> Result<(), Box<dyn Error>> {
        let (quote, size) = get_quote()?;
        self.file_content = quote;
        self.file_size = size;

        Ok(())
    }
}

// Filesystem is a trait to provide the userspace file system via FUSE technology
impl Filesystem for QuoteFs {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent == 1 && name.to_str() == Some("random_quote.txt") {
            self.update_file().expect("Python script was not run");
            reply.entry(&TTL, &generate_file_attr(self.file_size), 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        match ino {
            1 => reply.attr(&TTL, &QUOTE_DIR_ATTR),
            2 => reply.attr(&TTL, &generate_file_attr(self.file_size)),
            _ => reply.error(ENOENT)
        }
    }

    fn read(&mut self, _req: &Request<'_>, ino: u64, _fh: u64,
            offset: i64, _size: u32, _flags: i32,
            _lock_owner: Option<u64>, reply: ReplyData) {
        if ino == 2 {
            reply.data(&self.file_content.as_bytes()[offset as usize..]);
        } else { reply.error(ENOENT); }
    }

    fn readdir(&mut self, _req: &Request<'_>, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let entries = vec![
            (1, FileType::Directory, "."),
            (2, FileType::Directory, ".."),
            (3, FileType::RegularFile, "random_quote.txt")
        ];

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                break;
            }
        }
        reply.ok();
    }
}

fn main() {
    let matches = Command::new("quote")
        .version(crate_version!())
        .author("rastr-0")
        .arg(
            Arg::new("MOUNT_POINT")
                .required(true)
                .index(1)
                .help("Automatically unmount on process exit")
        )
        .arg(
            Arg::new("auto_unmount")
                .long("auto_unmount")
                .action(ArgAction::SetTrue)
                .help("Automatically unmount on process exit")
        )
        .arg(
            Arg::new("allow_root")
                .long("allow_root")
                .action(ArgAction::SetTrue)
                .help("Allow root user to access filesystem")
        )
        .get_matches();

    let mount_point = matches.get_one::<String>("MOUNT_POINT").unwrap();
    let mut options = vec![MountOption::RO, MountOption::FSName("quote".to_string())];
    if matches.get_flag("auto_unmount") {
        options.push(MountOption::AutoUnmount);
    }
    if matches.get_flag("allow_root") {
        options.push(MountOption::AllowRoot);
    }

    let quote_fs = QuoteFs::new();

    fuser::mount2(quote_fs, mount_point, &options).unwrap();
}
