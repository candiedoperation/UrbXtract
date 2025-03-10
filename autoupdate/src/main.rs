/*
    UrbXtract
    Copyright (C) 2025  Atheesh Thirumalairajan

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>. 
*/

use semver::Version;
use serde::{Deserialize, Serialize};
use std::{env, fs};

#[derive(Debug, Serialize, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub struct AutoUpdater {
    repo_owner: String,
    repo_name: String,
    current_version: Version,
    binary_name: String,
}

impl AutoUpdater {
    pub fn new(repo_owner: &str, repo_name: &str, current_version: &str, binary_name: &str) -> Self {
        Self {
            repo_owner: repo_owner.to_string(),
            repo_name: repo_name.to_string(),
            current_version: Version::parse(current_version).unwrap(),
            binary_name: binary_name.to_string(),
        }
    }

    pub async fn init_update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Get latest release from GitHub
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.repo_owner, self.repo_name
        );
        
        let client = reqwest::Client::builder()
            .user_agent("rust-autoupdate")
            .build()?;
            
        let response = client.get(&url).send().await?;
        let release: GitHubRelease = response.json().await?;
        
        // Compare versionsf
        let updatepkt_version = Version::parse(&release.tag_name.trim_end_matches("-")).unwrap();
        if updatepkt_version <= self.current_version {
            return Ok(());
        }
        
        // Find the appropriate asset
        let asset = release.assets.iter()
            .find(|a| a.name.contains(&self.binary_name))
            .ok_or("No matching binary found in release assets")?;
            
        // Download new binary
        let binary_data = client.get(&asset.browser_download_url)
            .send()
            .await?
            .bytes()
            .await?;
            
        // Construct Release Binary Path
        let mut current_path = env::current_exe().unwrap();
        current_path.pop();
        current_path.push(self.binary_name.clone());
        
        /* Write new Binary and Confirm */
        fs::write(current_path.as_path(), binary_data)?;
        println!("Updated UrbXtract from Version {} to {}", self.current_version, release.tag_name);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut updater = AutoUpdater::new(
        "candiedoperation",
        "UrbXtract",
        &urbxtract::app_version(),  // Gets version from core library
        &format!("{}-{}", "urbxtract", env::consts::OS)
    );
    
    updater.init_update().await?;
    println!("Ver 1.2.3 >= Ver 0.9.2: {}, {}", urbxtract::app_version(), Version::parse("0.2.3").unwrap() >= Version::parse("0.9.2").unwrap());
    Ok(())
}