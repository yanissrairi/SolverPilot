# Deployment Guide

## Overview

SolverPilot uses **GitHub Actions** for automated CI/CD with multi-platform builds and releases.

---

## CI/CD Pipeline

### Continuous Integration (ci.yml)

Runs on every push to `main` and on all pull requests.

**Triggers**:

- Push to `main` branch
- Pull requests targeting `main`

**Jobs**:

#### 1. Frontend Checks

**Platform**: Ubuntu Latest
**Steps**:

1. Setup Bun runtime
2. Cache dependencies
3. Install packages (`bun install --frozen-lockfile`)
4. **Lint**: ESLint check (`bun run lint:check`)
5. **Format**: Prettier check (`bun run format:check`)
6. **Type Check**: Svelte + TypeScript (`bun run check`)

**Cache Strategy**: Bun install cache based on `bun.lockb` hash

#### 2. Backend Checks

**Platform**: Ubuntu Latest
**Working Directory**: `src-tauri/`
**System Dependencies**:

- `libwebkit2gtk-4.1-dev`
- `libappindicator3-dev`
- `librsvg2-dev`
- `patchelf`

**Steps**:

1. Setup Rust stable with rustfmt + clippy
2. Cache Cargo registry, git database, and build artifacts
3. **Format Check**: `cargo fmt --all -- --check`
4. **Clippy**: `cargo clippy --all-targets --all-features -- -D warnings`
5. **Build Check**: `cargo check --all-targets --all-features`

**Cargo Cache**: Based on `Cargo.lock` hash

#### 3. Security Audit

**Platform**: Ubuntu Latest
**Tool**: cargo-deny
**Check**: Advisory database for known vulnerabilities

**Configuration**: `src-tauri/deny.toml`

---

### Release Pipeline (release.yml)

Builds production binaries for all supported platforms.

**Triggers**:

- Git tags matching `v*` (e.g., `v0.1.0`)
- Manual workflow dispatch

**Permissions**: `contents: write` (for creating releases)

#### Multi-Platform Build Matrix

| Platform        | OS               | Rust Target                 | Output Formats      |
| --------------- | ---------------- | --------------------------- | ------------------- |
| **Linux x64**   | Ubuntu 22.04     | `x86_64-unknown-linux-gnu`  | `.deb`, `.AppImage` |
| **Linux ARM64** | Ubuntu 22.04 ARM | `aarch64-unknown-linux-gnu` | `.deb`, `.AppImage` |
| **macOS Intel** | macOS 13         | `x86_64-apple-darwin`       | `.dmg`, `.app`      |
| **macOS ARM**   | macOS 14         | `aarch64-apple-darwin`      | `.dmg`, `.app`      |
| **Windows x64** | Windows Latest   | `x86_64-pc-windows-msvc`    | `.msi`, `.exe`      |

#### Build Steps (Per Platform)

1. **Checkout code**
2. **Install system dependencies** (Linux only):
   - `libwebkit2gtk-4.1-dev`
   - `libappindicator3-dev`
   - `librsvg2-dev`
   - `patchelf`

3. **Setup Bun**:
   - Latest version
   - Cache install directory

4. **Setup Rust**:
   - Stable toolchain
   - Platform-specific target

5. **Cache Cargo**:
   - Registry, git database, build artifacts
   - Key: `${{ runner.os }}-${{ rust-target }}-cargo-${{ Cargo.lock hash }}`

6. **Install frontend dependencies**:

   ```bash
   bun install --frozen-lockfile
   ```

7. **Build Tauri app**:
   - Uses `tauri-apps/tauri-action@v0.6`
   - Automatically creates GitHub release
   - Uploads platform-specific installers

---

## Build Artifacts

### Output Locations

**Development**:

```
src-tauri/target/debug/
├── solver-pilot          # Debug executable
└── bundle/               # Debug installers (if generated)
```

**Production**:

```
src-tauri/target/release/
├── solver-pilot          # Optimized executable
└── bundle/
    ├── deb/              # Linux .deb packages
    ├── appimage/         # Linux .AppImage
    ├── dmg/              # macOS disk images
    ├── macos/            # macOS .app bundle
    └── msi/              # Windows installers
```

### Artifact Sizes (Approximate)

| Platform | Format    | Size   |
| -------- | --------- | ------ |
| Linux    | .deb      | ~18 MB |
| Linux    | .AppImage | ~22 MB |
| macOS    | .dmg      | ~15 MB |
| Windows  | .msi      | ~16 MB |

**Size Optimization**:

- LTO (Link-Time Optimization) enabled
- Debug symbols stripped
- `opt-level = "z"` (size optimization)
- `panic = "abort"` (removes unwinding code)

---

## Release Process

### 1. Prepare Release

```bash
# Update version in Cargo.toml
vim src-tauri/Cargo.toml  # version = "0.2.0"

# Update version in tauri.conf.json
vim src-tauri/tauri.conf.json  # "version": "0.2.0"

# Update version in package.json
vim package.json  # "version": "0.2.0"

# Commit changes
git add .
git commit -m "chore: bump version to 0.2.0"
```

### 2. Create Git Tag

```bash
# Create annotated tag
git tag -a v0.2.0 -m "Release v0.2.0"

# Push tag to trigger release
git push origin v0.2.0
```

### 3. Monitor Build

1. Go to **Actions** tab in GitHub
2. Watch **Release** workflow progress
3. Builds run in parallel for all platforms (~15-20 minutes total)

### 4. Verify Release

1. Check **Releases** page
2. Verify all platform artifacts uploaded
3. Download and test installers

### 5. Publish Release Notes

