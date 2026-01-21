use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use walkdir::WalkDir;
use rand::seq::SliceRandom;

/// Find the web directory by searching upward from the current directory
pub fn find_web_dir() -> PathBuf {
    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    loop {
        let web_path = current.join("web");
        if web_path.exists() && web_path.is_dir() {
            return current;
        }
        if !current.pop() {
            break;
        }
    }
    
    // Fallback to current directory
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Get the path to a resource relative to the web directory
pub fn music_dir_path() -> PathBuf {
    find_web_dir().join("web/music")
}

/// Configuration for music playback
#[derive(Clone)]
pub struct MusicConfig {
    /// Fade duration in milliseconds when transitioning between songs
    pub fade_duration_ms: u64,
    /// Delay in milliseconds between songs
    pub delay_between_songs_ms: u64,
    /// Volume level (0.0 to 1.0)
    pub volume: f32,
}

impl Default for MusicConfig {
    fn default() -> Self {
        Self {
            fade_duration_ms: 1000,
            delay_between_songs_ms: 2000,
            volume: 0.5,
        }
    }
}

/// Music player for background music using rodio
pub struct MusicPlayer {
    config: Arc<Mutex<MusicConfig>>,
    music_files: Vec<PathBuf>,
    is_running: Arc<Mutex<bool>>,
}

impl MusicPlayer {
    /// Create a new music player that loads music from the specified directory
    pub fn new(music_dir: &str, config: MusicConfig) -> Self {
        let music_files = Self::load_music_files(music_dir);
        
        if music_files.is_empty() {
            println!("No music files found in {}", music_dir);
        } else {
            println!("Loaded {} music file(s) for background playback", music_files.len());
        }

        Self {
            config: Arc::new(Mutex::new(config)),
            music_files,
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Load all audio files from a directory (recursively)
    fn load_music_files(music_dir: &str) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let supported_extensions = ["mp3", "wav", "flac", "ogg"];

        for entry in WalkDir::new(music_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file())
        {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if supported_extensions.contains(&ext_str.to_lowercase().as_str()) {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }

        // Shuffle for random playback order
        files.shuffle(&mut rand::thread_rng());
        files
    }

    /// Start playing background music in a background thread
    pub fn start(&self) {
        if self.music_files.is_empty() {
            return;
        }

        let music_files = self.music_files.clone();
        let config = Arc::clone(&self.config);
        let is_running = Arc::clone(&self.is_running);

        *is_running.lock().unwrap() = true;

        thread::spawn(move || {
            // Try to create output stream, but don't fail if no audio device is available
            let audio_available = OutputStream::try_default().is_ok();
            
            if audio_available {
                if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
                    if let Ok(sink) = Sink::try_new(&stream_handle) {
                        let mut current_index = 0;

                        while *is_running.lock().unwrap() {
                            let current_file = &music_files[current_index % music_files.len()];

                            // Load and play the file
                            if let Ok(file) = File::open(current_file) {
                                let reader = BufReader::new(file);
                                if let Ok(source) = Decoder::new(reader) {
                                    let config_lock = config.lock().unwrap();
                                    let volume = config_lock.volume;
                                    let fade_duration = config_lock.fade_duration_ms;
                                    let delay_ms = config_lock.delay_between_songs_ms;
                                    drop(config_lock);

                                    // Set volume and add source to sink
                                    sink.set_volume(volume);
                                    sink.append(source);
                                    
                                    // Wait for playback to complete
                                    sink.sleep_until_end();

                                    // Fade-out effect by reducing volume gradually
                                    if fade_duration > 0 {
                                        let steps = 20;
                                        let step_duration = Duration::from_millis(fade_duration / steps);
                                        
                                        for i in 1..=steps {
                                            let progress = i as f32 / steps as f32;
                                            let new_volume = volume * (1.0 - progress);
                                            sink.set_volume(new_volume.max(0.0));
                                            thread::sleep(step_duration);
                                        }
                                        
                                        sink.set_volume(0.0);
                                    }

                                    // Clear the sink for next song
                                    sink.clear();
                                    sink.set_volume(volume); // Reset volume

                                    // Delay before next song
                                    if delay_ms > 0 {
                                        thread::sleep(Duration::from_millis(delay_ms));
                                    }
                                }
                            }

                            current_index += 1;
                        }

                        sink.stop();
                    }
                }
            } else {
                // No audio device available - simulate playback by waiting for song durations
                let mut current_index = 0;
                
                while *is_running.lock().unwrap() {
                    let current_file = &music_files[current_index % music_files.len()];
                    
                    // Try to estimate song duration by reading metadata
                    if let Ok(file) = File::open(current_file) {
                        let reader = BufReader::new(file);
                        if let Ok(source) = Decoder::new(reader) {
                            if let Some(duration) = source.total_duration() {
                                // Simulate playback by sleeping for the song duration
                                thread::sleep(duration);
                            } else {
                                // If we can't get duration, default to 3 minutes
                                thread::sleep(Duration::from_secs(180));
                            }
                        } else {
                            // If we can't decode, skip
                            thread::sleep(Duration::from_millis(100));
                        }
                    }
                    
                    // Apply fade and delay settings
                    let config_lock = config.lock().unwrap();
                    let fade_duration = config_lock.fade_duration_ms;
                    let delay_ms = config_lock.delay_between_songs_ms;
                    drop(config_lock);
                    
                    if fade_duration > 0 {
                        thread::sleep(Duration::from_millis(fade_duration));
                    }
                    
                    if delay_ms > 0 {
                        thread::sleep(Duration::from_millis(delay_ms));
                    }
                    
                    current_index += 1;
                }
            }
        });
    }

    /// Stop playing music
    pub fn stop(&self) {
        *self.is_running.lock().unwrap() = false;
    }

    /// Update the music configuration
    pub fn update_config(&self, config: MusicConfig) {
        *self.config.lock().unwrap() = config;
    }

    /// Get the current music configuration
    pub fn get_config(&self) -> MusicConfig {
        self.config.lock().unwrap().clone()
    }
}

impl Drop for MusicPlayer {
    fn drop(&mut self) {
        self.stop();
        // Give the thread a moment to shut down cleanly
        thread::sleep(Duration::from_millis(100));
    }
}

