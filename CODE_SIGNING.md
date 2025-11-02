# Code Signing Guide for ClipForge

This guide explains how to add code signing certificates to ClipForge for distribution on macOS and Windows.

## Table of Contents

- [Why Code Signing?](#why-code-signing)
- [macOS Code Signing](#macos-code-signing)
- [Windows Code Signing](#windows-code-signing)
- [GitHub Actions Integration](#github-actions-integration)
- [Testing Signed Builds](#testing-signed-builds)

---

## Why Code Signing?

Code signing provides several benefits:

1. **User Trust:** Operating systems trust signed applications and display fewer security warnings
2. **Identity Verification:** Proves the application comes from a verified developer
3. **Tamper Protection:** Detects if the application has been modified after signing
4. **Distribution Requirements:** Required for Mac App Store and recommended for Microsoft Store

**Current Status:** ClipForge builds are currently **unsigned**. Users will see security warnings on first launch.

---

## macOS Code Signing

### Requirements

1. **Apple Developer Account**
   - Enroll at [developer.apple.com](https://developer.apple.com/programs/)
   - Cost: $99/year (individual) or $299/year (organization)

2. **Developer ID Application Certificate**
   - This certificate is for distributing apps **outside** the Mac App Store
   - If targeting App Store, you'll need a different certificate

### Step 1: Create Certificates

1. Log in to [Apple Developer Account](https://developer.apple.com/account/)
2. Go to **Certificates, Identifiers & Profiles**
3. Click **Certificates** > **+** button
4. Select **Developer ID Application**
5. Follow the wizard to generate a certificate signing request (CSR):
   - Open **Keychain Access** on your Mac
   - Go to **Keychain Access > Certificate Assistant > Request a Certificate from a Certificate Authority**
   - Enter your email and name
   - Select "Saved to disk"
6. Upload the CSR file to Apple Developer portal
7. Download the certificate and double-click to install in Keychain

### Step 2: Get Certificate Details

Find your signing identity:

```bash
# List all code signing identities
security find-identity -v -p codesigning

# Output will look like:
# 1) ABCD1234... "Developer ID Application: Your Name (TEAM_ID)"
```

Note down:
- **Signing Identity:** The full string (e.g., "Developer ID Application: Your Name (TEAM_ID)")
- **Team ID:** The 10-character ID in parentheses

### Step 3: Set Up App-Specific Password (for Notarization)

Apple requires notarization for apps distributed outside the App Store:

1. Go to [appleid.apple.com](https://appleid.apple.com/)
2. Sign in with your Apple ID
3. Go to **Security > App-Specific Passwords**
4. Click **+** to generate a new password
5. Name it "ClipForge Notarization"
6. **Save this password securely** - you'll need it for CI/CD

### Step 4: Update Configuration

Edit `src-tauri/tauri.conf.json`:

```json
{
  "bundle": {
    "macOS": {
      "minimumSystemVersion": "11.0",
      "entitlements": "entitlements.plist",
      "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)",
      "hardenedRuntime": true
    }
  }
}
```

### Step 5: Build and Sign Locally

```bash
# Build with signing
npm run build:mac

# The bundle will be signed automatically if identity is found
```

### Step 6: Notarize (Required for macOS 10.15+)

After building:

```bash
# Submit for notarization
xcrun notarytool submit \
  src-tauri/target/release/bundle/dmg/ClipForge_universal.dmg \
  --apple-id "your-apple-id@example.com" \
  --team-id "YOUR_TEAM_ID" \
  --password "xxxx-xxxx-xxxx-xxxx"

# Wait for approval (usually 5-15 minutes)
# Check status
xcrun notarytool info <submission-id> \
  --apple-id "your-apple-id@example.com" \
  --team-id "YOUR_TEAM_ID" \
  --password "xxxx-xxxx-xxxx-xxxx"

# Once approved, staple the ticket
xcrun stapler staple src-tauri/target/release/bundle/dmg/ClipForge_universal.dmg
```

### Step 7: Verify Signing

```bash
# Check if app is signed
codesign -dv --verbose=4 src-tauri/target/release/bundle/macos/ClipForge.app

# Check if notarized
spctl -a -vv src-tauri/target/release/bundle/macos/ClipForge.app

# Should output: "accepted" and "source=Notarized Developer ID"
```

---

## Windows Code Signing

### Requirements

1. **Code Signing Certificate**
   - Purchase from a Certificate Authority (CA):
     - [DigiCert](https://www.digicert.com/code-signing/) - $474/year
     - [Sectigo](https://sectigo.com/ssl-certificates-tls/code-signing) - $184/year
     - [GlobalSign](https://www.globalsign.com/en/code-signing-certificate) - $249/year
   - Must be an **EV (Extended Validation) certificate** for best trust

2. **USB Token or Cloud HSM**
   - Modern EV certificates require hardware storage
   - Token is typically provided with the certificate

### Step 1: Obtain Certificate

1. Purchase a code signing certificate from a CA
2. Complete identity verification (can take 1-7 days)
3. Receive USB token or download certificate
4. Install certificate on your Windows build machine

### Step 2: Find Certificate Thumbprint

```powershell
# List all code signing certificates
Get-ChildItem -Path Cert:\CurrentUser\My -CodeSigningCert

# Note the Thumbprint value
```

### Step 3: Update Configuration

Edit `src-tauri/tauri.conf.json`:

```json
{
  "bundle": {
    "windows": {
      "digestAlgorithm": "sha256",
      "certificateThumbprint": "YOUR_CERTIFICATE_THUMBPRINT_HERE",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

Available timestamp servers:
- DigiCert: `http://timestamp.digicert.com`
- Sectigo: `http://timestamp.sectigo.com`
- GlobalSign: `http://timestamp.globalsign.com`

### Step 4: Build and Sign

```bash
# Build with signing (on Windows)
npm run build:win

# The installer will be signed automatically if certificate is found
```

### Step 5: Verify Signing

```powershell
# Check if executable is signed
Get-AuthenticodeSignature src-tauri\target\release\ClipForge.exe

# Should show "Valid" status
```

Right-click the .exe > Properties > Digital Signatures tab to view certificate details.

---

## GitHub Actions Integration

### Setting Up Secrets

Add these secrets to your GitHub repository (Settings > Secrets and variables > Actions):

#### macOS Secrets

1. **APPLE_CERTIFICATE**
   - Export your signing certificate from Keychain:
     ```bash
     # Export as .p12 file with a password
     # Then convert to base64:
     base64 -i certificate.p12 | pbcopy
     ```
   - Paste the base64 string as secret value

2. **APPLE_CERTIFICATE_PASSWORD**
   - The password you used when exporting the certificate

3. **APPLE_SIGNING_IDENTITY**
   - Example: `Developer ID Application: Your Name (TEAM_ID)`

4. **APPLE_ID**
   - Your Apple ID email

5. **APPLE_PASSWORD**
   - The app-specific password you generated

6. **APPLE_TEAM_ID**
   - Your 10-character team ID

#### Windows Secrets

1. **WINDOWS_CERTIFICATE**
   - Export your certificate as base64:
     ```powershell
     [Convert]::ToBase64String([IO.File]::ReadAllBytes("certificate.pfx")) | Set-Clipboard
     ```

2. **WINDOWS_CERTIFICATE_PASSWORD**
   - The certificate password

3. **WINDOWS_CERTIFICATE_THUMBPRINT**
   - The thumbprint from Step 2 above

### Update GitHub Actions Workflow

Edit `.github/workflows/release.yml` and uncomment the signing sections:

```yaml
- name: Build Tauri app
  uses: tauri-apps/tauri-action@v0
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    # macOS code signing
    APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
    APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
    APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
    APPLE_ID: ${{ secrets.APPLE_ID }}
    APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
    APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
    # Windows code signing
    TAURI_PRIVATE_KEY: ${{ secrets.WINDOWS_CERTIFICATE }}
    TAURI_KEY_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
```

### Test the Workflow

1. Create a tag:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. Monitor the GitHub Actions run
3. Signed binaries will be uploaded to the release

---

## Testing Signed Builds

### macOS

```bash
# Verify signature
codesign -dv --verbose=4 ClipForge.app

# Verify notarization
spctl -a -vv ClipForge.app

# Test Gatekeeper
xattr -cr ClipForge.app
open ClipForge.app
# Should launch without warnings
```

### Windows

1. Right-click the installer > Properties
2. Go to Digital Signatures tab
3. Verify certificate is valid and trusted
4. Run the installer - should not show SmartScreen warnings

---

## Troubleshooting

### macOS: "No identity found"

```bash
# List available identities
security find-identity -v -p codesigning

# If empty, install your certificate:
# 1. Download from Apple Developer portal
# 2. Double-click to install in Keychain
```

### macOS: Notarization fails

Common issues:
1. **Wrong Apple ID:** Use the Apple ID associated with your developer account
2. **Wrong password:** Must be an app-specific password, not your regular password
3. **Hardened Runtime not enabled:** Check `hardenedRuntime: true` in config
4. **Missing entitlements:** Ensure `entitlements.plist` is configured

Check detailed logs:
```bash
xcrun notarytool log <submission-id> --apple-id "..." --password "..."
```

### Windows: Certificate not found

```powershell
# Verify certificate is installed
Get-ChildItem -Path Cert:\CurrentUser\My -CodeSigningCert

# If not found, import the certificate
Import-PfxCertificate -FilePath certificate.pfx -CertStoreLocation Cert:\CurrentUser\My
```

---

## Cost Summary

| Item | Cost | Frequency |
|------|------|-----------|
| Apple Developer Account | $99 | Annual |
| Windows Code Signing (Standard) | $184-$474 | Annual |
| Windows EV Certificate | $300-$600 | Annual |

**Total annual cost:** ~$500-$1,100 depending on certificate provider

---

## Alternative: Start Unsigned

It's perfectly acceptable to distribute unsigned builds initially:

**Pros:**
- Zero cost
- Faster iteration
- Good for beta testing and early releases

**Cons:**
- Users see security warnings
- Less professional appearance
- May deter non-technical users

You can add signing later without changing the build infrastructure - just update the configuration and add certificates.

---

## Next Steps

1. **For Production:** Obtain certificates and follow the steps above
2. **For Testing:** Continue with unsigned builds
3. **For Open Source:** Consider asking users to build from source if cost is prohibitive

---

## Resources

- [Apple Code Signing Guide](https://developer.apple.com/support/code-signing/)
- [Apple Notarization Guide](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [Windows Code Signing Best Practices](https://docs.microsoft.com/en-us/windows-hardware/drivers/install/code-signing-best-practices)
- [Tauri Code Signing Docs](https://tauri.app/v1/guides/distribution/sign-macos)

---

**Last Updated:** 2025

For questions or issues with code signing, please open a GitHub issue or consult the resources above.