Edit the auto-generated release with:

- Changelog (features, fixes, breaking changes)
- Installation instructions
- Known issues
- Upgrade notes

---

## Manual Build

### Local Production Build

```bash
# Full production build for current platform
bun run tauri build

# Output in src-tauri/target/release/bundle/
```

### Cross-Platform Builds

**Linux → Windows** (using Docker):

```bash
# Not officially supported, use GitHub Actions
```

**macOS Universal Binary**:

```bash
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
bun run tauri build --target universal-apple-darwin
```

---

## Deployment Targets

### GitHub Releases (Primary)

- **URL**: `https://github.com/yanissrairi/SolverPilot/releases`
- **Auto-generated** by `tauri-action`
- **Artifacts**: All platform installers
- **Update mechanism**: Users manually download new versions

### Future: Auto-Update Support

Tauri supports auto-updates via `tauri-plugin-updater`:

**Setup**:

1. Enable updater in `tauri.conf.json`
2. Configure update endpoint
3. Sign releases with private key
4. Users receive in-app update notifications

**Not Yet Implemented** - Manual downloads for now

---

## Environment Variables

### CI/CD Secrets

Required in GitHub repository settings:

| Secret         | Purpose          | Used In                     |
| -------------- | ---------------- | --------------------------- |
| `GITHUB_TOKEN` | Release creation | release.yml (auto-provided) |

### Build-Time Variables

Set in workflow:

| Variable           | Value    | Purpose               |
| ------------------ | -------- | --------------------- |
| `CARGO_TERM_COLOR` | `always` | Colored cargo output  |
| `RUST_BACKTRACE`   | `1`      | Full error backtraces |

---

## Cache Strategy

### Frontend Cache

- **What**: Bun install cache
- **Key**: `${{ runner.os }}-bun-${{ hashFiles('**/bun.lockb') }}`
- **Restore**: Partial key match on OS
- **Location**: `~/.bun/install/cache`

### Backend Cache

- **What**: Cargo registry + build artifacts
- **Key**: `${{ runner.os }}-${{ rust-target }}-cargo-${{ hashFiles('**/Cargo.lock') }}`
- **Restore**: Partial key match on OS + target
- **Locations**:
  - `~/.cargo/bin/`
  - `~/.cargo/registry/`
  - `~/.cargo/git/`
  - `src-tauri/target/`

**Cache Hit Benefits**:

- Frontend: ~30s → ~5s install time
- Backend: ~5 min → ~1 min build time

---

## Continuous Deployment

### Current Strategy

**Semi-Automated**:

1. Developer creates tag
2. GitHub Actions builds all platforms
3. Release auto-published
4. Users manually download

### Future Improvements

1. **Automated Version Bumping**:
   - Bot auto-increments versions
   - Changelog generation from commits

2. **Auto-Updates**:
   - In-app update notifications
   - Background downloads
   - Restart to apply updates

3. **Beta Releases**:
   - Separate beta channel
   - Early testing before stable release

4. **Homebrew/Chocolatey**:
   - Package managers for easier installation
   - Auto-update via package manager

---

## Monitoring & Rollback

### Build Monitoring

- **GitHub Actions**: Real-time build logs
- **Notifications**: Email on build failure
- **Badges**: README.md status badges

### Rollback Procedure

If a release has critical issues:

```bash
# 1. Delete bad release from GitHub
# (Manually via GitHub UI)

# 2. Create hotfix tag
git tag -d v0.2.0              # Delete local tag
git push origin :refs/tags/v0.2.0  # Delete remote tag

# 3. Fix issue and re-release
git commit -m "fix: critical bug"
git tag -a v0.2.1 -m "Hotfix release"
git push origin v0.2.1
```

### Version Pinning

Users can download specific older versions from Releases page.

---

## Security

### Dependency Scanning

- **cargo-deny**: Checks Rust dependencies for advisories
- **Dependabot**: Auto-creates PRs for dependency updates (configured in `.github/dependabot.yml`)

### Supply Chain Security

- **Locked Dependencies**: `--frozen-lockfile` ensures reproducible builds
- **Checksums**: Cargo.lock and bun.lock pin exact versions
- **Audit Logs**: GitHub Actions logs all build steps

### Code Signing

**Not Yet Implemented**:

- macOS: Needs Apple Developer certificate
- Windows: Needs code signing certificate
- Users see "Unverified Publisher" warnings

---

## Performance

### Build Times (Approximate)

| Stage                   | Cold Build | Cached Build |
| ----------------------- | ---------- | ------------ |
| Frontend Install        | 30s        | 5s           |
| Frontend Type Check     | 15s        | 15s          |
| Backend Build (Debug)   | 5 min      | 1 min        |
| Backend Build (Release) | 8 min      | 2 min        |
| **Total CI**            | ~12 min    | ~4 min       |
| **Total Release**       | ~18 min    | ~6 min       |

### Optimization Opportunities

1. **Incremental Builds**: Already enabled for dependencies
2. **Parallel Jobs**: Frontend + Backend run concurrently
3. **Cache Hits**: ~70% hit rate with stable dependencies
4. **Artifact Reuse**: Consider building once, packaging multiple formats

---

## Summary

**CI/CD Platform**: GitHub Actions
**Supported Platforms**: Linux (x64, ARM64), macOS (Intel, ARM), Windows (x64)
**Build Triggers**: Push to main (CI), Git tags (Release)
**Artifact Storage**: GitHub Releases
**Update Mechanism**: Manual download (auto-update planned)
**Build Time**: ~15-20 minutes for all platforms
**Security**: cargo-deny audits, Dependabot updates, locked dependencies
