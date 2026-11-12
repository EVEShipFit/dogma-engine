//! The cached `sde.dat` and `names.dat`.

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use base64::Engine;
use esf_data::{MIN_SDE_VERSION, Names, Sde};
use serde::Deserialize;
use sha2::{Digest, Sha512};

use crate::Game;

const PACKAGE_URL: &str = "https://registry.npmjs.org/@eveshipfit/sde";
const CHECK_EVERY: Duration = Duration::from_secs(24 * 60 * 60);
const CHECK_TIMEOUT: Duration = Duration::from_secs(10);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);
const STALE_PARTIAL: Duration = Duration::from_secs(60 * 60);
const FILES: [&str; 2] = ["sde.dat", "names.dat"];

#[derive(Deserialize)]
struct Packument {
    versions: HashMap<String, PackageVersion>,
}

#[derive(Deserialize)]
struct PackageVersion {
    dist: Dist,
}

#[derive(Deserialize)]
struct Dist {
    tarball: String,
    integrity: String,
}

/// The game data from the cache, downloading the newest SDE when due.
pub fn load(cache_dir: &Path) -> Result<Game, String> {
    let dir = cached(cache_dir)?;
    let loaded = read(&dir.join("sde.dat"))
        .and_then(|sde| Game::new(sde, Some(read(&dir.join("names.dat"))?)));
    match &loaded {
        Ok(_) => prune(cache_dir, &dir),
        Err(_) => {
            let _ = fs::remove_dir_all(&dir);
        }
    }
    loaded
}

/// The bytes of a file.
pub fn read(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

/// The directory holding the cached `sde.dat` and `names.dat`.
fn cached(cache_dir: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(cache_dir).map_err(|error| format!("{}: {error}", cache_dir.display()))?;
    let marker = cache_dir.join("current");
    let current = fs::read_to_string(&marker)
        .ok()
        .map(|version| version.trim().to_string())
        .filter(|version| parse_version(version).is_some() && has_files(&cache_dir.join(version)));

    let fresh = fs::metadata(&marker)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age < CHECK_EVERY);

    if let Some(version) = &current
        && fresh
    {
        return Ok(cache_dir.join(version));
    }

    let version = match download_newest(cache_dir) {
        Ok(version) => version,
        Err(error) => match current {
            Some(version) => {
                eprintln!("esf-mcp: could not check for a newer SDE ({error}); using {version}");
                version
            }
            None => return Err(format!("could not download the SDE: {error}")),
        },
    };
    fs::write(&marker, &version).map_err(|error| format!("{}: {error}", marker.display()))?;
    Ok(cache_dir.join(version))
}

/// Remove every cached version but `keep`, and downloads abandoned long ago.
fn prune(cache_dir: &Path, keep: &Path) {
    for entry in fs::read_dir(cache_dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path == keep || !path.is_dir() {
            continue;
        }
        let partial = entry.file_name().to_string_lossy().contains(".partial-");
        let age = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| SystemTime::now().duration_since(modified).ok());
        if !partial || age.is_some_and(|age| age > STALE_PARTIAL) {
            let _ = fs::remove_dir_all(path);
        }
    }
}

fn has_files(dir: &Path) -> bool {
    FILES.iter().all(|file| dir.join(file).is_file())
}

/// `major.minor.patch`, without a pre-release.
fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.split('.').map(|part| part.parse::<u64>().ok());
    let parsed = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(parsed)
}

fn agent(timeout: Duration) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .build()
        .into()
}

