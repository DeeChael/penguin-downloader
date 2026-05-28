use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::error::CoreError;
use crate::models::*;
use crate::traits::MusicProvider;

pub type Result<T> = std::result::Result<T, CoreError>;

pub struct PenguinDownloader {
    music_provider: Arc<dyn MusicProvider>,
    credential: Option<String>,
    unknown_artists: Mutex<String>,
    disabled_qualities: Mutex<HashSet<i64>>,
}

impl PenguinDownloader {
    pub fn new(music_provider: Arc<dyn MusicProvider>, credential: Option<String>) -> Self {
        Self {
            music_provider,
            credential,
            unknown_artists: Mutex::new("unknown".to_string()),
            disabled_qualities: Mutex::new(HashSet::new()),
        }
    }

    pub fn get_music_provider(&self) -> &dyn MusicProvider {
        &*self.music_provider
    }

    pub fn get_unknown_artists(&self) -> String {
        self.unknown_artists.lock().unwrap().clone()
    }

    pub fn set_unknown_artists(&self, content: &str) {
        *self.unknown_artists.lock().unwrap() = content.to_string();
    }

    pub fn disable_quality(&self, quality: i64) {
        self.disabled_qualities.lock().unwrap().insert(quality);
    }

    pub fn enable_quality(&self, quality: i64) {
        self.disabled_qualities.lock().unwrap().remove(&quality);
    }

    fn resolve_quality(&self, song: &SongInfo, preferred: &PreferredQuality) -> Result<i64> {
        let disabled = self.disabled_qualities.lock().unwrap();

        let available: Vec<i64> = song
            .qualities
            .iter()
            .filter(|q| !disabled.contains(q))
            .copied()
            .collect();

        if available.is_empty() {
            return Err(CoreError::UnsupportedQuality);
        }

        match preferred {
            PreferredQuality::Highest => available.into_iter().max().ok_or(CoreError::UnsupportedQuality),
            PreferredQuality::Lowest => available.into_iter().min().ok_or(CoreError::UnsupportedQuality),
            PreferredQuality::Specific(q) => {
                if available.contains(q) {
                    Ok(*q)
                } else {
                    let lower: Vec<i64> = available.iter().filter(|&&a| a < *q).copied().collect();
                    if !lower.is_empty() {
                        Ok(lower.into_iter().max().unwrap())
                    } else {
                        let higher: Vec<i64> = available.iter().filter(|&&a| a > *q).copied().collect();
                        if !higher.is_empty() {
                            Ok(higher.into_iter().min().unwrap())
                        } else {
                            Err(CoreError::UnsupportedQuality)
                        }
                    }
                }
            }
        }
    }

    fn format_file_name(&self, fmt: &str, song: &SongInfo) -> String {
        let mut result = fmt.to_string();

        result = result.replace("{title}", &song.title);

        let artists_default = song
            .artists
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join("/");
        result = result.replace("{artists}", &artists_default);

        loop {
            let start = result.find("{artists:");
            match start {
                None => break,
                Some(s) => {
                    let after_prefix = &result[s + 9..];
                    let end = after_prefix.find('}');
                    match end {
                        None => break,
                        Some(e) => {
                            let sep = &after_prefix[..e];
                            let replacement = song
                                .artists
                                .iter()
                                .map(|a| a.name.as_str())
                                .collect::<Vec<_>>()
                                .join(sep);
                            let range = s..s + 9 + e + 1;
                            result.replace_range(range, &replacement);
                        }
                    }
                }
            }
        }

        if let Some(ref album) = song.album {
            result = result.replace("{album}", &album.title);
        } else {
            result = result.replace("{album}", "unknown_album");
        }

        if let Some(ref track) = song.track_number {
            result = result.replace("{track}", track);
        } else {
            result = result.replace("{track}", "unknown");
        }

        if let Some(ref disc) = song.disc_number {
            result = result.replace("{disc}", disc);
        } else {
            result = result.replace("{disc}", "unknown");
        }

        if let Ok(info) = self.music_provider.info() {
            result = result.replace("{provider}", info.id());
        }

        result = self.music_provider.format_file_name(&result);

        let invalid_chars: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
        result = result
            .chars()
            .map(|c| {
                if invalid_chars.contains(&c) || c == '\0' {
                    '_'
                } else {
                    c
                }
            })
            .collect();

        result.trim().to_string()
    }

