// Copyright 2020 Oxide Computer Company

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::process::exit;

use getopts::Options;
use tar::{Builder, EntryType, Header};

mod pkgmf;
use pkgmf::Entry;

mod repo;
use repo::Repository;

enum Source {
    ManifestProto(PathBuf, PathBuf, HashMap<String, String>),
    RepositoryPackages(PathBuf, Vec<String>),
}

struct Params {
    source: Source,
    tar: PathBuf,
    append: bool,
    excludes: Vec<String>,
}

fn parse_args() -> Params {
    let mut opts = Options::new();

    opts.optopt("r", "repository", "IPS repository directory (repo.redist)",
        "REPOSITORY_DIR");
    opts.optmulti("P", "package", "IPS package name (e.g., \"system/header\")",
        "PACKAGE_NAME");

    opts.optopt("p", "proto", "proto area from which the tar will \
        be populated", "PROTO_DIR");
    opts.optopt("m", "manifest", "IPS manifest file", "MANIFEST_FILE");
    opts.optmulti("d", "define", "variable replacement \"macros\"",
        "NAME=VALUE");

    opts.optflag("a", "append", "append to tar file (instead of \
        overwriting)");
    opts.optmulti("E", "exclude-path", "exclude manifest object path",
        "EXCLUDE_PATH");

    opts.optflag("", "help", "print usage information");

    let usage = || {
        let mut out = String::new();
        out.push_str("Usage: mf2tar -r REPOSITORY_DIR -P PACKAGE_NAME... \
            TARFILE\n");
        out.push_str("       mf2tar -m MANIFEST_FILE -p PROTO_DIR \
            TARFILE");
        println!("{}", opts.usage(&out));
    };

    let args: Vec<String> = std::env::args().skip(1).collect();
    let res = match opts.parse(args) {
        Ok(r) => r,
        Err(e) => {
            usage();
            println!("ERROR: {}", e);
            exit(1);
        }
    };

    let have = |n: &str| -> bool {
        res.opt_present(n)
    };

    if have("help") {
        usage();
        exit(0);
    }

    let source = if let Some(proto) = res.opt_str("proto") {
        if have("r") || have("P") {
            usage();
            println!("ERROR: -p, -m, & -d are exclusive with -r & -P");
            exit(1);
        }

        let proto = PathBuf::from(proto);
        let manifest = if let Some(m) = res.opt_str("manifest") {
            PathBuf::from(m)
        } else {
            usage();
            println!("ERROR: -p and -m must be specified together");
            exit(1);
        };

        let mut defines = HashMap::new();
        for dv in res.opt_strs("define").iter() {
            let t: Vec<_> = dv.splitn(2, '=').collect();
            if t.len() != 2 {
                usage();
                println!("ERROR: -d requires NAME=VALUE arguments");
                exit(1);
            }
            defines.insert(t[0].to_string(), t[1].to_string());
        }

        Source::ManifestProto(manifest, proto, defines)

    } else if let Some(repo) = res.opt_str("repository") {
        if have("p") || have("m") || have("d") {
            usage();
            println!("ERROR: -p, -m, & -d are exclusive with -r & -P");
            exit(1);
        }

        let repository = PathBuf::from(repo);
        let packages = res.opt_strs("package");

        Source::RepositoryPackages(repository, packages)

    } else {
        usage();
        println!("ERROR: must specify either -r & -P, or -p & -m");
        exit(1);
    };

    if res.free.len() != 1 {
        usage();
        println!("ERROR: must specify a single tar file for output");
        exit(1);
    }
    let tar = PathBuf::from(&res.free[0]);

    let mut excludes = res.opt_strs("exclude-path");
    excludes.sort();

    Params {
        source,
        tar,
        append: res.opt_present("append"),
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
    F: FnMut(&Entry) -> io::Result<()>,
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
                Entry::Include(name) => {
                    break Some(name);
                }
                Entry::Unknown(_) => {}
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

enum TarFileSource<'a> {
    Proto(&'a PathBuf),
    Repository(&'a Repository),
}

fn append_tar<W: io::Write>(
    builder: &mut Builder<W>,
    source: &TarFileSource,
    entry: &Entry,
    mtime: u64,
) -> io::Result<()> {
    match entry {
        Entry::Dir(dir) => {
            let mut header = Header::new_ustar();
            header.set_entry_type(EntryType::Directory);
            header.set_path(&dir.path)?;
            header.set_mode(0o755);
            header.set_mtime(mtime);
            header.set_cksum();

            builder.append(&header, &[] as &[u8])?;
            println!(" d {}", &dir.path);
            Ok(())
        }
        Entry::File(file) => {
            let mut header = Header::new_ustar();
            header.set_entry_type(EntryType::Regular);
            header.set_path(&file.path)?;
            header.set_mode(0o644);
            header.set_mtime(mtime);

            match source {
                TarFileSource::Proto(proto_dir) => {
                    let mut source_path = proto_dir.to_path_buf();
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

                    header.set_size(source_len);
                    header.set_cksum();
                    builder.append(&header, source.take(source_len))?;
                }
                TarFileSource::Repository(repo) => {
                    let buf = match repo.file(file.cname.as_ref().unwrap(),
                        file.chash.as_ref().unwrap())
                    {
                        Ok(v) => v,
                        Err(e) => {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                format!("file {}: {}", &file.path, e)));
                        }
                    };

                    header.set_size(buf.len() as u64);
                    header.set_cksum();
                    builder.append(&header, buf.as_slice())?;
                }
            };

            println!(" f {}", &file.path);
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
            println!(" l {} -> {}", &link.path, &link.target);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn main() {
    let params = parse_args();

    /*
     * Use a single mtime for all files in the archive.
     */
    let mtime = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut tar_builder = match &params.source {
        Source::ManifestProto(manifest, proto_area, defines) => {
            let (manifest_dir, manifest_file) = match prepare_manifest(&manifest) {
                Err(err) => {
                    eprintln!("Error preparing: {}", err);
                    exit(118);
                }
                Ok(state) => state,
            };

            let proto_dir = match prepare_proto(&proto_area) {
                Err(err) => {
                    eprintln!("Invalid proto area: {}", err);
                    exit(119);
                }
                Ok(t) => t,
            };
            let source = TarFileSource::Proto(&proto_dir);

            for (key, value) in defines.iter() {
                println!("'{}' => '{}'", key, value);
            }

            let mut tar_builder = match prepare_tar(&params.tar, params.append)
            {
                Err(err) => {
                    eprintln!("Error preparing tar: {}", err);
                    exit(85);
                }
                Ok(t) => t,
            };

            let proc_func = |entry: &Entry| {
                if let Some(path) = entry.get_path() {
                    if params
                        .excludes
                        .iter()
                        .find(|&comp| path.starts_with(comp))
                        .is_none()
                    {
                        append_tar(&mut tar_builder, &source, entry, mtime)?;
                    }
                }
                Ok(())
            };

            let res = iterate_items(
                &manifest,
                &manifest_dir,
                manifest_file,
                &defines,
                proc_func,
            );
            if let Err(e) = res {
                eprintln!("{}", e);
                exit(117);
            }

            tar_builder
        }
        Source::RepositoryPackages(repo_dir, package_names) => {
            let repo = match Repository::new(&repo_dir) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("ERROR: repository open: {}", e);
                    exit(103);
                }
            };
            let source = TarFileSource::Repository(&repo);

            let packages = match repo.scan() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("ERROR: repository scan: {}", e);
                    exit(104);
                }
            };

            let mut tar_builder = match prepare_tar(&params.tar, params.append)
            {
                Err(err) => {
                    eprintln!("Error preparing tar: {}", err);
                    exit(85);
                }
                Ok(t) => t,
            };

            for pn in package_names {
                let pkg = if let Some(pkg) = packages.get(pn) {
                    pkg
                } else {
                    eprintln!("ERROR: package \"{}\" not found in repository",
                        pn);
                    exit(100);
                };

                if pkg.versions.len() != 1 {
                    eprintln!("ERROR: package \"{}\" has {} versions, not 1",
                        pn, pkg.versions.len());
                    exit(101);
                }

                let mfest = match pkg.versions.first().unwrap().manifest() {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("ERROR: manifest load: {}", e);
                        exit(105);
                    }
                };

                for ent in mfest {
                    if let Some(path) = ent.get_path() {
                        if params.excludes
                            .iter()
                            .find(|&comp| path.starts_with(comp))
                            .is_none()
                        {
                            if let Err(e) = append_tar(&mut tar_builder,
                                &source, &ent, mtime)
                            {
                                eprintln!("ERROR: tar: {}", e);
                                exit(110);
                            }
                        }
                    }
                }
            }

            tar_builder
        }
    };

    if let Err(e) = tar_builder.finish() {
        eprintln!("ERROR: tar: {}", e);
        exit(97);
    }
}
