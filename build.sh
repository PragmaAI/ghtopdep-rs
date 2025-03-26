#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

echo "Building ghtopdep-rs..."

# Navigate to the project directory
#cd "$(dirname "$0")/ghtopdep-rs"

# Build the project in release mode
cargo build --release

# Create a directory for the binary if it doesn't exist
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

# Copy the binary to the install directory
cp target/release/ghtopdep-rs "$INSTALL_DIR/"

# Check if the install directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "Adding $INSTALL_DIR to PATH in your shell configuration..."
    
    # Determine which shell configuration file to use
    if [ -f "$HOME/.zshrc" ]; then
        SHELL_CONFIG="$HOME/.zshrc"
    elif [ -f "$HOME/.bashrc" ]; then
        SHELL_CONFIG="$HOME/.bashrc"
    elif [ -f "$HOME/.bash_profile" ]; then
        SHELL_CONFIG="$HOME/.bash_profile"
    else
        SHELL_CONFIG="$HOME/.profile"
    fi
    
    # Add the directory to PATH in the shell configuration
    echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_CONFIG"
    
    echo "Added $INSTALL_DIR to PATH in $SHELL_CONFIG"
    echo "Please run 'source $SHELL_CONFIG' or start a new terminal to update your PATH."
else
    echo "$INSTALL_DIR is already in your PATH."
fi

echo "Installation complete! You can now run 'ghtopdep-rs' from anywhere."
echo "Example usage: ghtopdep-rs near/near-sdk-rs"