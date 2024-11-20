use super::types::*;
use anyhow::Result;
use colored::*;
use reqwest;

pub async fn get_package_info(name: &str) -> Result<Option<NpmPackageInfo>> {
    let client = reqwest::Client::new();
    let url = format!("https://registry.npmjs.org/{}", name);
    let response = client.get(&url).send().await?;

    if response.status().is_success() {
        Ok(Some(response.json().await?))
    } else {
        Ok(None)
    }
}

pub fn display_package_info(info: &NpmPackageInfo) {
    println!("\nNpm Package Information:");
    println!("  Name: {}", info.name.green());
    if let Some(desc) = &info.description {
        println!("  Description: {}", desc);
    }
    println!(
        "  Latest Version: {}",
        info.dist_tags.as_ref().unwrap().get("latest").unwrap()
    );

    if let Some(dist_tags) = &info.dist_tags {
        println!("\n  Available Tags:");
        for (tag, version) in dist_tags {
            println!("    {}: {}", tag, version);
        }
    }

    if let Some(license) = &info.license {
        println!("  License: {}", license);
    }
    if let Some(homepage) = &info.homepage {
        println!("  Homepage: {}", homepage);
    }
    if let Some(author) = &info.author {
        if let Some(name) = &author.name {
            print!("  Author: {}", name);
            if let Some(email) = &author.email {
                print!(" <{}>", email);
            }
            println!();
        }
    }
    if let Some(keywords) = &info.keywords {
        println!("  Keywords: {}", keywords.join(", "));
    }
}
