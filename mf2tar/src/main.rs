// Copyright 2020 Oxide Computer Company

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::iter::Iterator;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use clap::{App, Arg};
use tar::{Builder, EntryType, Header};

mod pkgmf;

struct Params {
    manifest: PathBuf,
    proto_area: PathBuf,
    tar: PathBuf,
    append: bool,
    defines: HashMap<String, String>,
    excludes: Vec<String>,
}

fn parse_args() -> Params {
    #[rustfmt::skip]
    let app = App::new("Manifest-to-tar")
        .about("Build a tarball based on IPS manifest listing")
        .usage("mf2tar [OPTIONS] -m <MANIFEST_FILE> -p <PROTO_DIR> <OUTPUT>")
        .arg(Arg::with_name("manifest")
            .short("m")
            .value_name("MANIFEST_FILE")
            .required(true)
            .takes_value(true)
            .display_order(1)
            .help("IPS manifest file"))
        .arg(Arg::with_name("proto_area")
            .short("p")
            .value_name("PROTO_DIR")
            .required(true)
            .takes_value(true)
            .display_order(2)
            .help("proto area from which the tar will be populated"))
        .arg(Arg::with_name("output")
            .value_name("OUTPUT")
            .required(true)
            .index(1)
            .help("Output tar file"))
        .arg(Arg::with_name("append")
            .short("a")
            .help("Append to tar file (instead of overwriting)"))
        .arg(Arg::with_name("define")
            .short("D")
            .value_name("name=value")
            .takes_value(true)
            .multiple(true)
            .validator(|x| {
                match x.split('=').count() {
                    2 => Ok(()),
                    _ => Err("must be in name=value format".to_string())
                }
            })
            .help("Variable replacement \"macros\""))
        .arg(Arg::with_name("exclude-path")
            .short("E")
            .value_name("PATH_PREFIX")
            .takes_value(true)
            .multiple(true)
            .help("Exclude manifest object path"));

    let matches = app.get_matches();

    let manifest = PathBuf::from(matches.value_of("manifest").unwrap());
    let proto_area = PathBuf::from(matches.value_of("proto_area").unwrap());
    let tar = PathBuf::from(matches.value_of("output").unwrap());

    let mut defines: HashMap<String, String> = HashMap::new();

    if let Some(define_vals) = matches.values_of("define") {
        for val in define_vals {
            // Validator should have already made this safe
            let mut fields = val.split('=');
            let key = fields.next().unwrap();
            let value = fields.next().unwrap();
            defines.insert(key.to_string(), value.to_string());
        }
    }

    let mut excludes: Vec<String> = match matches.values_of("exclude-path") {
        None => Vec::new(),
        Some(paths) => paths.map(|x| x.to_string()).collect(),
    };
    excludes.sort();

    Params {
        manifest,
        proto_area,
        tar,
        append: matches.is_present("append"),
        defines,
        excludes,
    }
}

fn prepare_manifest(manifest: &Path) -> io::Result<(PathBuf, File)> {
    let parent = manifest.parent().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidInput,
        "invalid manifest directory",
    ))?;
    let manifest_file = File::open(&manifest)?;

    Ok((parent.to_path_buf(), manifest_file))
}

fn prepare_proto(proto_dir: &Path) -> io::Result<PathBuf> {
    let cpath = proto_dir.canonicalize()?;

    if !cpath.is_dir() {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("not a directory: {}", proto_dir.to_str().unwrap_or("")),
        ))
    } else {
        Ok(cpath.to_path_buf())
    }
}

fn prepare_tar(tar_path: &Path, append: bool) -> io::Result<Builder<File>> {
    let mut tar_file = OpenOptions::new()
        .write(true)
        .read(append)
        .create(true)
        .truncate(!append)
        .open(tar_path)?;

    if append {
        let mut parser = tar::Archive::new(tar_file);
        let entries = parser.entries()?;
        let pos = entries.last().map_or(Ok(0u64), |last| -> io::Result<u64> {
            match last {
                Err(e) => Err(e),
                Ok(ent) => {
                    let size = ent.header().entry_size()?;
                    let next = ent.raw_file_position() + size;
                    let next_align = (next + (512 - 1)) & !(512 - 1);
                    Ok(next_align)
                }
            }
        })?;
        tar_file = parser.into_inner();
        tar_file.seek(SeekFrom::Start(pos))?;
    }

    // TODO: handle seeking to end (and indexing existing records) for append mode
    Ok(Builder::new(tar_file))
}