    async fn download_file(
        &self,
        url: &str,
        path: &Path,
        referer: Option<&str>,
        callbacks: &Option<DownloadCallbacks>,
        current: i32,
        total: i32,
        song: &SongInfo,
        quality_label: Option<String>,
    ) -> Result<u64> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| CoreError::Custom(e.to_string()))?;

        let mut req = client.get(url);
        if let Some(r) = referer {
            req = req.header("Referer", r);
        }

        let response = req
            .send()
            .await
            .map_err(|e| CoreError::Custom(format!("HTTP request failed: {}", e)))?;

        let total_size = response.content_length();

        if let Some(ref cbs) = callbacks {
            if let Some(on_start) = cbs.on_start {
                on_start(&DownloadStart {
                    current,
                    total,
                    song: song.clone(),
                    quality: quality_label.clone(),
                });
            }
        }

        let parent = path.parent().unwrap_or(Path::new("."));
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| CoreError::Custom(format!("Failed to create directory: {}", e)))?;

        let mut file = tokio::fs::File::create(path)
            .await
            .map_err(|e| CoreError::Custom(format!("Failed to create file: {}", e)))?;

        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        let mut last_progress = Instant::now();

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|e| CoreError::Custom(format!("Download stream error: {}", e)))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| CoreError::Custom(format!("File write error: {}", e)))?;
            downloaded += chunk.len() as u64;

            if last_progress.elapsed().as_millis() >= 100 {
                if let Some(ref cbs) = callbacks {
                    if let Some(on_progress) = cbs.on_progress {
                        on_progress(&DownloadProgress {
                            current,
                            total,
                            song: song.clone(),
                            quality: quality_label.clone(),
                            downloaded,
                            total_size,
                        });
                    }
                }
                last_progress = Instant::now();
            }
        }

        if let Some(ref cbs) = callbacks {
            if let Some(on_progress) = cbs.on_progress {
                on_progress(&DownloadProgress {
                    current,
                    total,
                    song: song.clone(),
                    quality: quality_label.clone(),
                    downloaded,
                    total_size,
                });
            }
            if let Some(on_complete) = cbs.on_complete {
                on_complete(&DownloadComplete {
                    current,
                    total,
                    song: song.clone(),
                    quality: quality_label,
                    final_size: downloaded,
                    path: path.to_path_buf(),
                });
            }
        }

        Ok(downloaded)
    }

    async fn download_lyrics(
        &self,
        song: &SongInfo,
        options: &DownloadOptions,
        folder: &str,
        fmt: &str,
    ) -> Result<()> {
        let want_verbatim = options.lyrics == LyricsType::Verbatim;
        let translation = options.lyrics_translation;
        let roma = options.lyrics_roma;

        let lyrics = self
            .music_provider
            .get_lyrics(
                SongRef::Info(song.clone()),
                want_verbatim,
                translation,
                roma,
                self.credential.clone(),
            )
            .await;

        let lyrics = match lyrics {
            Ok(l) => l,
            Err(_) if want_verbatim => {
                self.music_provider
                    .get_lyrics(
                        SongRef::Info(song.clone()),
                        false,
                        translation,
                        roma,
                        self.credential.clone(),
                    )
                    .await?
            }
            Err(e) => return Err(e),
        };

        let folder_path = Path::new(folder);

        if want_verbatim {
            let ext = self
                .music_provider
                .get_verbatim_provider()
                .ok()
                .flatten()
                .map(|p| p.get_extension_name())
                .unwrap_or_else(|| "vtt".to_string());

            if let Some(ref text) = lyrics.verbatim {
                let p = folder_path.join(format!("{}.{}", fmt, ext));
                let _ = tokio::fs::write(&p, text).await;
            }
            if let Some(ref text) = lyrics.trans_verbatim {
                let p = folder_path.join(format!("{}_trans.{}", fmt, ext));
                let _ = tokio::fs::write(&p, text).await;
            }
            if let Some(ref text) = lyrics.roma_verbatim {
                let p = folder_path.join(format!("{}_roma.{}", fmt, ext));
                let _ = tokio::fs::write(&p, text).await;
            }
        } else {
            if let Some(ref text) = lyrics.lrc {
                let p = folder_path.join(format!("{}.lrc", fmt));
                let _ = tokio::fs::write(&p, text).await;
            }
            if let Some(ref text) = lyrics.trans {
                let p = folder_path.join(format!("{}_trans.lrc", fmt));
                let _ = tokio::fs::write(&p, text).await;
            }
            if let Some(ref text) = lyrics.roma {
                let p = folder_path.join(format!("{}_roma.lrc", fmt));
                let _ = tokio::fs::write(&p, text).await;
            }
        }

        Ok(())
    }

    async fn download_song_collection(
        &self,
        songs: Vec<SongInfo>,
        options: DownloadOptions,
        folder_path: Option<String>,
        callbacks: Option<DownloadCallbacks>,
    ) {
        let total = songs.len() as i32;
        let folder = folder_path.unwrap_or_else(|| ".".to_string());
        let fmt = options
            .format
            .clone()
            .unwrap_or_else(|| "{artists} - {title}".to_string());

        let mut song_quality_pairs: Vec<(SongInfo, i64)> = Vec::new();
        for song in &songs {
            match self.resolve_quality(song, &options.preferred_quality) {
                Ok(q) => song_quality_pairs.push((song.clone(), q)),
                Err(e) => {
                    if let Some(ref cbs) = callbacks {
                        if let Some(on_error) = cbs.on_error {
                            on_error(&DownloadError {
                                current: 0,
                                total,
                                song: song.clone(),
                                error_message: format!("Quality resolution failed: {}", e),
                            });
                        }
                    }
                }
            }
        }

        let mut all_urls: HashMap<String, String> = HashMap::new();
        for chunk in song_quality_pairs.chunks(100) {
            let batch_map: HashMap<SongInfo, i64> = chunk.iter().cloned().collect();
            match self
                .music_provider
                .get_song_urls(batch_map, self.credential.clone())
                .await
            {
                Ok(urls) => {
                    all_urls.extend(urls);
                }
                Err(e) => {
                    for (song, _) in chunk {
                        if let Some(ref cbs) = callbacks {
                            if let Some(on_error) = cbs.on_error {
                                on_error(&DownloadError {
                                    current: 0,
                                    total,
                                    song: song.clone(),
                                    error_message: format!("Failed to get URLs: {}", e),
                                });
                            }
                        }
                    }
                }
            }
        }

        let referer = self.music_provider.get_download_referer().ok();

        for (i, (song, quality)) in song_quality_pairs.iter().enumerate() {
            let current = (i + 1) as i32;

            let url = match all_urls.get(&song.id) {
                Some(u) => u.clone(),
                None => {
                    if let Some(ref cbs) = callbacks {
                        if let Some(on_error) = cbs.on_error {
                            on_error(&DownloadError {
                                current,
                                total,
                                song: song.clone(),
                                error_message: "No URL returned for song".to_string(),
                            });
                        }
                    }
                    continue;
                }
            };

            let filename = self.format_file_name(&fmt, song);
            let file_path = Path::new(&folder).join(&filename);

            if file_path.exists() && !options.force {
                if let Some(&expected_size) = song.size.get(quality) {
                    let actual_size = match tokio::fs::metadata(&file_path).await {
                        Ok(m) => m.len(),
                        Err(_) => 0,
                    };
                    if actual_size != expected_size {
                        if let Some(cb) = options.on_size_mismatch {
                            match cb(actual_size, expected_size) {
                                n if n > 0 => {}
                                0 => {
                                    if let Some(ref cbs) = callbacks {
                                        if let Some(on_existing) = cbs.on_existing {
                                            on_existing(&DownloadExisting {
                                                current,
                                                total,
                                                song: song.clone(),
                                                path: file_path.clone(),
                                                size: actual_size,
                                            });
                                        }
                                    }
                                    continue;
                                }
                                _ => {
                                    if let Some(ref cbs) = callbacks {
                                        if let Some(on_error) = cbs.on_error {
                                            on_error(&DownloadError {
                                                current,
                                                total,
                                                song: song.clone(),
                                                error_message: format!(
                                                    "File size mismatch: expected {}, actual {}",
                                                    expected_size, actual_size
                                                ),
                                            });
                                        }
                                    }
                                    continue;
                                }
                            }
                        }
                    }
                }
            }

            match self
                .download_file(
                    &url,
                    &file_path,
                    referer.as_deref(),
                    &callbacks,
                    current,
                    total,
                    song,
                    Some(quality.to_string()),
                )
                .await
            {
                Ok(_) => {
                    if options.lyrics != LyricsType::None {
                        let _ = self
                            .download_lyrics(song, &options, &folder, &fmt)
                            .await;
                    }
                }
                Err(e) => {
                    if let Some(ref cbs) = callbacks {
                        if let Some(on_error) = cbs.on_error {
                            on_error(&DownloadError {
                                current,
                                total,
                                song: song.clone(),
                                error_message: format!("Download failed: {}", e),
                            });
                        }
                    }
                }
            }
        }
    }

    pub async fn download_single(
        &self,
        song: SongInfo,
        options: DownloadOptions,
        folder_path: Option<String>,
        callbacks: Option<DownloadCallbacks>,
    ) {
        let total = 1i32;
        let current = 1i32;
        let folder = folder_path.unwrap_or_else(|| ".".to_string());
        let fmt = options
            .format
            .clone()
            .unwrap_or_else(|| "{artists} - {title}".to_string());

        let quality = match self.resolve_quality(&song, &options.preferred_quality) {
            Ok(q) => q,
            Err(e) => {
                if let Some(ref cbs) = callbacks {
                    if let Some(on_error) = cbs.on_error {
                        on_error(&DownloadError {
                            current,
                            total,
                            song: song.clone(),
                            error_message: format!("Quality resolution failed: {}", e),
                        });
                    }
                }
                return;
            }
        };

        let mut song_quality_map = HashMap::new();
        song_quality_map.insert(song.clone(), quality);

        let urls = match self
            .music_provider
            .get_song_urls(song_quality_map, self.credential.clone())
            .await
        {
            Ok(u) => u,
            Err(e) => {
                if let Some(ref cbs) = callbacks {
                    if let Some(on_error) = cbs.on_error {
                        on_error(&DownloadError {
                            current,
                            total,
                            song: song.clone(),
                            error_message: format!("Failed to get song URL: {}", e),
                        });
                    }
                }
                return;
            }
        };

        let url = match urls.get(&song.id) {
            Some(u) => u.clone(),
            None => {
                if let Some(ref cbs) = callbacks {
                    if let Some(on_error) = cbs.on_error {
                        on_error(&DownloadError {
                            current,
                            total,
                            song: song.clone(),
                            error_message: "No URL returned for song".to_string(),
                        });
                    }
                }
                return;
            }
        };

        let filename = self.format_file_name(&fmt, &song);
        let file_path = Path::new(&folder).join(&filename);

        if file_path.exists() && !options.force {
            if let Some(&expected_size) = song.size.get(&quality) {
                let actual_size = match tokio::fs::metadata(&file_path).await {
                    Ok(m) => m.len(),
                    Err(_) => 0,
                };
                if actual_size != expected_size {
                    if let Some(cb) = options.on_size_mismatch {
                        match cb(actual_size, expected_size) {
                            n if n > 0 => {}
                            0 => {
                                if let Some(ref cbs) = callbacks {
                                    if let Some(on_existing) = cbs.on_existing {
                                        on_existing(&DownloadExisting {
                                            current,
                                            total,
                                            song: song.clone(),
                                            path: file_path.clone(),
                                            size: actual_size,
                                        });
                                    }
                                }
                                return;
                            }
                            _ => {
                                if let Some(ref cbs) = callbacks {
                                    if let Some(on_error) = cbs.on_error {
                                        on_error(&DownloadError {
                                            current,
                                            total,
                                            song: song.clone(),
                                            error_message: format!(
                                                "File size mismatch: expected {}, actual {}",
                                                expected_size, actual_size
                                            ),
                                        });
                                    }
                                }
                                return;
                            }
                        }
                    }
                }
            }
        }

        let referer = self.music_provider.get_download_referer().ok();

        match self
            .download_file(
                &url,
                &file_path,
                referer.as_deref(),
                &callbacks,
                current,
                total,
                &song,
                Some(quality.to_string()),
            )
            .await
        {
            Ok(_) => {
                if options.lyrics != LyricsType::None {
                    let _ = self.download_lyrics(&song, &options, &folder, &fmt).await;
                }
            }
            Err(e) => {
                if let Some(ref cbs) = callbacks {
                    if let Some(on_error) = cbs.on_error {
                        on_error(&DownloadError {
                            current,
                            total,
                            song: song.clone(),
                            error_message: format!("Download failed: {}", e),
                        });
                    }
                }
            }
        }
    }

    pub async fn download_album(
        &self,
        album: AlbumRef,
        options: DownloadOptions,
        folder_path: Option<String>,
        callbacks: Option<DownloadCallbacks>,
    ) {
        let mut all_songs = Vec::new();
        let mut page = 1u64;
        let page_size = 100u64;

        loop {
            let pagination = Pagination { page_size, page };
            match self
                .music_provider
                .list_album_songs(album.clone(), Some(pagination), self.credential.clone())
                .await
            {
                Ok(result) => {
                    let count = result.results.len() as u64;
                    all_songs.extend(result.results);
                    if count < page_size {
                        break;
                    }
                    page += 1;
                }
                Err(e) => {
                    if let Some(ref cbs) = callbacks {
                        if let Some(on_error) = cbs.on_error {
                            on_error(&DownloadError {
                                current: 0,
                                total: 0,
                                song: SongInfo {
                                    id: String::new(),
                                    title: format!("Failed to list album songs: {}", e),
                                    artists: Vec::new(),
                                    album: None,
                                    cover: None,
                                    duration: None,
                                    published_date: None,
                                    track_number: None,
                                    disc_number: None,
                                    qualities: Vec::new(),
                                    size: HashMap::new(),
                                    subtitle: None,
                                    extras: HashMap::new(),
                                },
                                error_message: format!("Failed to list album songs: {}", e),
                            });
                        }
                    }
                    return;
                }
            }
        }

        self.download_song_collection(all_songs, options, folder_path, callbacks)
            .await;
    }

    pub async fn download_playlist(
        &self,
        playlist: PlaylistRef,
        options: DownloadOptions,
        folder_path: Option<String>,
        callbacks: Option<DownloadCallbacks>,
    ) {
        let mut all_songs = Vec::new();
        let mut page = 1u64;
        let page_size = 100u64;

        loop {
            let pagination = Pagination { page_size, page };
            match self
                .music_provider
                .list_playlist_songs(
                    playlist.clone(),
                    Some(pagination),
                    self.credential.clone(),
                )
                .await
            {
                Ok(result) => {
                    let count = result.results.len() as u64;
                    all_songs.extend(result.results);
                    if count < page_size {
                        break;
                    }
                    page += 1;
                }
                Err(e) => {
                    if let Some(ref cbs) = callbacks {
                        if let Some(on_error) = cbs.on_error {
                            on_error(&DownloadError {
                                current: 0,
                                total: 0,
                                song: SongInfo {
                                    id: String::new(),
                                    title: format!("Failed to list playlist songs: {}", e),
                                    artists: Vec::new(),
                                    album: None,
                                    cover: None,
                                    duration: None,
                                    published_date: None,
                                    track_number: None,
                                    disc_number: None,
                                    qualities: Vec::new(),
                                    size: HashMap::new(),
                                    subtitle: None,
                                    extras: HashMap::new(),
                                },
                                error_message: format!("Failed to list playlist songs: {}", e),
                            });
                        }
                    }
                    return;
                }
            }
        }

        self.download_song_collection(all_songs, options, folder_path, callbacks)
            .await;
    }
}
