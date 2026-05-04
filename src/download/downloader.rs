use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use futures::StreamExt;
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use tracing::warn;

use crate::provider::{MusicProvider, Pagination};
use crate::tagger::Tagger;
use crate::model::*;
use crate::error::{Error, Result};

/// 音乐下载器
///
/// 负责从音源提供者下载歌曲、专辑和歌单。
/// 支持自定义文件名格式、音质选择和元数据嵌入。
///
/// # 文件名格式
///
/// 可以使用以下占位符：
/// - `{title}` - 歌曲标题
/// - `{artist}` - 艺术家
/// - `{album}` - 专辑名
/// - `{track}` - 曲目编号（补零）
/// - `{provider}` - 音源名称
///
/// # Examples
///
/// ```rust,no_run
/// use penguin_downloader::{Downloader, DownloadOptions};
/// use std::sync::Arc;
///
/// # async fn example(provider: Arc<dyn penguin_downloader::MusicProvider>) -> anyhow::Result<()> {
/// let downloader = Downloader::new(provider, None);
///
/// // 获取歌曲信息后下载
/// let song = provider.search_songs("晴天", Default::default(), None).await?.songs[0].clone();
/// let options = DownloadOptions::default();
/// downloader.download_song(&song, &options).await?;
/// # Ok(())
/// # }
/// ```
pub struct Downloader {
    provider: Arc<dyn MusicProvider>,
    credential: Option<String>,
    client: Client,
    total_count: AtomicI32,
    success_count: AtomicI32,
    fail_count: AtomicI32,
    tagger: Option<Arc<dyn Tagger>>,
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
    let invalid_chars = regex::Regex::new(r#"[<>"/\\|?*]"#).unwrap();
    let whitespace = regex::Regex::new(r"\s+").unwrap();
    let result = invalid_chars.replace_all(name, "_");
    whitespace.replace_all(&result, " ").trim().to_string()
}

impl Downloader {
    /// 创建新的下载器实例
    ///
    /// # Arguments
    ///
    /// * `provider` - 音源提供者
    /// * `credential` - 可选的登录凭证（base64 编码）
    pub fn new(provider: Arc<dyn MusicProvider>, credential: Option<String>) -> Self {
        Self {
            provider,
            credential,
            client: Client::new(),
            total_count: AtomicI32::new(0),
            success_count: AtomicI32::new(0),
            fail_count: AtomicI32::new(0),
            tagger: None,
        }
    }
    
    fn credential(&self) -> Option<&str> {
        self.credential.as_deref()
    }

    /// 设置元数据标签器
    ///
    /// 用于在音源不提供元数据时，从其他来源获取
    pub fn use_tagger(mut self, tagger: Arc<dyn Tagger>) -> Self {
        self.tagger = Some(tagger);
        self
    }

