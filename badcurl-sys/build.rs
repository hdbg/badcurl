use core::arch;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

const REPO_OWNER: &str = "yifeikong";
const REPO_NAME: &str = "curl-impersonate";

const COMPATIBLE_RELEASE: &str = "v0.6.1";
const GITHUB_API_VER: &str = "2022-11-28";

use reqwest::header::ACCEPT;
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use std::option::Option;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReleaseAsset {
    pub url: String,
    pub browser_download_url: String,
    pub id: i32,
    pub node_id: String,
    pub name: String,
    pub label: Option<String>,
    pub state: String,
    pub content_type: String,
    pub size: i32,
    pub download_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Release {
    assets: Vec<ReleaseAsset>,
}

fn get_release() -> Result<Release, reqwest::Error> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/tags/{}",
        REPO_OWNER, REPO_NAME, COMPATIBLE_RELEASE
    );
    let client = reqwest::blocking::Client::new();
    let req = client
        .get(url)
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", GITHUB_API_VER)
        .header(USER_AGENT, "curl-sys/0.1.0")
        .build()?;

    let resp = client.execute(req)?;

    let text = resp.text()?;


    let resp = serde_json::from_str(&text).unwrap();
    Ok(resp)
}

fn main() {
    println!("cargo:rerun-if-changed=lib.rs");
    println!("cargo:rerun-if-changed=build.rs");
    let target_triple_string = env::var("TARGET").unwrap();

    let target = target_lexicon::Triple::from_str(&target_triple_string).unwrap();

    use target_lexicon::OperatingSystem as OS;
    let release_target = match &target.operating_system {
        target_lexicon::OperatingSystem::Windows => {
            format!("{}-win32", target.architecture.into_str())
        }
        OS::MacOSX { .. } => format!("{}-macos", target.architecture.into_str()),
        OS::Linux { .. } => format!(
            "{}-linux-{}",
            target.architecture.into_str(),
            target.environment.into_str()
        ),
        _ => panic!("unsupported OS"),
    };

    let release_filename = format!(
        "libcurl-impersonate-{}.{}.tar.gz",
        COMPATIBLE_RELEASE, release_target
    );

    let github_release = get_release().expect("Failed to parse release");

    let asset = github_release
        .assets
        .iter()
        .find(|asset| asset.name == release_filename)
        .expect("Failed to find asset");


    let url = &asset.browser_download_url;

    let mut archive_buffer = Vec::with_capacity(asset.size as usize);
    let mut resp = reqwest::blocking::get(url).expect("Failed to download");
    std::io::copy(&mut resp, &mut archive_buffer).expect("Failed to write file");

    let tar_buffer = flate2::read::GzDecoder::new(archive_buffer.as_slice());

    let mut archive = tar::Archive::new(tar_buffer);
    let cargo_temp = env::var("OUT_DIR").unwrap();

    let target_archive_path = Path::new(&cargo_temp);
    archive.unpack(target_archive_path).expect("Failed to unpack");


    // suffix for static library file
    let library_suffix = ".a";


    let lib_name = format!("libcurl");

    // link
    println!("cargo:rustc-link-search={}", target_archive_path.display());

    println!(
        "cargo:rustc-link-lib=dylib={}",
        lib_name
    );
}
