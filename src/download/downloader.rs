use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use futures::StreamExt;
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use tracing::warn;

use crate::provider::{MusicProvider, Pagination};
use crate::model::*;
use crate::error::{Error, Result};

pub struct Downloader {
    provider: Arc<dyn MusicProvider>,
    credential: Option<String>,
    client: Client,
    base_output_dir: PathBuf,
    total_count: AtomicI32,
    success_count: AtomicI32,
    fail_count: AtomicI32,
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn sanitize_file_name(name: &str) -> String {
    let invalid_chars = regex::Regex::new(r#"[<>:"/\\|?*]"#).unwrap();
    let whitespace = regex::Regex::new(r"\s+").unwrap();
    let result = invalid_chars.replace_all(name, "_");
    whitespace.replace_all(&result, " ").trim().to_string()
}

impl Downloader {

    pub fn new(provider: Arc<dyn MusicProvider>, credential: Option<String>) -> Self {
        Self {
            provider,
            credential,
            client: Client::new(),
            base_output_dir: PathBuf::from("."),
            total_count: AtomicI32::new(0),
            success_count: AtomicI32::new(0),
            fail_count: AtomicI32::new(0),
        }
    }
    
    fn credential(&self) -> Option<&str> {
        self.credential.as_deref()
    }

    pub fn with_output_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.base_output_dir = dir.as_ref().to_path_buf();
        self
    }

    fn format_file_name(&self, format: &str, info: &SongInfo, extension: &str) -> String {
        // 1. 处理通用占位符
        let track_padded = format!("{:02}", info.track_number.unwrap_or(0));
        
        let sanitize = |s: &str| {
            let invalid_chars = regex::Regex::new(r#"[<>\":/\\|?*]"#).unwrap();
            let whitespace = regex::Regex::new(r"\s+").unwrap();
            let result = invalid_chars.replace_all(s, "_");
            whitespace.replace_all(&result, " ").trim().to_string()
        };
        
        let mut formatted = format
            .replace("{track}", &track_padded)
            .replace("{title}", &sanitize(&info.title))
            .replace("{artist}", &sanitize(info.artist.as_deref().unwrap_or("未知歌手")))
            .replace("{album}", &sanitize(info.album.as_deref().unwrap_or("未知专辑")))
            .replace("{provider}", &sanitize(self.provider.name()));
        
        // 2. 调用 provider 处理专属占位符
        formatted = self.provider.format_file_name_custom(&formatted, info);
        
        // 3. 添加扩展名
        formatted + "." + extension
    }

    async fn get_remote_file_size(&self, url: &str) -> Option<u64> {
        let referer = self.provider.download_referer();
        
        let mut request = self.client
            .head(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36");
        
        if !referer.is_empty() {
            request = request.header("Referer", referer);
        }
        
        let response = request.send().await.ok()?;
        
        if response.status().is_success() {
            response.headers()
                .get(reqwest::header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
        } else {
            None
        }
    }

    pub async fn download_song(&self, info: &SongInfo, options: &DownloadOptions) -> Result<PathBuf> {
        self.download_song_to_dir(info, options, &self.base_output_dir).await
    }

    pub async fn download_song_to_dir(&self, info: &SongInfo, options: &DownloadOptions, output_dir: &Path) -> Result<PathBuf> {
        let format = options.format.as_deref().unwrap_or("{title} - {artist}");
        let result = self.download_internal(info, output_dir, format, options).await?;
        self.success_count.fetch_add(1, Ordering::SeqCst);
        Ok(result)
    }

    async fn download_internal(
        &self,
        info: &SongInfo,
        output_dir: &Path,
        format: &str,
        options: &DownloadOptions,
    ) -> Result<PathBuf> {
        let extensions = ["flac", "mp3", "m4a", "ogg"];

        if !options.force {
            for &ext in &extensions {
                let file_name = self.format_file_name(format, info, ext);
                let existing_file = output_dir.join(file_name);
                tracing::debug!("[下载] 检查文件是否存在: path={:?}", existing_file);
                if existing_file.exists() {
                    tracing::info!("[下载] 文件已存在: path={:?}", existing_file);
                    if options.on_size_mismatch.is_none() {
                        let size = existing_file.metadata()?.len();
                        return Err(Error::AlreadyExists { path: existing_file });
                    }
                }
            }
        }

        let url_result = self.provider.get_song_url(&info.id, options.quality, self.credential()).await?
            .ok_or(Error::UrlNotFound)?;

        if !url_result.is_success() {
            return Err(Error::UrlNotFound);
        }

        let url = url_result.url.as_ref().ok_or(Error::UrlNotFound)?;
        let extension = Self::detect_extension(url);
        let file_name = self.format_file_name(format, info, extension);
        
        if !output_dir.exists() {
            tokio::fs::create_dir_all(output_dir).await?;
        }
        
        let output_path = output_dir.join(&file_name);

        if !options.force && options.on_size_mismatch.is_some() && output_path.exists() {
            let expected_size = self.get_remote_file_size(url).await;
            let actual_size = output_path.metadata()?.len();
            
            if let Some(expected) = expected_size {
                let tolerance = 20 * 1024u64;
                if actual_size.abs_diff(expected) > tolerance {
                    let should_redownload = options.on_size_mismatch.unwrap()(expected, actual_size);
                    if !should_redownload {
                        return Err(Error::SizeMismatch { expected, actual: actual_size });
                    }
                } else {
                    return Err(Error::AlreadyExists { path: output_path });
                }
            }
        }
        
        let quality_name = url_result.quality.as_deref().unwrap_or("未知音质");
        
        self.download_file_with_progress(url, &output_path, info, quality_name, info.track_number.unwrap_or(0), 1, options).await?;
        
        let cover_data = if let Some(ref cover_url) = info.cover {
            self.download_image(cover_url).await
        } else {
            None
        };
        
        self.embed_metadata(&output_path, cover_data.as_deref(), info).await?;
        
        if options.lyric_type != crate::model::LyricType::None {
            let final_file_name = output_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            self.download_lyric(info, output_dir, final_file_name, options).await;
        }
        
        Ok(output_path)
    }

    async fn download_file_with_progress(
        &self,
        url: &str,
        output_path: &Path,
        info: &SongInfo,
        quality_name: &str,
        track_number: i32,
        total: i32,
        options: &DownloadOptions,
    ) -> Result<()> {
        let referer = self.provider.download_referer();
        
        let mut request = self.client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36");
        
        if !referer.is_empty() {
            request = request.header("Referer", referer);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Error::DownloadFailed(format!("HTTP error: {}", response.status())));
        }

        // Get total size from Content-Length header
        let total_size = response.headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        // Call on_start callback
        if let Some(callback) = options.callbacks.on_start {
            callback(&crate::model::DownloadStart {
                current: track_number,
                total,
                song: info.clone(),
                quality: Some(quality_name.to_string()),
            });
        }

        let mut file = tokio::fs::File::create(output_path).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut last_reported_progress: u64 = 0;
        let progress_threshold = total_size.map(|s| (s / 100).min(1024 * 1024)).unwrap_or(1024 * 1024);

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let chunk_size = chunk.len() as u64;
            file.write_all(&chunk).await?;
            downloaded += chunk_size;

            if let Some(callback) = options.callbacks.on_progress {
                if downloaded - last_reported_progress >= progress_threshold || downloaded == total_size.unwrap_or(0) {
                    last_reported_progress = downloaded;
                    callback(&crate::model::DownloadProgress {
                        current: track_number,
                        total,
                        song: info.clone(),
                        quality: Some(quality_name.to_string()),
                        downloaded,
                        total_size,
                    });
                }
            }
        }

        file.flush().await?;

        if let Some(callback) = options.callbacks.on_complete {
            callback(&crate::model::DownloadComplete {
                current: track_number,
                total,
                song: info.clone(),
                quality: Some(quality_name.to_string()),
                final_size: downloaded,
                path: output_path.to_path_buf(),
            });
        }

        Ok(())
    }

    async fn download_image(&self, url: &str) -> Option<Vec<u8>> {
        let referer = self.provider.download_referer();
        
        let mut request = self.client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36");
        
        if !referer.is_empty() {
            request = request.header("Referer", referer);
        }
        
        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    response.bytes().await.ok().map(|b| b.to_vec())
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    async fn embed_metadata(&self, file_path: &Path, cover_data: Option<&[u8]>, info: &SongInfo) -> Result<()> {
        use lofty::prelude::*;
        use lofty::file::AudioFile;
        
        if let Ok(mut audio_file) = lofty::read_from_path(file_path) {
            let tag_opt = if audio_file.primary_tag().is_some() {
                audio_file.primary_tag_mut()
            } else {
                audio_file.first_tag_mut()
            };
            
            if let Some(tag) = tag_opt {
                tag.insert_text(ItemKey::TrackTitle, info.title.clone());
                
                if let Some(ref artist) = info.artist {
                    tag.insert_text(ItemKey::TrackArtist, artist.clone());
                }
                
                if let Some(ref album) = info.album {
                    tag.insert_text(ItemKey::AlbumTitle, album.clone());
                }
                
                if let Some(track_num) = info.track_number {
                    if track_num > 0 {
                        tag.insert_text(ItemKey::TrackNumber, track_num.to_string());
                    }
                }
                
                if let Some(disc_num) = info.disc_number {
                    if disc_num > 0 {
                        tag.insert_text(ItemKey::DiscNumber, disc_num.to_string());
                    }
                }
                
                if let Some(ref date) = info.publish_date {
                    tag.insert_text(ItemKey::ReleaseDate, date.clone());
                }
                
                if let Some(cover) = cover_data {
                    let picture = lofty::picture::Picture::new_unchecked(
                        lofty::picture::PictureType::CoverFront,
                        Some(lofty::picture::MimeType::Jpeg),
                        None,
                        cover.to_vec(),
                    );
                    tag.push_picture(picture);
                }
                
                let _ = audio_file.save_to_path(file_path, lofty::config::WriteOptions::default());
            }
        }
        
        Ok(())
    }

    async fn download_lyric(
        &self,
        info: &SongInfo,
        output_dir: &Path,
        base_name: &str,
        options: &DownloadOptions,
    ) {
        if options.lyric_type == crate::model::LyricType::None {
            return;
        }

        let verbatim = options.lyric_type == crate::model::LyricType::Verbatim;
        
        match self
            .provider
            .get_lyric(&info.id, verbatim, options.lyric_translation, options.lyric_romanization, self.credential())
            .await
        {
            Ok(Some(lyric)) => {
                let lyric_content = if verbatim {
                    lyric.verbatim
                } else {
                    lyric.lrc
                };
                
                if let Some(content) = lyric_content {
                    let ext = if verbatim {
                        let ext = self.provider.verbatim_lyric_extension();
                        if ext.is_empty() { "verbatim" } else { ext }
                    } else {
                        "lrc"
                    };
                    let lrc_path = output_dir.join(format!("{}.{}", base_name, ext));
                    if !lrc_path.exists() {
                        if let Ok(mut file) = tokio::fs::File::create(&lrc_path).await {
                            let _ = file.write_all(content.as_bytes()).await;
                        }
                    }
                }
            }
            Ok(None) => {}
            Err(e) => {
                tracing::warn!("[下载] 歌词下载失败: {}", e);
            }
        }
    }

    fn detect_extension(url: &str) -> &'static str {
        if url.contains(".flac") { "flac" }
        else if url.contains(".m4a") { "m4a" }
        else if url.contains(".ogg") { "ogg" }
        else { "mp3" }
    }

    pub async fn download_playlist(&self, playlist_id: &str, options: &mut DownloadOptions) -> Result<Vec<PathBuf>> {
        let pagination = Pagination::default_list();
        let playlist = self.provider.get_playlist_songs(playlist_id, pagination, self.credential()).await?
            .ok_or_else(|| Error::PlaylistNotFound(playlist_id.to_string()))?;
        
        let total = playlist.songs.len() as i32;
        self.total_count.store(total, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.fail_count.store(0, Ordering::SeqCst);
        
        let playlist_name = if playlist.title.is_empty() { "未知歌单" } else { &playlist.title };
        let output_dir = self.base_output_dir.join("playlists").join(sanitize_file_name(playlist_name));
        if options.format.is_none() {
            options.format = Some("{title} - {artist}".to_string());
        }
        
        self.download_sequential(&playlist.songs, &output_dir, options).await
    }

    pub async fn download_album(&self, album_id: &str, options: &mut DownloadOptions) -> Result<Vec<PathBuf>> {
        let pagination = Pagination::default_list();
        let songs = self.provider.get_album_songs(album_id, pagination, self.credential()).await?;
        
        if songs.is_empty() {
            return Err(Error::AlbumNotFound(album_id.to_string()));
        }
        
        let total = songs.len() as i32;
        self.total_count.store(total, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.fail_count.store(0, Ordering::SeqCst);
        
        let album_name = songs.first()
            .and_then(|s| s.album.as_ref())
            .map(|n| n.as_str())
            .unwrap_or("未知专辑");
        
        let output_dir = self.base_output_dir.join("albums").join(sanitize_file_name(album_name));
        if options.format.is_none() {
            options.format = Some("{track} {title}".to_string());
        }
        
        self.download_sequential(&songs, &output_dir, options).await
    }

    async fn download_sequential(
        &self,
        songs: &[SongInfo],
        output_dir: &Path,
        options: &DownloadOptions,
    ) -> Result<Vec<PathBuf>> {
        let mut results = Vec::new();
        
        for song in songs.iter() {
            match self.download_song_to_dir(song, options, output_dir).await {
                Ok(path) => {
                    results.push(path);
                }
                Err(e) => {
                    warn!("[下载] 歌曲下载失败: title={}, id={}, error={}", song.title, song.id, e);
                }
            }
        }
        
        Ok(results)
    }

    pub async fn download_song_by_id(&self, id: &str, output_dir: &Path, options: &DownloadOptions) -> Result<PathBuf> {
        let info = self.provider.get_song_detail(id, self.credential()).await?
            .ok_or_else(|| Error::SongNotFound(id.to_string()))?;
        
        self.download_song_to_dir(&info, options, output_dir).await
    }

}
