# Background Music Configuration Guide

## Overview

The TCG simulator now includes background music functionality. Music files from the `web/music` folder will automatically play as background music while the program runs, with configurable fade-in/out and delays between songs.

## Features

- **Automatic Music Loading**: Discovers and plays all audio files from the `web/music` folder
- **Configurable Settings**:
  - Fade duration between songs (fade-out simulation)
  - Delay between songs
  - Volume level (0.0 to 1.0)
- **Graceful Shutdown**: Music automatically stops when the program exits
- **Multiple Format Support**: MP3, WAV, FLAC, OGG, M4A, AAC

## How It Works

The music player is initialized in `engine/src/main.rs`:

```rust
let music_config = MusicConfig {
    fade_duration_ms: 1500,      // 1.5 second fade between songs
    delay_between_songs_ms: 2000, // 2 second delay between songs
    volume: 0.3,                  // 30% volume
};
let _music_player = MusicPlayer::new("../web/music", music_config);
_music_player.start();
```

## Customization

To adjust the music settings, edit `engine/src/main.rs` and modify the `MusicConfig` values:

```rust
let music_config = MusicConfig {
    fade_duration_ms: 1500,      // Fade time between songs in milliseconds
    delay_between_songs_ms: 2000, // Pause time between songs in milliseconds
    volume: 0.3,                  // Volume (0.0 = silent, 1.0 = maximum)
};
```

### Common Settings

| Setting | Value | Effect |
|---------|-------|--------|
| `fade_duration_ms` | 1000 | 1 second fade out before next song |
| `fade_duration_ms` | 2000 | 2 second fade out before next song |
| `delay_between_songs_ms` | 0 | No gap between songs |
| `delay_between_songs_ms` | 5000 | 5 second silence between songs |
| `volume` | 0.5 | 50% volume |
| `volume` | 0.8 | 80% volume |
| `volume` | 1.0 | Full volume |

## Requirements

The system requires an audio player to be installed. The application will automatically detect and use one of these players (in order of preference):

1. **ffplay** (recommended) - From FFmpeg suite
2. **mpg123** - Lightweight MP3 player
3. **play** - From SoX audio suite
4. **aplay** - ALSA player
5. **paplay** - PulseAudio player

### Installation

**Ubuntu/Debian:**
```bash
# Install ffplay (recommended)
sudo apt-get install ffmpeg

# Or install mpg123
sudo apt-get install mpg123

# Or install SoX
sudo apt-get install sox
```

**macOS:**
```bash
# Install ffplay
brew install ffmpeg

# Or install mpg123
brew install mpg123
```

If no audio player is found, the application will warn you and continue running without music.

## Music Files

Add your music files to the `web/music/` directory. Supported formats:
- MP3
- WAV
- FLAC
- OGG
- M4A
- AAC

The music player will cycle through all files in the directory repeatedly.

## Troubleshooting

### No music is playing
1. Verify that audio files exist in `web/music/`
2. Check that an audio player is installed (run `which ffplay` or `which mpg123`)
3. Check the console output for error messages
4. Verify file permissions (audio files should be readable)

### Music is too loud/quiet
Adjust the `volume` value in the `MusicConfig` (0.0 to 1.0)

### Gaps between songs are too long/short
Adjust `delay_between_songs_ms` in the `MusicConfig`

### Music cuts off too abruptly
Increase `fade_duration_ms` to create a longer fade-out effect

## Implementation Details

The music player implementation is in `engine/src/music.rs` and provides:

- `MusicPlayer::new()` - Creates a new music player instance
- `MusicPlayer::start()` - Begins background music playback
- `MusicPlayer::stop()` - Stops playback
- `MusicPlayer::update_config()` - Updates settings while running
- `MusicPlayer::get_config()` - Retrieves current settings

The player automatically:
- Discovers audio files in the target directory
- Cycles through all found files in order
- Respects fade and delay settings
- Cleans up processes on exit
