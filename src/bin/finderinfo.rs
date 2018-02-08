#[macro_use]
extern crate cfg_if;
extern crate docopt;
extern crate finder_info;
extern crate hex;
#[cfg(all(feature = "xattr", target_os = "macos"))]
extern crate libc;
#[macro_use]
extern crate serde_derive;

use std::io;
use std::fs;

use docopt::Docopt;
use hex::FromHex;

use finder_info::{FinderInfoFile, FinderInfoFolder, OSType};

const USAGE: &'static str = "
        FinderInfo utility.

        Usage:
        finderinfo read <path>
        finderinfo parse-hex (-d | -f) <hex-data>
        finderinfo read-filetype <path>
        finderinfo write-filetype <path> <value>
        finderinfo (-h | --help)

        Options:
        -h --help   Show this screen.
        -d          Read FinderInfo as directory
        -f          Read FinderInfo as file
        ";

#[derive(Debug, Deserialize)]
struct Args {
    arg_path: String,
    arg_value: String,
    arg_hex_data: String,
    cmd_read: bool,
    cmd_read_filetype: bool,
    cmd_write_filetype: bool,
    cmd_parse_hex: bool,
    flag_d: bool,
}

#[derive(Clone, Debug)]
enum FinderInfo {
    File(FinderInfoFile),
    Directory(FinderInfoFolder),
}

cfg_if! {
    if #[cfg(all(feature = "xattr", target_os = "macos"))] {
        use std::ffi::CString;
        const FINDERINFO_XATTR_NAME: &'static str = "com.apple.FinderInfo";

        fn read_finderinfo_from_path(path: &str) -> io::Result<FinderInfo> {
            let path_cstring = CString::new(path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let xattr_name =
                CString::new(FINDERINFO_XATTR_NAME).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let mut buf = [0u8; 32];

            let ret = unsafe {
                libc::getxattr(
                    path_cstring.as_ptr(),
                    xattr_name.as_ptr(),
                    buf.as_mut_ptr() as *mut libc::c_void,
                    buf.len(),
                    0, /* position */
                    0, /* flags */
                )
            };
            if ret == -1 {
                return Err(io::Error::last_os_error());
            } else if ret != 32 {
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    format!("only received {:?} bytes", ret),
                ));
            }

            let is_dir = fs::metadata(path)?.is_dir();

            Ok(if is_dir {
                FinderInfo::Directory(FinderInfoFolder::read(&mut io::Cursor::new(buf))?)
            } else {
                FinderInfo::File(FinderInfoFile::read(&mut io::Cursor::new(buf))?)
            })
        }

        fn write_finderinfo_to_path(path: &str, fi: FinderInfo) -> io::Result<()> {
            let mut cursor = io::Cursor::new(vec![]);
            match fi {
                FinderInfo::File(fi) => fi.write(&mut cursor)?,
                FinderInfo::Directory(fi) => fi.write(&mut cursor)?,
            }
            let bytes = cursor.into_inner();
            let path_cstring = CString::new(path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let xattr_name =
                CString::new(FINDERINFO_XATTR_NAME).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let ret = unsafe {
                libc::setxattr(
                    path_cstring.as_ptr(),
                    xattr_name.as_ptr(),
                    bytes.as_ptr() as *const libc::c_void,
                    bytes.len(),
                    0, /* position */
                    0, /* flags */
                )
            };
            if ret == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
    } else {
        fn read_finderinfo_from_path(_path: &str) -> io::Result<FinderInfo> {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "xattr i/o not supported",
            ))
        }

        fn write_finderinfo_to_path(_path: &str, _fi: FinderInfo) -> io::Result<()> {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "xattr i/o not supported",
            ))
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    if args.cmd_parse_hex {
        let buf = Vec::from_hex(&args.arg_hex_data).expect("invalid hexadecimal string");
        let finder_info = if args.flag_d {
            FinderInfo::Directory(
                FinderInfoFolder::read(&mut io::Cursor::new(buf)).expect("Read failed"),
            )
        } else {
            FinderInfo::File(FinderInfoFile::read(&mut io::Cursor::new(buf)).expect("Read failed"))
        };
        println!("{:#?}", finder_info);
    }
    if args.cmd_read || args.cmd_read_filetype {
        println!("Attempting to read FinderInfo from {:?}", args.arg_path);
        let finder_info = read_finderinfo_from_path(&args.arg_path);
        if args.cmd_read {
            println!("{:#?}", finder_info);
        }
        if args.cmd_read_filetype {
            match finder_info {
                Ok(FinderInfo::File(fi)) => println!("file type: {:?}", fi.file_info.fileType),
                _ => panic!("Not found"),
            }
        }
    }
    if args.cmd_write_filetype {
        println!("Attempting to read FinderInfo from {:?}", args.arg_path);
        let finder_info = read_finderinfo_from_path(&args.arg_path).unwrap_or_else(|_| {
            if fs::metadata(&args.arg_path).unwrap().is_dir() {
                panic!("attempted to set filetype on a directory")
            }
            FinderInfo::File(FinderInfoFile::default())
        });
        match finder_info {
            FinderInfo::File(mut fi) => {
                println!("Original filetype: {:?}", fi.file_info.fileType);
                let bytes = args.arg_value.into_bytes();
                if bytes.len() != 4 {
                    panic!("file type {:?} must be 4 bytes", bytes);
                }
                let new_filetype = OSType([bytes[0], bytes[1], bytes[2], bytes[3]]);
                println!("New filetype: {:?}", new_filetype);
                fi.file_info.fileType = new_filetype;

                write_finderinfo_to_path(&args.arg_path, FinderInfo::File(fi)).unwrap();
                println!("Successfully wrote FinderInfo!");
            }
            FinderInfo::Directory(fi) => panic!("target is not a file! {:?}", fi),
        }
    }
}