fn download_newest(cache_dir: &Path) -> Result<String, String> {
    let packument: Packument = agent(CHECK_TIMEOUT)
        .get(PACKAGE_URL)
        .header("Accept", "application/vnd.npm.install-v1+json")
        .call()
        .and_then(|mut response| response.body_mut().read_to_string())
        .map_err(|error| error.to_string())
        .and_then(|body| serde_json::from_str(&body).map_err(|error| error.to_string()))?;

    let (version, package) = packument
        .versions
        .into_iter()
        .filter_map(|(version, package)| Some((parse_version(&version)?, version, package)))
        .filter(|((major, _, _), _, _)| *major >= MIN_SDE_VERSION as u64)
        .max_by_key(|(parsed, _, _)| *parsed)
        .map(|(_, version, package)| (version, package))
        .ok_or_else(|| format!("no version {MIN_SDE_VERSION}.x or newer is published"))?;

    let target = cache_dir.join(&version);
    if has_files(&target) {
        return Ok(version);
    }

    eprintln!("esf-mcp: downloading SDE {version}");
    let mut tarball = Vec::new();
    agent(DOWNLOAD_TIMEOUT)
        .get(&package.dist.tarball)
        .call()
        .map_err(|error| error.to_string())?
        .into_body()
        .into_reader()
        .read_to_end(&mut tarball)
        .map_err(|error| error.to_string())?;
    check_integrity(&tarball, &package.dist.integrity)?;

    let staging = cache_dir.join(format!("{version}.partial-{}", std::process::id()));
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging).map_err(|error| format!("{}: {error}", staging.display()))?;
    let result =
        extract(tarball.as_slice(), &staging).and_then(|()| match fs::rename(&staging, &target) {
            Ok(()) => Ok(()),
            Err(_) if has_files(&target) => Ok(()),
            Err(error) => Err(format!("{}: {error}", target.display())),
        });
    let _ = fs::remove_dir_all(&staging);
    result.map(|()| version)
}

/// Check bytes against an npm `sha512-<base64>` integrity.
fn check_integrity(bytes: &[u8], integrity: &str) -> Result<(), String> {
    let expected = integrity
        .strip_prefix("sha512-")
        .ok_or_else(|| format!("unsupported integrity {integrity}"))?;
    let actual = base64::engine::general_purpose::STANDARD.encode(Sha512::digest(bytes));
    match actual == expected {
        true => Ok(()),
        false => Err("the download does not match its integrity".to_string()),
    }
}

fn extract(tarball: impl Read, staging: &Path) -> Result<(), String> {
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(tarball));
    let mut files: HashMap<&str, Vec<u8>> = HashMap::new();
    for entry in archive.entries().map_err(|error| error.to_string())? {
        let mut entry = entry.map_err(|error| error.to_string())?;
        let path = entry
            .path()
            .map_err(|error| error.to_string())?
            .into_owned();
        let Some(file) = FILES
            .iter()
            .find(|file| path == Path::new("package/dist").join(file))
        else {
            continue;
        };
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        files.insert(file, bytes);
    }

    let (Some(sde), Some(names)) = (files.get("sde.dat"), files.get("names.dat")) else {
        return Err(format!("the package lacks {FILES:?}"));
    };
    let sde = Sde::new(sde).map_err(|error| format!("sde.dat: {error}"))?;
    let names = Names::new(names).map_err(|error| format!("names.dat: {error}"))?;
    if sde.build_number() != names.build_number() {
        return Err("sde.dat and names.dat are of different builds".to_string());
    }

    for (file, bytes) in &files {
        fs::write(staging.join(file), bytes).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_versions() {
        assert_eq!(parse_version("14.3687575.0"), Some((14, 3687575, 0)));
        assert_eq!(parse_version("14.3678022.0-singularity.92"), None);
        assert_eq!(parse_version("../14.1.0"), None);
        assert_eq!(parse_version("14.1"), None);
    }

    #[test]
    fn checks_integrity() {
        let integrity = "sha512-7iaw3Ur350mqGo7jwQrpkj9hiYB3Lkc/iBml1JQODbJ6wYX4oOHV+E+IvIh/1nsUNzLDBMxfqa2Ob1f1ACio/w==";
        assert_eq!(check_integrity(b"test", integrity), Ok(()));
        assert!(check_integrity(b"tset", integrity).is_err());
        assert!(check_integrity(b"test", "sha1-abc").is_err());
    }
}
