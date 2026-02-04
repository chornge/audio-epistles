# Audio Epistles

![Build Status](https://github.com/chornge/audio-epistles/actions/workflows/build.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-2.3.0-green.svg)

An automated service for fetching the latest video from a YouTube playlist, extracting its details, and publishing the audio to Spotify/Anchor.fm. The service is designed to run periodically (e.g., every hour via cron), ensuring new podcasts are published reliably.

## Architecture

![Design Doc](./DESIGN-DOC.excalidraw.png)

The application follows a modular architecture with the following components:

- **Video Service (`video.rs`):** Fetches the latest video ID from a YouTube playlist using regex pattern matching.
- **Database Service (`db.rs`):** Manages SQLite database operations to track uploaded video IDs and prevent duplicates.
- **Processor Service (`processor.rs`):** Orchestrates the workflow by coordinating between services.
- **Episode Service (`episode.rs`):** Downloads videos using yt-dlp, extracts sermon chapters from descriptions, and trims audio segments using FFmpeg.
- **WebDriver Service (`webdriver.rs`):** Automates browser interactions using Fantoccini/ChromeDriver to upload episodes to Spotify/Anchor.fm.
- **Main (`main.rs`):** Entry point that initializes services and manages the execution flow.

## Features

- **Automated Fetch & Publish:** Automatically detects and publishes the latest sermon from a YouTube playlist to Spotify.
- **Low Latency:** Publishes new sermons within approximately 1 hour of playlist upload.
- **Duplicate Prevention:** Uses SQLite database to track published videos and avoid re-uploads.
- **Smart Chapter Detection:** Automatically extracts sermon segments from timestamped YouTube descriptions.
- **Audio Trimming:** Precisely extracts sermon audio using FFmpeg, removing pre/post sermon content.
- **Human-like Interaction:** Implements randomized delays to mimic human behavior and avoid bot detection.

## File Structure

```
audio_epistles/
├── .github/
│   └── workflows/          # CI/CD workflows
│       ├── build.yml       # Build and lint on push/PR
│       └── release.yml     # Release automation on tags
├── assets/                 # Downloaded media files (video.mp4, audio.mp3)
├── src/
│   ├── db.rs               # Database operations (SQLite)
│   ├── episode.rs          # Video download, chapter extraction, audio trimming
│   ├── main.rs             # Application entry point
│   ├── processor.rs        # Workflow orchestration
│   ├── video.rs            # YouTube playlist video ID fetching
│   └── webdriver.rs        # Browser automation for Spotify upload
├── .dockerignore           # Docker build exclusions
├── .env                    # Environment variables (not in version control)
├── .gitignore              # Git exclusions
├── build.rs                # Build script (creates videos.db)
├── Cargo.lock              # Dependency lock file
├── Cargo.toml              # Project manifest and dependencies
├── CHANGELOG.md            # Version history and changes
├── CONTRIBUTING.md         # Contribution guidelines
├── DESIGN-DOC.excalidraw.png # Architecture diagram
├── docker-compose.yml      # Docker Compose configuration
├── Dockerfile              # Multi-stage Docker build
├── init-db.sh              # Database initialization script for Docker
├── LICENSE                 # MIT License
├── README.md               # This file
└── videos.db               # SQLite database (stores last uploaded video ID)
```

## Requirements

### Native Development

- **Rust** (1.85.1 or later) - Install via [rustup](https://rustup.rs/)
- **FFmpeg** - Audio/video processing
  - macOS: `brew install ffmpeg`
  - Linux: `apt-get install ffmpeg` or `yum install ffmpeg`
- **yt-dlp** - YouTube video downloader
  - macOS: `brew install yt-dlp`
  - Linux: `pip install yt-dlp`
- **ChromeDriver** - Browser automation driver
  - macOS: `brew install chromedriver`
  - Linux: Download from [ChromeDriver site](https://chromedriver.chromium.org/)

### Docker Development

- Docker Desktop or Docker Engine
- Docker Compose (v2 or later)

## Setup

### Option 1: Native Setup

1. **Clone the repository:**

   ```bash
   git clone https://github.com/chornge/audio-epistles.git
   cd audio-epistles
   ```

2. **Create environment file:**

   ```bash
   cp .env.example .env
   ```

3. **Configure environment variables in `.env`:**

   ```env
   SPOTIFY_EMAIL=your-spotify-email@example.com
   SPOTIFY_PASSWORD=your-spotify-password
   SERMON_PLAYLIST_ID=PLqOU6DjSKs7wkpl8NK-dplD2o31m1lXFT
   AUDIO_FILE=./assets/audio.mp3
   DB_URL=videos.db
   ```

   **Environment Variable Details:**

   - `SPOTIFY_EMAIL`: Your Spotify account email (used for Anchor.fm login)
   - `SPOTIFY_PASSWORD`: Your Spotify account password
   - `SERMON_PLAYLIST_ID`: YouTube playlist ID (found in playlist URL after `list=`)
   - `AUDIO_FILE`: Path to save extracted audio (relative to project root)
   - `DB_URL`: SQLite database file path (relative to project root)

4. **Build and run:**
   ```bash
   cargo build --release
   cargo run --release
   ```

### Option 2: Docker Setup

1. **Clone and configure:**

   ```bash
   git clone https://github.com/chornge/audio-epistles.git
   cd audio-epistles
   cp .env.example .env
   # Edit .env with your credentials
   ```

2. **Build and run with Docker Compose:**

   ```bash
   docker-compose up --build
   ```

3. **Run in background:**
   ```bash
   docker-compose up -d
   ```

## Automation

### Cron Job (macOS/Linux)

To run the service periodically, add a cron job:

```bash
crontab -e
```

**Run every hour (on the hour):**

```cron
0 * * * * cd /full/path/to/audio-epistles && cargo run --release >> cron.log 2>&1
```

**Run weekdays (Monday-Friday) at noon:**

```cron
0 12 * * 1-5 cd /full/path/to/audio-epistles && cargo run --release >> cron.log 2>&1
```

**Run Wednesdays at 9:00 PM:**

```cron
0 21 * * 3 cd /full/path/to/audio-epistles && cargo run --release >> cron.log 2>&1
```

### Docker with Cron

You can use a cron container or host-level cron to trigger Docker runs:

```cron
0 * * * * cd /path/to/audio-epistles && docker-compose run worker >> cron.log 2>&1
```

## Security Best Practices

### Protecting Credentials

1. **Never commit `.env` file** - It's already in `.gitignore`
2. **Use GitHub Secrets** for CI/CD workflows:
   - Repository Settings → Secrets → Actions
   - Add: `SPOTIFY_EMAIL`, `SPOTIFY_PASSWORD`, `SERMON_PLAYLIST_ID`
3. **Use environment-specific configs** for production deployments
4. **Consider using a secrets manager** (AWS Secrets Manager, HashiCorp Vault, etc.) for production

### GitHub Secrets Example

In GitHub Actions workflow:

```yaml
env:
  SPOTIFY_EMAIL: ${{ secrets.SPOTIFY_EMAIL }}
  SPOTIFY_PASSWORD: ${{ secrets.SPOTIFY_PASSWORD }}
  SERMON_PLAYLIST_ID: ${{ secrets.SERMON_PLAYLIST_ID }}
```

## Usage

The application runs as a one-shot process. Each execution:

1. Connects to the SQLite database
2. Fetches the latest video ID from the configured YouTube playlist
3. Compares with the last processed video ID
4. If new video found:
   - Downloads video and metadata
   - Extracts sermon chapter timestamps from description
   - Trims audio to sermon portion
   - Uploads to Spotify/Anchor.fm as a draft episode
   - Updates database with new video ID
5. Exits with status report and timing information

### Manual Run

```bash
cargo run --release
```

### Docker Run

```bash
docker-compose run worker
```

## Development

### Running Tests

```bash
cargo test
```

### Linting

```bash
cargo clippy -- -D warnings
```

### Formatting

```bash
cargo fmt
```

### Building for Release

```bash
cargo build --release
```

Binary will be located at: `target/release/audio_epistles`

## Troubleshooting

### ChromeDriver Issues

**Error: "ChromeDriver not found"**

- Ensure ChromeDriver is installed and in PATH
- macOS may require: `xattr -d com.apple.quarantine $(which chromedriver)`

**Error: "ChromeDriver version mismatch"**

- Update ChromeDriver to match your Chrome version
- Check versions: `chrome --version` and `chromedriver --version`

### FFmpeg Issues

**Error: "ffmpeg: command not found"**

- Install FFmpeg using your package manager
- Verify installation: `ffmpeg -version`

### yt-dlp Issues

**Error: "Video unavailable" or "Sign in to confirm your age"**

- Some videos require authentication
- Try updating yt-dlp: `pip install --upgrade yt-dlp`
- Check if video is publicly accessible

### Spotify Upload Issues

**Error: "Login failed" or CAPTCHA triggered**

- Verify credentials in `.env`
- Reduce run frequency (max 2x per hour recommended)
- Clear browser cache/cookies and retry
- Check if Spotify requires 2FA (not currently supported)

### Database Issues

**Error: "database is locked"**

- Ensure only one instance is running
- Check file permissions on `videos.db`
- Docker: Verify volume mount permissions

## Platform Support

- **macOS:** Fully tested and supported
- **Linux:** Supported via Docker (native untested)
- **Windows:** Not tested (Docker recommended)

## Limitations

- Untested on Windows and native Linux environments (Docker recommended)
- Running more than twice per hour may trigger anti-bot measures from YouTube or Spotify
- Two-factor authentication (2FA) is not supported for Spotify login
- Requires publicly accessible YouTube videos (private/unlisted may fail)
- Episode is saved as draft; manual publish or schedule step required

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## Changelog

See [CHANGELOG.md](./CHANGELOG.md) for version history and release notes.

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## Authors

- Christian Mbaba - [mxlightningx@gmail.com](mailto:mxlightningx@gmail.com)
- Dominion Chapel Houston - [media@dominionchapel.org](mailto:media@dominionchapel.org)

## Acknowledgments

A special shout-out to the authors and contributors of [Schroedinger-Hat/youtube-to-spotify](https://github.com/Schroedinger-Hat/youtube-to-spotify), whose work directly inspired this project. Thank you for your dedication and open-source spirit!

## Support

For bug reports and feature requests, please open an issue on [GitHub Issues](https://github.com/chornge/audio-epistles/issues).
