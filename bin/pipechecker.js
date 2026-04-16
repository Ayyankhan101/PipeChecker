#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

const binPath = path.join(__dirname, '..', 'npm', getBinaryName());

const child = spawn(binPath, process.argv.slice(2), { stdio: 'inherit' });

child.on('exit', (code) => {
  process.exit(code);
});

function getBinaryName() {
  const platform = process.platform;
  const arch = process.arch;
  
  if (platform === 'win32') {
    return `pipechecker-${arch}.exe/pipechecker.exe`;
  }
  return `pipechecker-${platform}-${arch}/pipechecker`;
}
