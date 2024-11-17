// use anyhow::Result;
// use std::path::PathBuf;
// use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
// #[derive(PartialEq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOS
        } else {
            Platform::Linux
        }
    }

    // pub fn bin_path() -> PathBuf {
    //     match Self::current() {
    //         Platform::Windows => {
    //             let local_app_data = platform_dirs::AppDirs::new(Some("bert"), false)
    //                 .unwrap()
    //                 .data_dir;
    //             local_app_data.join("bin")
    //         }
    //         Platform::MacOS => PathBuf::from("/usr/local/bin"),
    //         Platform::Linux => PathBuf::from("/usr/local/bin"),
    //     }
    // }

    // pub fn package_manager_commands() -> PackageManagerCommands {
    //     match Self::current() {
    //         Platform::Windows => PackageManagerCommands {
    //             install: "install",
    //             uninstall: "uninstall",
    //             update: "upgrade",
    //             command_prefix: "brew",
    //         },
    //         Platform::MacOS => PackageManagerCommands {
    //             install: "install",
    //             uninstall: "uninstall",
    //             update: "upgrade",
    //             command_prefix: "brew",
    //         },
    //         Platform::Linux => PackageManagerCommands {
    //             install: "install",
    //             uninstall: "uninstall",
    //             update: "upgrade",
    //             command_prefix: "brew",
    //         },
    //     }
    // }
}

// pub struct PackageManagerCommands {
//     pub install: &'static str,
//     pub uninstall: &'static str,
//     pub update: &'static str,
//     pub command_prefix: &'static str,
// }
