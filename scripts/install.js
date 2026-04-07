#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const REPO = 'yourusername/pipecheck';
const VERSION = require('../package.json').version;

function getBinaryName() {
  const platform = process.platform;
  const arch = process.arch;
  
  if (platform === 'win32') {
    return `pipecheck-${arch}.exe`;
  }
  return `pipecheck-${platform}-${arch}`;
}

function install() {
  const binaryName = getBinaryName();
  const npmDir = path.join(__dirname, '..', 'npm');
  const binaryPath = path.join(npmDir, binaryName);

  // Check if binary already exists
  if (fs.existsSync(binaryPath)) {
    console.log('✓ Pipecheck binary already installed');
    return;
  }

  console.log('Installing pipecheck...');
  
  // Try to build from source if Cargo is available
  try {
    console.log('Building from source...');
    execSync('cargo build --release', { stdio: 'inherit' });
    
    const sourceBinary = path.join(__dirname, '..', 'target', 'release', 
      process.platform === 'win32' ? 'pipecheck.exe' : 'pipecheck');
    
    if (!fs.existsSync(npmDir)) {
      fs.mkdirSync(npmDir, { recursive: true });
    }
    
    fs.copyFileSync(sourceBinary, binaryPath);
    fs.chmodSync(binaryPath, 0o755);
    
    console.log('✓ Pipecheck installed successfully');
  } catch (error) {
    console.error('Failed to build from source. Please ensure Rust is installed.');
    console.error('Visit https://rustup.rs to install Rust');
    process.exit(1);
  }
}

install();
