# BERT-CLI

## Overview

BERT-CLI is a friendly cross-platform package assistant built on top of Homebrew. It leverages Homebrew's extensive package repository to provide seamless package management across different operating systems. BERT-CLI automatically handles the installation of missing commands and manages Homebrew installations, making it easier for users to manage their software packages from the command line.

## Features

- **Automatic Command Installation**: Automatically installs missing commands using Homebrew.
- **Cross-Platform Support**: Works on Windows, macOS, and Linux.
- **Package Management**: Install, uninstall, update, and search for packages.
- **Homebrew Integration**: Leverages Homebrew's package repository for managing software.

## Installation

```bash
# Download the binary for MacOS
curl -L -o /usr/local/bin/bert "https://github.com/michaelessiet/bert-cli/releases/download/v0.1.4/bert-darwin-amd64"

# Make it executable
chmod +x /usr/local/bin/bert
```

## Usage

Once installed, you can use BERT-CLI from your terminal. Below are some examples of how to use the tool for different tasks:

### Install a Package

```bash
bert install <package_name>
```

### Uninstall a Package

```bash
bert uninstall <package_name>
```

### Update Installed Packages

```bash
bert update
```

### Search for a Package

```bash
bert search <query>
```

### List Installed Packages

```bash
bert list
```

### Execute a Command

If a command is not found, BERT-CLI will attempt to install it using Homebrew:

```bash
bert <command> [args...]
```

## Configuration

BERT-CLI does not require any specific configuration. It automatically detects the platform and manages Homebrew installations accordingly.

## Contributing

We welcome contributions to BERT-CLI! If you have any ideas, suggestions, or bug reports, please open an issue or submit a pull request on our GitHub repository.

## License

BERT-CLI is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

## Acknowledgements

This project leverages the Homebrew package manager and is inspired by the advancements in cross-platform package management. It is also heavily inspired by Bert the Pomeranian, and its web3 community.
