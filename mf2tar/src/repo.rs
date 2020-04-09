use digest::Digest;
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::fs::{metadata, read_dir, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use super::pkgmf;

use pkgmf::Entry;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn read_file<P: AsRef<Path>>(p: P) -> Result<Vec<u8>> {
    let f = File::open(p.as_ref())?;
    let mut r = BufReader::new(f);
    let mut buf = Vec::new();
    r.read_to_end(&mut buf)?;
    Ok(buf)
}

fn hash_buf(buf: &[u8]) -> String {
    let mut digest = sha1::Sha1::new();
    digest.input(buf);
    let mut out = String::new();
    for oct in digest.result().iter() {
        out.push_str(&format!("{:02x}", oct));
    }
    out
}

#[derive(Debug)]
pub struct Repository {
    file: PathBuf,
    pkg: PathBuf,
}

#[derive(Debug)]
pub struct Version<'a> {
    version: String,
    file: PathBuf,
    repo: &'a Repository,
}

impl Version<'_> {
    pub fn manifest(&self) -> Result<Box<dyn Iterator<Item = Entry>>> {
        let f = File::open(&self.file)?;
        let data = BufReader::new(f).lines().filter_map(|x| x.ok());
        fn replace(name: &str) -> Option<String> {
            panic!("unexpected expansion in repository manifest: {}", name);
        }
        Ok(Box::new(pkgmf::Reader::new(data, replace)))
    }
}

#[derive(Debug)]
pub struct Package<'a> {
    pub name: String,
    pub versions: Vec<Version<'a>>,
}

impl Repository {
    pub fn file(&self, cname: &str, chash: &str) -> Result<Vec<u8>> {
        let mut p = self.file.clone();
        p.push(&cname[0..=1]);
        p.push(&cname);

        let buf = read_file(&p)?;

        let outer_hash = hash_buf(&buf);
        if &outer_hash != chash {
            return Err(format!("hash mismatch: {} != expected {}", outer_hash,
                chash).into());
        }

        let mut gunzip = GzDecoder::new(buf.as_slice());
        let mut rawbuf: Vec<u8> = Vec::new();
        gunzip.read_to_end(&mut rawbuf)?;
        let inner_hash = hash_buf(&rawbuf);
        if &inner_hash != cname {
            return Err(format!("hash mismatch: {} != expected {}", inner_hash,
                cname).into());
        }

        Ok(rawbuf)
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Result<Repository> {
        let root = path.as_ref().to_path_buf();

        let mut file = root.clone();
        file.push("file");
        if !metadata(&file)?.is_dir() {
            return Err(format!("{} is not a directory", file.display()).into());
        }

        let mut pkg = root.clone();
        pkg.push("pkg");
        if !metadata(&pkg)?.is_dir() {
            return Err(format!("{} is not a directory", pkg.display()).into());
        }

        Ok(Repository { file, pkg })
    }

    pub fn scan(&self) -> Result<HashMap<String, Package>> {
        let mut pkgs = HashMap::new();

        for p in read_dir(&self.pkg)? {
            let p = p?;

            if !p.file_type()?.is_dir() {
                continue;
            }

            let name = if let Some(name) = p.file_name().to_str() {
                percent_encoding::percent_decode_str(&name)
                    .decode_utf8()?
                    .to_string()
            } else {
                continue;
            };

            let mut versions = Vec::new();

            for file in read_dir(&p.path())? {
                let file = file?;

                let version = if let Some(name) = file.file_name().to_str() {
                    percent_encoding::percent_decode_str(&name)
                        .decode_utf8()?
                        .to_string()
                } else {
                    continue;
                };

                let file = file.path();

                versions.push(Version {
                    version,
                    file,
                    repo: &self,
                });
            }

            pkgs.insert(name.clone(), Package { name, versions });
        }

        Ok(pkgs)
    }
}