    fn format_file_name(&self, format: &str, info: &SongInfo, extension: &str) -> String {
        let track_padded = format!("{:02}", info.track_number.unwrap_or(0));
        
        let sanitize = |s: &str| {
            let invalid_chars = regex::Regex::new(r#"[<>\":/\\|?*]"#).unwrap();
            let whitespace = regex::Regex::new(r"\s+").unwrap();
            let result = invalid_chars.replace_all(s, "_");
            whitespace.replace_all(&result, " ").trim().to_string()
        };
        
        // 处理 {artist} 和 {artist:分隔符}
        let artist_re = regex::Regex::new(r"\{artist(:[^}]*)?\}").unwrap();
        
        let mut formatted = artist_re.replace_all(format, |caps: &regex::Captures| {
            if info.artists.is_empty() {
                return "未知歌手".to_string();
            }
            let raw_sep = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            if raw_sep.is_empty() {
                // {artist} - 默认使用 " / "（sanitize 会清理非法字符，假设真有文件系统支持 /，岂不美哉？）
                let joined = info.artists.join(" / ");
                sanitize(&joined)
            } else {
                // {artist:X} - 使用 X 作为分隔符（sanitize 会清理非法字符）
                let sep = &raw_sep[1..]; // 去掉冒号
                let joined = info.artists.join(sep);
                sanitize(&joined)
            }
        }).to_string();
        
        let mut formatted = formatted
            .replace("{track}", &track_padded)
            .replace("{title}", &sanitize(&info.title))
            .replace("{album}", &sanitize(info.album.as_deref().unwrap_or("未知专辑")))
            .replace("{provider}", &sanitize(self.provider.name()));
        
        formatted = self.provider.format_file_name_custom(&formatted, info);
        
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
        self.download_song_to_dir(info, options, &PathBuf::from(".")).await
    }

    pub async fn download_song_to_dir(&self, info: &SongInfo, options: &DownloadOptions, output_dir: &Path
    ) -> Result<PathBuf> {
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
                        return Err(Error::AlreadyExists { path: existing_file });
                    }
                }
            }
        }

        let url_result = self.provider.get_song_url(&info.id, options.quality, self.credential()).await?;

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
        
        let mut final_info = info.clone();
        
        if !self.provider.has_metadata() {
            if let Some(ref tagger) = self.tagger {
                if let Some(metadata) = tagger.fetch_metadata(info).await {
                    if final_info.artists.is_empty() && !metadata.artists.is_empty() {
                        final_info.artists = metadata.artists.clone();
                    }
                    if final_info.album.is_none() && metadata.album.is_some() {
                        final_info.album = metadata.album.clone();
                    }
                    if final_info.cover.is_none() && metadata.cover.is_some() {
                        final_info.cover = metadata.cover.clone();
                    }
                    if final_info.subtitle.is_none() && metadata.subtitle.is_some() {
                        final_info.subtitle = metadata.subtitle.clone();
                    }
                    if final_info.publish_date.is_none() && metadata.publish_date.is_some() {
                        final_info.publish_date = metadata.publish_date.clone();
                    }
                    if final_info.track_number.is_none() && metadata.track_number.is_some() {
                        final_info.track_number = metadata.track_number;
                    }
                    if final_info.disc_number.is_none() && metadata.disc_number.is_some() {
                        final_info.disc_number = metadata.disc_number;
                    }
                }
            }
        }
        
        let cover_data = if let Some(ref cover_url) = final_info.cover {
            self.download_image(cover_url).await
        } else {
            None
        };
        
        self.embed_metadata(&output_path, cover_data.as_deref(), &final_info).await?;
        
        if options.lyric_type != crate::model::LyricType::None {
            let final_file_name = output_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            self.download_lyric(&final_info, output_dir, final_file_name, options).await;
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

        let total_size = response.headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

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

    async fn embed_metadata(&self, file_path: &Path, cover_data: Option<&[u8]>, info: &SongInfo
    ) -> Result<()> {
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
                
                let artist_str = info.artists.join("、");
                if !artist_str.is_empty() {
                    tag.insert_text(ItemKey::TrackArtist, artist_str);
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
            Ok(lyric) => {
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
            Err(crate::Error::NoDataExists) => {}
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

    pub async fn download_playlist(
        &self, playlist_id: &str,
        output_dir: &Path,
        options: &mut DownloadOptions
    ) -> Result<Vec<PathBuf>> {
        let pagination = Pagination::default_list();
        let playlist = self.provider.get_playlist_songs(playlist_id, pagination, self.credential()).await?;
        
        let total = playlist.songs.len() as i32;
        self.total_count.store(total, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.fail_count.store(0, Ordering::SeqCst);
        
        if options.format.is_none() {
            options.format = Some("{title} - {artist}".to_string());
        }
        
        self.download_sequential(&playlist.songs, &output_dir, options).await
    }

    pub async fn download_album(
        &self, album_id: &str,
        output_dir: &Path,
        options: &mut DownloadOptions
    ) -> Result<Vec<PathBuf>> {
        let pagination = Pagination::default_list();
        let songs = self.provider.get_album_songs(album_id, pagination, self.credential()).await?;
        
        if songs.is_empty() {
            return Err(Error::AlbumNotFound(album_id.to_string()));
        }
        
        let total = songs.len() as i32;
        self.total_count.store(total, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.fail_count.store(0, Ordering::SeqCst);
        
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

    pub async fn download_song_by_id(
        &self, id: &str, output_dir: &Path, options: &DownloadOptions
    ) -> Result<PathBuf> {
        let info = self.provider.get_song_detail(id, self.credential()).await?;
        
        self.download_song_to_dir(&info, options, output_dir).await
    }

}