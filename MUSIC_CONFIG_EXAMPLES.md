// Example Music Configurations
// Copy these configurations to engine/src/main.rs and replace the MusicConfig block

// === Example 1: Quiet Background Music (Default) ===
let music_config = MusicConfig {
    fade_duration_ms: 1500,      // 1.5 second fade between songs
    delay_between_songs_ms: 2000, // 2 second delay between songs
    volume: 0.3,                  // 30% volume
};

// === Example 2: Cinematic with Long Fades ===
let music_config = MusicConfig {
    fade_duration_ms: 3000,       // 3 second fade between songs (cinematic)
    delay_between_songs_ms: 5000, // 5 second gap between songs
    volume: 0.4,                  // 40% volume
};

// === Example 3: Energetic - No Gap Between Songs ===
let music_config = MusicConfig {
    fade_duration_ms: 500,        // 0.5 second quick fade
    delay_between_songs_ms: 0,    // No gap, songs play immediately
    volume: 0.5,                  // 50% volume
};

// === Example 4: Loud and Intense ===
let music_config = MusicConfig {
    fade_duration_ms: 1000,       // 1 second fade
    delay_between_songs_ms: 1000, // 1 second between songs
    volume: 0.8,                  // 80% volume
};

// === Example 5: Ambient/Subtle ===
let music_config = MusicConfig {
    fade_duration_ms: 2000,       // 2 second fade
    delay_between_songs_ms: 3000, // 3 second gap
    volume: 0.2,                  // 20% volume (very quiet)
};

// === Example 6: Maximum Volume ===
let music_config = MusicConfig {
    fade_duration_ms: 500,        // 0.5 second fade
    delay_between_songs_ms: 1000, // 1 second between songs
    volume: 1.0,                  // 100% volume (maximum)
};

// === Example 7: No Fading (Abrupt Transitions) ===
let music_config = MusicConfig {
    fade_duration_ms: 0,          // No fade effect
    delay_between_songs_ms: 1000, // 1 second between songs
    volume: 0.5,                  // 50% volume
};

// === Example 8: Continuous Play ===
let music_config = MusicConfig {
    fade_duration_ms: 1000,       // 1 second fade
    delay_between_songs_ms: 0,    // No delay between songs
    volume: 0.4,                  // 40% volume
};