fn iterate_items<F>(
    manifest_path: &Path,
    manifest_dir: &PathBuf,
    manifest_file: File,
    defines: &HashMap<String, String>,
    mut process_func: F,
) -> io::Result<()>
where
    F: FnMut(&pkgmf::Entry) -> io::Result<()>,
{
    println!(
        "processing {} in {}",
        manifest_path.to_str().unwrap_or(""),
        manifest_dir.to_str().unwrap_or("")
    );

    // Buffer a file input (and strictly convert to line-separated utf8)
    let file_to_strings = |f: File| BufReader::new(f).lines().filter_map(|x| x.ok());
    // Handle $(variable) replacement with provided defines
    let replace = |name: &str| {
        let val = defines.get(name)?;
        Some(val.to_string())
    };

    // To avoid malicious manifests creating an infinite loop of includes, track them in a stack.
    let mut include_stack: Vec<(PathBuf, pkgmf::Reader<_, _>)> = Vec::new();

    let full_manifest_path = manifest_path.canonicalize()?;
    include_stack.push((
        full_manifest_path,
        pkgmf::Reader::new(file_to_strings(manifest_file), replace),
    ));

    while let Some((path, mut reader)) = include_stack.pop() {
        let include = loop {
            let next = reader.next();
            if next.is_none() {
                break None;
            }
            match next.unwrap() {
                pkgmf::Entry::Include(name) => {
                    break Some(name);
                }
                pkgmf::Entry::Unknown(_) => {}
                entry => {
                    if let Err(e) = process_func(&entry) {
                        eprintln!("{}", e);
                    }
                    //if let Err(e) = process_func(&entry)?;
                }
            }
        };

        if let Some(name) = include {
            include_stack.push((path, reader));

            let mut inc_path: PathBuf = manifest_dir.clone();
            inc_path.push(name);
            inc_path = inc_path.canonicalize()?;

            // Search for a match in the include stack to avoid infinite include loops
            if include_stack
                .iter()
                .any(|(path, _)| path.to_str().unwrap() == inc_path.to_str().unwrap())
            {
                let msg = format!(
                    "infinite include loop through {}",
                    &inc_path.to_str().unwrap_or("")
                );
                return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
            }

            let file = File::open(&inc_path)?;
            let new_reader = pkgmf::Reader::new(file_to_strings(file), replace);
            include_stack.push((inc_path, new_reader));
        }
    }
    Ok(())
}

fn append_tar<W: io::Write>(
    builder: &mut Builder<W>,
    proto_dir: &PathBuf,
    entry: &pkgmf::Entry,
) -> io::Result<()> {
    match entry {
        pkgmf::Entry::Dir(_dir) => {
            // TODO: handle directories to preserve any special permissions
            Ok(())
        }
        pkgmf::Entry::File(file) => {
            let mut source_path = proto_dir.clone();
            source_path.push(&file.path);

            let source = match File::open(&source_path) {
                Err(e) => {
                    return Err(io::Error::new(
                        e.kind(),
                        format!("{}: {}", e, &source_path.to_str().unwrap()),
                    ));
                }
                Ok(f) => f,
            };
            let meta = source.metadata()?;

            if !meta.file_type().is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{} is not a file", &source_path.to_str().unwrap()),
                ));
            }
            let source_len = meta.len();

            let mut header = Header::new_ustar();
            header.set_entry_type(EntryType::Regular);
            header.set_size(source_len);
            header.set_path(&file.path)?;
            header.set_mode(meta.permissions().mode());
            let modified = meta.modified()?;
            header.set_mtime(
                modified
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            header.set_cksum();

            builder.append(&header, source.take(source_len))?;
            println!("added file {}", &source_path.to_str().unwrap());
            Ok(())
        }
        pkgmf::Entry::Link(link) => {
            let mut header = Header::new_ustar();
            header.set_entry_type(EntryType::Symlink);
            header.set_size(0);
            header.set_path(&link.path)?;
            header.set_mtime(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            header.set_mode(0o777);
            header.set_link_name(&link.target)?;
            header.set_cksum();

            // TODO: handle symlinks which are too long
            builder.append(&header, io::empty())?;
            println!("added link {} -> {}", &link.path, &link.target);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn main() {
    let params = parse_args();

    let (manifest_dir, manifest_file) = match prepare_manifest(&params.manifest) {
        Err(err) => {
            eprintln!("Error preparing: {}", err);
            return;
        }
        Ok(state) => state,
    };

    let proto_dir = match prepare_proto(&params.proto_area) {
        Err(err) => {
            eprintln!("Invalid proto area: {}", err);
            return;
        }
        Ok(t) => t,
    };

    let mut tar_builder = match prepare_tar(&params.tar, params.append) {
        Err(err) => {
            eprintln!("Error preparing tar: {}", err);
            return;
        }
        Ok(t) => t,
    };

    for (key, value) in params.defines.iter() {
        println!("'{}' => '{}'", key, value);
    }

    let res = iterate_items(
        &params.manifest,
        &manifest_dir,
        manifest_file,
        &params.defines,
        |entry: &pkgmf::Entry| {
            if let Some(path) = entry.get_path() {
                if params
                    .excludes
                    .iter()
                    .find(|&comp| path.starts_with(comp))
                    .is_none()
                {
                    append_tar(&mut tar_builder, &proto_dir, entry)?;
                }
            }
            Ok(())
        },
    );
    if let Err(e) = res {
        eprintln!("{}", e);
    }
}
