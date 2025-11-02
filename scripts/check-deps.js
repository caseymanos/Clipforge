#!/usr/bin/env node

/**
 * Pre-build dependency checker for ClipForge
 * Verifies that required dependencies are installed before building
 */

import { execSync } from 'child_process';
import { platform } from 'os';

const currentPlatform = platform();
let hasErrors = false;

console.log('🔍 Checking build dependencies...\n');

// Check Node.js version
try {
  const nodeVersion = process.version;
  const majorVersion = parseInt(nodeVersion.slice(1).split('.')[0]);

  if (majorVersion >= 18) {
    console.log('✅ Node.js:', nodeVersion);
  } else {
    console.error('❌ Node.js version too old:', nodeVersion, '(requires >= 18)');
    hasErrors = true;
  }
} catch (err) {
  console.error('❌ Failed to check Node.js version');
  hasErrors = true;
}

// Check npm
try {
  const npmVersion = execSync('npm --version', { encoding: 'utf-8' }).trim();
  console.log('✅ npm:', npmVersion);
} catch (err) {
  console.error('❌ npm not found');
  hasErrors = true;
}

// Check Rust
try {
  const rustVersion = execSync('rustc --version', { encoding: 'utf-8' }).trim();
  console.log('✅ Rust:', rustVersion);
} catch (err) {
  console.error('❌ Rust not found - install from https://rustup.rs/');
  hasErrors = true;
}

// Check Cargo
try {
  const cargoVersion = execSync('cargo --version', { encoding: 'utf-8' }).trim();
  console.log('✅ Cargo:', cargoVersion);
} catch (err) {
  console.error('❌ Cargo not found');
  hasErrors = true;
}

// Check FFmpeg (warning only, not required for build)
try {
  const ffmpegVersion = execSync('ffmpeg -version', { encoding: 'utf-8' }).split('\n')[0];
  console.log('✅ FFmpeg:', ffmpegVersion);
} catch (err) {
  console.warn('⚠️  FFmpeg not found in PATH');
  console.warn('   App will build but requires FFmpeg at runtime');
  console.warn('   Install instructions:');
  if (currentPlatform === 'darwin') {
    console.warn('   macOS: brew install ffmpeg');
  } else if (currentPlatform === 'win32') {
    console.warn('   Windows: https://ffmpeg.org/download.html');
  } else {
    console.warn('   Linux: sudo apt install ffmpeg');
  }
  console.log('');
}

// Platform-specific checks
if (currentPlatform === 'darwin') {
  console.log('\n📦 macOS-specific dependencies:');

  // Check for Xcode Command Line Tools
  try {
    execSync('xcode-select -p', { stdio: 'ignore' });
    console.log('✅ Xcode Command Line Tools installed');
  } catch (err) {
    console.error('❌ Xcode Command Line Tools not found');
    console.error('   Install: xcode-select --install');
    hasErrors = true;
  }
} else if (currentPlatform === 'linux') {
  console.log('\n📦 Linux-specific dependencies:');

  // Check for required system libraries
  const requiredLibs = [
    'libwebkit2gtk-4.1-dev',
    'libappindicator3-dev',
    'librsvg2-dev',
    'patchelf'
  ];

  console.log('ℹ️  Required system libraries:');
  requiredLibs.forEach(lib => console.log(`   - ${lib}`));
  console.log('   Install: sudo apt-get install ' + requiredLibs.join(' '));
} else if (currentPlatform === 'win32') {
  console.log('\n📦 Windows build environment detected');
  console.log('ℹ️  Ensure you have Visual Studio Build Tools installed');
  console.log('   https://visualstudio.microsoft.com/downloads/');
}

console.log('\n' + '='.repeat(50));

if (hasErrors) {
  console.error('\n❌ Build cannot proceed - missing required dependencies');
  console.error('   Please install missing dependencies and try again\n');
  process.exit(1);
} else {
  console.log('\n✅ All required dependencies are installed');
  console.log('   Ready to build ClipForge!\n');
  process.exit(0);
}
