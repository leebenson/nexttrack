use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use rand::seq::SliceRandom;
use rand::thread_rng;
use reqwest::header::AUTHORIZATION;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex as TokioMutex;
use tower_http::cors::{Any, CorsLayer};
use urlencoding;

macro_rules! hashmap {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = ::std::collections::HashMap::new();
        $(map.insert($key, $value);)*
        map
    }};
}

// Structs
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Preferences {
    energy: f64,
    obscurity: f64,
    mood: f64,
}

#[derive(Serialize, Deserialize, Clone)]
struct Track {
    id: String,
    name: String,
    artist: String,
    features: HashMap<String, f64>,
    popularity: u32,
    album_art: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TrackId {
    mbid: Option<String>,    // MusicBrainz ID (primary)
    spotify: Option<String>, // Spotify ID (for migration)
    name: String,            // Track name for fallback searches
    artist: String,          // Artist name for fallback searches
}

#[derive(Serialize, Deserialize, Debug)]
struct RecommendRequest {
    tracks: Vec<String>,
    preferences: Preferences,
}

#[derive(Serialize, Deserialize)]
struct SpotifySearchResponse {
    tracks: SpotifyItems,
}

#[derive(Serialize, Deserialize)]
struct SpotifyItems {
    items: Vec<SpotifyTrack>,
}

#[derive(Serialize, Deserialize)]
struct SpotifyTrack {
    id: String,
    name: String,
    artists: Vec<SpotifyArtist>,
    popularity: u32,
}

#[derive(Serialize, Deserialize)]
struct SpotifyArtist {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct SpotifyFeatures {
    energy: f64,
    valence: f64,
    tempo: f64,
}

#[derive(Serialize, Deserialize)]
struct GeniusSearchResponse {
    response: GeniusResponse,
}

#[derive(Serialize, Deserialize)]
struct GeniusResponse {
    hits: Vec<GeniusHit>,
}

#[derive(Serialize, Deserialize)]
struct GeniusHit {
    result: GeniusResult,
}

#[derive(Serialize, Deserialize)]
struct GeniusResult {
    path: String,
}

// MusicBrainz structs
#[derive(Deserialize)]
struct MusicBrainzSearchResponse {
    recordings: Vec<MusicBrainzRecording>,
}

// Cover Art Archive structs
#[derive(Deserialize)]
struct CoverArtArchiveResponse {
    images: Vec<CoverArtImage>,
}

#[derive(Deserialize)]
struct CoverArtImage {
    image: String, // URL to the image
    thumbnails: CoverArtThumbnails,
}

#[derive(Deserialize)]
struct CoverArtThumbnails {
    #[serde(rename = "250")]
    small: Option<String>,
    #[serde(rename = "500")]
    large: Option<String>,
}

#[derive(Deserialize, Clone)]
struct MusicBrainzRecording {
    id: String, // MBID
    title: String,
    #[serde(rename = "artist-credit")]
    artist_credit: Option<Vec<MusicBrainzArtistCredit>>,
    #[allow(dead_code)]
    releases: Option<Vec<MusicBrainzRelease>>,
}

#[derive(Deserialize, Clone)]
struct MusicBrainzArtistCredit {
    artist: MusicBrainzArtist,
}

#[derive(Deserialize, Clone)]
struct MusicBrainzArtist {
    #[allow(dead_code)]
    id: String,
    name: String,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
struct MusicBrainzRelease {
    id: String,
    title: String,
}

// AcousticBrainz structs
#[derive(Deserialize)]
struct AcousticBrainzResponse {
    highlevel: Option<AcousticBrainzHighLevel>,
    lowlevel: Option<AcousticBrainzLowLevel>,
}

#[derive(Deserialize)]
struct AcousticBrainzHighLevel {
    danceability: AcousticBrainzFeature,
    #[allow(dead_code)]
    mood_acoustic: AcousticBrainzFeature,
    mood_aggressive: AcousticBrainzFeature,
    mood_happy: AcousticBrainzFeature,
    #[allow(dead_code)]
    mood_sad: AcousticBrainzFeature,
}

#[derive(Deserialize)]
struct AcousticBrainzLowLevel {
    average_loudness: f64,
    bpm: f64,
    dynamic_complexity: f64,
}

#[derive(Deserialize)]
struct AcousticBrainzFeature {
    all: AcousticBrainzProbability,
}

#[derive(Deserialize)]
struct AcousticBrainzProbability {
    danceable: Option<f64>,
    #[allow(dead_code)]
    acoustic: Option<f64>,
    aggressive: Option<f64>,
    happy: Option<f64>,
    #[allow(dead_code)]
    sad: Option<f64>,
}

// ListenBrainz structs
#[derive(Serialize)]
struct ListenBrainzRecordingRequest {
    recording_mbids: Vec<String>,
}

#[derive(Deserialize)]
struct ListenBrainzPopularityResponse {
    payload: Vec<ListenBrainzRecordingPopularity>,
}

#[derive(Deserialize)]
struct ListenBrainzRecordingPopularity {
    recording_mbid: String,
    total_listen_count: Option<u64>,
    #[allow(dead_code)]
    total_user_count: Option<u64>,
}

// Last.fm structs
#[derive(Deserialize)]
struct LastFmSimilar {
    similartracks: LastFmSimilarTracks,
}

#[derive(Deserialize)]
struct LastFmSimilarTracks {
    track: Vec<LastFmTrack>,
}

#[derive(Deserialize)]
struct LastFmTrack {
    name: String,
    #[serde(rename = "match")]
    match_score: f64,
    artist: LastFmArtist,
}

#[derive(Deserialize)]
struct LastFmArtist {
    name: String,
}

// Token Response from Spotify
#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

// Combined app state
#[derive(Clone)]
struct AppState {
    rate_limiter: Arc<RateLimiter>,
    spotify_token_manager: Option<Arc<TokenManager>>,
}

// Rate limiter for MusicBrainz API (1 request per second)
struct RateLimiter {
    last_request: TokioMutex<Instant>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            last_request: TokioMutex::new(Instant::now() - Duration::from_secs(1)),
        }
    }

    async fn wait(&self) {
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();
        if elapsed < Duration::from_secs(1) {
            let wait_time = Duration::from_secs(1) - elapsed;
            tokio::time::sleep(wait_time).await;
        }
        *last = Instant::now();
    }
}

// Token Manager
struct TokenManager {
    token: TokioMutex<Option<(String, Instant)>>,
    client_id: String,
    client_secret: String,
}

impl TokenManager {
    fn new(client_id: String, client_secret: String) -> Self {
        Self {
            token: TokioMutex::new(None),
            client_id,
            client_secret,
        }
    }

    async fn get_token(&self) -> String {
        let mut guard = self.token.lock().await;
        if let Some((ref tok, ref time)) = *guard {
            if time.elapsed() < Duration::from_secs(3600 - 60) {
                return tok.clone();
            }
        }
        let client = reqwest::Client::new();
        let params = [("grant_type", "client_credentials")];
        let res = client
            .post("https://accounts.spotify.com/api/token")
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&params)
            .send()
            .await
            .unwrap()
            .json::<TokenResponse>()
            .await
            .unwrap();
        let new_token = res.access_token;
        *guard = Some((new_token.clone(), Instant::now()));
        new_token
    }
}

// Levenshtein distance
fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut matrix = vec![vec![0; b_chars.len() + 1]; a_chars.len() + 1];
    for i in 0..=a_chars.len() {
        matrix[i][0] = i;
    }
    for j in 0..=b_chars.len() {
        matrix[0][j] = j;
    }
    for i in 1..=a_chars.len() {
        for j in 1..=b_chars.len() {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = *[
                matrix[i - 1][j] + 1,
                matrix[i][j - 1] + 1,
                matrix[i - 1][j - 1] + cost,
            ]
            .iter()
            .min()
            .unwrap();
        }
    }
    matrix[a_chars.len()][b_chars.len()]
}

// Simple VADER-like sentiment analysis
fn vader_analyse(lyrics: &str) -> f64 {
    let words: Vec<&str> = lyrics.split_whitespace().collect();
    if words.is_empty() {
        return 0.0;
    }
    let lexicon: HashMap<&str, f64> = [
        ("love", 3.0),
        ("hate", -3.0),
        ("happy", 2.5),
        ("sad", -2.5),
        // Add more words as needed
    ]
    .iter()
    .cloned()
    .collect();
    let sum: f64 = words
        .iter()
        .map(|w| {
            lexicon
                .get(w.to_lowercase().as_str())
                .cloned()
                .unwrap_or(0.0)
        })
        .sum();
    sum / words.len() as f64
}

// Cosine similarity
fn cosine_similarity(a: &Vec<f64>, b: &Vec<f64>, weights: &HashMap<String, f64>) -> f64 {
    let dot: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| x * y * weights.get("default").cloned().unwrap_or(1.0))
        .sum();
    let mag_a = a.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    let mag_b = b.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

// Search MusicBrainz for recordings with better ranking
async fn search_musicbrainz(
    query: &str,
    rate_limiter: &RateLimiter,
) -> Result<Vec<MusicBrainzRecording>, Box<dyn std::error::Error + Send + Sync>> {
    rate_limiter.wait().await;

    let client = reqwest::Client::new();

    // Parse query to extract track and artist
    let enhanced_query = if query.contains(" by ") {
        // If "by" is present, use as-is
        let parts: Vec<&str> = query.split(" by ").collect();
        if parts.len() == 2 {
            format!("recording:\"{}\" AND artist:\"{}\"", parts[0], parts[1])
        } else {
            query.to_string()
        }
    } else {
        // Try to intelligently split track and artist
        // Common patterns: "Track Name Artist Name" or "Track Name - Artist Name"
        if query.contains(" - ") {
            let parts: Vec<&str> = query.split(" - ").collect();
            if parts.len() == 2 {
                format!("recording:\"{}\" AND artist:\"{}\"", parts[0].trim(), parts[1].trim())
            } else {
                query.to_string()
            }
        } else {
            // For queries like "All the Small Things Blink-182", try to identify where the artist starts
            // This is tricky, but we can make some educated guesses
            let words: Vec<&str> = query.split_whitespace().collect();
            if words.len() >= 3 {
                // Try different split points to find the best match
                format!("\"{}\"", query) // Search the whole phrase
            } else {
                query.to_string()
            }
        }
    };

    eprintln!("MusicBrainz search query: {}", enhanced_query);

    let url = format!(
        "https://musicbrainz.org/ws/2/recording?query={}&fmt=json&limit=50",
        urlencoding::encode(&enhanced_query)
    );

    let response = client
        .get(&url)
        .header(
            "User-Agent",
            "NextTrack/0.1.0 (https://github.com/leebenson/nexttrack)",
        )
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("MusicBrainz API error: {}", response.status()).into());
    }

    let search_result = response.json::<MusicBrainzSearchResponse>().await?;
    Ok(search_result.recordings)
}

// Fetch popularity data from ListenBrainz (no auth required)
async fn fetch_listenbrainz_popularity(
    mbids: Vec<String>,
) -> Result<HashMap<String, u32>, Box<dyn std::error::Error + Send + Sync>> {
    if mbids.is_empty() {
        return Ok(HashMap::new());
    }

    let client = reqwest::Client::new();
    let url = "https://api.listenbrainz.org/1/popularity/recording";

    let request = ListenBrainzRecordingRequest {
        recording_mbids: mbids,
    };

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("ListenBrainz API error: {}", response.status()).into());
    }

    let popularity_data = response.json::<ListenBrainzPopularityResponse>().await?;

    // Convert to a hashmap and calculate popularity score (0-100)
    let mut popularity_map = HashMap::new();

    // Find max listen count for normalization
    let max_count = popularity_data
        .payload
        .iter()
        .filter_map(|p| p.total_listen_count)
        .max()
        .unwrap_or(1);

    for recording in popularity_data.payload {
        if let Some(listen_count) = recording.total_listen_count {
            // Normalize to 0-100 scale, with logarithmic scaling for better distribution
            let popularity = if listen_count > 0 {
                let log_count = (listen_count as f64).ln();
                let log_max = (max_count as f64).ln();
                ((log_count / log_max) * 100.0).min(100.0) as u32
            } else {
                0
            };
            popularity_map.insert(recording.recording_mbid, popularity);
        } else {
            popularity_map.insert(recording.recording_mbid, 0);
        }
    }

    Ok(popularity_map)
}

// Fetch AcousticBrainz features for a recording
async fn fetch_acousticbrainz_features(
    mbid: &str,
) -> Result<AcousticBrainzResponse, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    // AcousticBrainz doesn't require rate limiting
    let url = format!("https://acousticbrainz.org/api/v1/{}/low-level", mbid);

    let response = client.get(&url).send().await?;

    if response.status() == 404 {
        return Err("No AcousticBrainz data for this recording".into());
    }

    if !response.status().is_success() {
        return Err(format!("AcousticBrainz API error: {}", response.status()).into());
    }

    let features = response.json::<AcousticBrainzResponse>().await?;
    Ok(features)
}

// Convert AcousticBrainz features to our internal format
fn convert_acousticbrainz_features(ab_features: &AcousticBrainzResponse) -> HashMap<String, f64> {
    let mut features = HashMap::new();

    if let Some(lowlevel) = &ab_features.lowlevel {
        features.insert("tempo".to_string(), lowlevel.bpm / 200.0); // Normalize to 0-1
        features.insert(
            "loudness".to_string(),
            (lowlevel.average_loudness + 60.0) / 60.0,
        ); // Normalize
        features.insert("complexity".to_string(), lowlevel.dynamic_complexity);
    }

    if let Some(highlevel) = &ab_features.highlevel {
        if let Some(danceable) = highlevel.danceability.all.danceable {
            features.insert("danceability".to_string(), danceable);
        }
        if let Some(happy) = highlevel.mood_happy.all.happy {
            features.insert("valence".to_string(), happy); // Similar to Spotify's valence
        }
        if let Some(aggressive) = highlevel.mood_aggressive.all.aggressive {
            features.insert("energy".to_string(), aggressive); // Similar to energy
        }
    }

    features
}

// Resolve tracks using MusicBrainz
async fn resolve_tracks_musicbrainz(
    queries: Vec<String>,
    rate_limiter: &RateLimiter,
) -> Result<Vec<TrackId>, Box<dyn std::error::Error + Send + Sync>> {
    let mut ids = Vec::new();
    let mut seen = HashSet::new();

    for query in queries {
        eprintln!("Searching for: {}", query);
        // Search MusicBrainz
        let recordings = search_musicbrainz(&query, rate_limiter).await?;

        if !recordings.is_empty() {
            // Try to find the best match
            let query_lower = query.to_lowercase();
            let words: Vec<&str> = query.split_whitespace().collect();
            
            // Score each recording based on how well it matches the query
            let mut scored_recordings: Vec<(&MusicBrainzRecording, i32)> = recordings
                .iter()
                .map(|rec| {
                    let title_lower = rec.title.to_lowercase();
                    let artist_name = rec
                        .artist_credit
                        .as_ref()
                        .and_then(|credits| credits.first())
                        .map(|credit| credit.artist.name.to_lowercase())
                        .unwrap_or_default();
                    
                    let mut score = 0;
                    
                    // HUGE bonus for exact artist match
                    for word in &words {
                        let word_lower = word.to_lowercase();
                        // Check if this might be the artist name
                        if artist_name == word_lower {
                            score += 100; // Exact artist match
                        } else if artist_name.contains(&word_lower) {
                            score += 10;
                        }
                        
                        if title_lower.contains(&word_lower) {
                            score += 5;
                        }
                    }
                    
                    // Check if the artist name in the result matches what we're searching for
                    // Split the query by "by" to extract expected artist
                    if query_lower.contains(" by ") {
                        if let Some(expected_artist) = query_lower.split(" by ").nth(1) {
                            let expected_artist = expected_artist.trim();
                            if artist_name == expected_artist {
                                score += 200; // Exact artist match
                            } else if artist_name.contains(expected_artist) || expected_artist.contains(&artist_name) {
                                score += 50; // Partial match
                            }
                        }
                    }
                    
                    // HEAVY penalties for covers
                    if title_lower.contains("cover") || title_lower.contains("tribute") || 
                       title_lower.contains("karaoke") || title_lower.contains(" vs ") ||
                       title_lower.contains("remix") || title_lower.contains("acoustic") ||
                       artist_name.contains("tribute") || artist_name.contains("karaoke") ||
                       artist_name.contains("twinkle") || artist_name.contains("rock star") ||
                       title_lower.contains("(") && title_lower.contains(")") {
                        score -= 100;
                    }
                    
                    // Bonus for clean titles (no parentheses)
                    if !title_lower.contains("(") && !title_lower.contains("[") {
                        score += 20;
                    }
                    
                    (rec, score)
                })
                .collect();
            
            // Sort by score descending
            scored_recordings.sort_by(|a, b| b.1.cmp(&a.1));
            
            // Debug: show top 5 results
            eprintln!("Top search results for '{}' =>", query);
            for (i, (rec, score)) in scored_recordings.iter().take(5).enumerate() {
                let artist_name = rec.artist_credit.as_ref()
                    .and_then(|credits| credits.first())
                    .map(|credit| credit.artist.name.as_str())
                    .unwrap_or("Unknown");
                eprintln!("  {}. {} by {} (score: {})", i+1, rec.title, artist_name, score);
            }
            
            let best_recording = scored_recordings.first()
                .map(|(rec, _)| *rec)
                .unwrap_or(&recordings[0]);

            let artist_name = best_recording
                .artist_credit
                .as_ref()
                .and_then(|credits| credits.first())
                .map(|credit| credit.artist.name.clone())
                .unwrap_or_else(|| "Unknown Artist".to_string());

            eprintln!(
                "Selected recording: {} by {} (from {} results)",
                best_recording.title, artist_name, recordings.len()
            );

            let track_id = TrackId {
                mbid: Some(best_recording.id.clone()),
                spotify: None,
                name: best_recording.title.clone(),
                artist: artist_name,
            };

            let id_string = best_recording.id.clone();
            if seen.insert(id_string) {
                ids.push(track_id);
            }
        } else {
            eprintln!("No recordings found for: {}", query);
        }
    }

    Ok(ids)
}

// Resolve tracks (legacy Spotify version - kept for migration)
async fn resolve_tracks(
    queries: Vec<String>,
    token: &str,
) -> Result<Vec<TrackId>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let mut ids = Vec::new();
    let mut seen = HashSet::new();
    for query in queries {
        // Check if query looks like a Spotify track ID (22 characters, alphanumeric)
        if query.len() == 22 && query.chars().all(|c| c.is_alphanumeric()) {
            // Verify it's a valid track ID by fetching track info
            let track_url = format!("https://api.spotify.com/v1/tracks/{}", query);
            let track_response = client
                .get(&track_url)
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .send()
                .await?;

            if track_response.status().is_success() {
                if seen.insert(query.clone()) {
                    // For Spotify IDs, we need to fetch track info to get name/artist
                    let track_info = track_response.json::<SpotifyTrack>().await?;
                    ids.push(TrackId {
                        spotify: Some(query),
                        mbid: None,
                        name: track_info.name,
                        artist: track_info
                            .artists
                            .first()
                            .map(|a| a.name.clone())
                            .unwrap_or_else(|| "Unknown Artist".to_string()),
                    });
                }
                continue;
            }
        }

        // Otherwise, search for the track
        let url = format!(
            "https://api.spotify.com/v1/search?q={}&type=track&limit=10",
            urlencoding::encode(&query)
        );
        let res = client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?
            .json::<SpotifySearchResponse>()
            .await?;
        if res.tracks.items.is_empty() {
            continue;
        }
        let lowercase_query = query.to_lowercase();
        let best_match = res
            .tracks
            .items
            .into_iter()
            .min_by_key(|t| levenshtein(&t.name.to_lowercase(), &lowercase_query))
            .unwrap();
        if seen.insert(best_match.id.clone()) {
            ids.push(TrackId {
                spotify: Some(best_match.id),
                mbid: None,
                name: best_match.name.clone(),
                artist: best_match
                    .artists
                    .first()
                    .map(|a| a.name.clone())
                    .unwrap_or_else(|| "Unknown Artist".to_string()),
            });
        }
    }
    Ok(ids)
}

// Fetch Spotify features
async fn fetch_spotify_features(
    track_id: &str,
    token: &str,
) -> Result<SpotifyFeatures, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let url = format!("https://api.spotify.com/v1/audio-features/{}", track_id);
    let res = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?
        .json::<SpotifyFeatures>()
        .await?;
    Ok(res)
}

// Fetch Genius lyrics
async fn fetch_genius_lyrics(
    query: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let genius_key = env::var("GENIUS_API_KEY").expect("GENIUS_API_KEY not set");
    let client = reqwest::Client::new();
    // First search
    let search_url = format!(
        "https://api.genius.com/search?q={}",
        urlencoding::encode(query)
    );
    let res = client
        .get(&search_url)
        .header(AUTHORIZATION, format!("Bearer {}", genius_key))
        .send()
        .await?
        .json::<GeniusSearchResponse>()
        .await?;
    if res.response.hits.is_empty() {
        return Ok("".to_string());
    }
    let path = res.response.hits[0].result.path.clone();
    let lyrics_url = format!("https://genius.com{}", path);
    let text = client.get(&lyrics_url).send().await?.text().await?;
    let document = Html::parse_document(&text);
    let selector = Selector::parse(r#"div[data-lyrics-container="true"]"#).unwrap();
    let lyrics = document
        .select(&selector)
        .flat_map(|el| el.text())
        .collect::<String>();
    Ok(lyrics)
}

// Fetch album art from MusicBrainz Cover Art Archive
async fn fetch_album_art(mbid: &str, rate_limiter: &RateLimiter) -> Option<String> {
    // First get recording details to find a release
    rate_limiter.wait().await;
    let client = reqwest::Client::new();
    
    let recording_url = format!(
        "https://musicbrainz.org/ws/2/recording/{}?inc=releases&fmt=json",
        mbid
    );
    
    let recording_response = client
        .get(&recording_url)
        .header("User-Agent", "NextTrack/1.0")
        .send()
        .await;
        
    if let Ok(resp) = recording_response {
        if let Ok(json) = resp.json::<serde_json::Value>().await {
            // Get the first release ID
            if let Some(releases) = json.get("releases").and_then(|r| r.as_array()) {
                if let Some(first_release) = releases.first() {
                    if let Some(release_id) = first_release.get("id").and_then(|id| id.as_str()) {
                        // Try to fetch cover art for this release
                        let cover_url = format!(
                            "https://coverartarchive.org/release/{}",
                            release_id
                        );
                        
                        if let Ok(cover_resp) = client.get(&cover_url).send().await {
                            if let Ok(cover_data) = cover_resp.json::<CoverArtArchiveResponse>().await {
                                // Return the large thumbnail if available, otherwise the full image
                                if let Some(first_image) = cover_data.images.first() {
                                    return first_image.thumbnails.large.clone()
                                        .or_else(|| first_image.thumbnails.small.clone())
                                        .or_else(|| Some(first_image.image.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}

// Aggregate features using MusicBrainz/AcousticBrainz
async fn aggregate_features_musicbrainz(
    track: &TrackId,
    _rate_limiter: &RateLimiter,
) -> Result<Track, Box<dyn std::error::Error + Send + Sync>> {
    let mut features = HashMap::new();

    // Try to get AcousticBrainz features if we have an MBID
    if let Some(mbid) = &track.mbid {
        match fetch_acousticbrainz_features(mbid).await {
            Ok(ab_features) => {
                features = convert_acousticbrainz_features(&ab_features);
            }
            Err(_) => {
                // No AcousticBrainz data available - use empty features
                // This is real - many tracks don't have audio analysis
            }
        }
    }

    // Get lyrics sentiment from Genius
    let query = format!("{} {}", track.name, track.artist);
    let sentiment = match fetch_genius_lyrics(&query).await {
        Ok(lyrics) => vader_analyse(&lyrics),
        Err(_) => 0.5, // Neutral sentiment as fallback
    };
    features.insert("sentiment".to_string(), sentiment);

    // Fetch real popularity from ListenBrainz if we have an MBID
    let popularity = if let Some(mbid) = &track.mbid {
        match fetch_listenbrainz_popularity(vec![mbid.clone()]).await {
            Ok(map) => map.get(mbid).copied().unwrap_or(0),
            Err(_) => 0,
        }
    } else {
        0
    };

    // Fetch album art if we have an MBID
    let album_art = if let Some(mbid) = &track.mbid {
        fetch_album_art(mbid, _rate_limiter).await
    } else {
        None
    };

    Ok(Track {
        id: track
            .mbid
            .clone()
            .unwrap_or_else(|| format!("{}-{}", track.name, track.artist)),
        name: track.name.clone(),
        artist: track.artist.clone(),
        features,
        popularity,
        album_art,
    })
}

// Aggregate features (legacy Spotify version)
async fn aggregate_features(
    track_id: &str,
    token: &str,
) -> Result<Track, Box<dyn std::error::Error + Send + Sync>> {
    let spotify_features = fetch_spotify_features(track_id, token).await?;
    // Fetch track info to get name and artist for Genius query
    let client = reqwest::Client::new();
    let track_url = format!("https://api.spotify.com/v1/tracks/{}", track_id);
    let track_res = client
        .get(&track_url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?
        .json::<SpotifyTrack>()
        .await?;
    let query = format!("{} {}", track_res.name, track_res.artists[0].name);
    let lyrics = fetch_genius_lyrics(&query).await?;
    let sentiment = vader_analyse(&lyrics);
    let mut features = HashMap::new();
    features.insert("energy".to_string(), spotify_features.energy);
    features.insert("valence".to_string(), spotify_features.valence);
    features.insert("tempo".to_string(), spotify_features.tempo);
    features.insert("sentiment".to_string(), sentiment);
    Ok(Track {
        id: track_id.to_string(),
        name: track_res.name,
        artist: track_res.artists[0].name.clone(),
        features,
        popularity: track_res.popularity,
        album_art: None, // Spotify version doesn't support album art yet
    })
}

// Generate candidates using Last.fm
async fn generate_candidates(
    seeds: Vec<TrackId>,
    inputs: &Vec<Track>,
    prefs: &Preferences,
    token: &str,
) -> Result<Vec<Track>, Box<dyn std::error::Error + Send + Sync>> {
    if seeds.is_empty() {
        return Err("No seed tracks provided".into());
    }

    let lastfm_key = env::var("LASTFM_API_KEY").expect("LASTFM_API_KEY not set");
    let client = reqwest::Client::new();
    let mut candidate_queries = Vec::new();
    for input in inputs {
        let url = format!(
            "https://ws.audioscrobbler.com/2.0/?method=track.getsimilar&track={}&artist={}&api_key={}&format=json&limit=50",
            urlencoding::encode(&input.name),
            urlencoding::encode(&input.artist),
            lastfm_key
        );
        let res = client
            .get(&url)
            .send()
            .await?
            .json::<LastFmSimilar>()
            .await?;
        for sim_track in res.similartracks.track {
            if sim_track.match_score > 0.1 {
                // threshold to filter low matches
                let query = format!("{} by {}", sim_track.name, sim_track.artist.name);
                candidate_queries.push(query);
            }
        }
    }

    let candidate_ids = resolve_tracks(candidate_queries, token).await?;

    let seed_ids: HashSet<String> = seeds.iter().filter_map(|s| s.spotify.clone()).collect();

    let mut candidates = Vec::new();
    for id in candidate_ids {
        if let Some(spotify_id) = &id.spotify {
            if seed_ids.contains(spotify_id) {
                continue;
            }
        }
        if let Some(spotify_id) = &id.spotify {
            if let Ok(track) = aggregate_features(spotify_id, token).await {
                let obscurity_score = 1.0 - (track.popularity as f64 / 100.0);
                if obscurity_score >= prefs.obscurity {
                    candidates.push(track);
                }
            }
        }
    }
    // Shuffle and truncate
    candidates.shuffle(&mut thread_rng());
    candidates.truncate(50);
    Ok(candidates)
}

// Scoring trait
trait ScoringFunction {
    fn score(&self, inputs: &Vec<Track>, candidate: &Track) -> f64;
}

struct AudioSimilarityScorer {
    weights: HashMap<String, f64>,
}

impl ScoringFunction for AudioSimilarityScorer {
    fn score(&self, inputs: &Vec<Track>, candidate: &Track) -> f64 {
        // Average input features
        let mut avg = HashMap::new();
        for key in inputs[0].features.keys() {
            let sum: f64 = inputs
                .iter()
                .map(|t| *t.features.get(key).unwrap_or(&0.0))
                .sum();
            avg.insert(key.clone(), sum / inputs.len() as f64);
        }
        let a_vec: Vec<f64> = avg.values().cloned().collect();
        let b_vec: Vec<f64> = candidate.features.values().cloned().collect();
        cosine_similarity(&a_vec, &b_vec, &self.weights)
    }
}

// Other scorers
struct ObscurityScorer;

impl ScoringFunction for ObscurityScorer {
    fn score(&self, _inputs: &Vec<Track>, candidate: &Track) -> f64 {
        1.0 - (candidate.popularity as f64 / 100.0)
    }
}

// SSE event types for streaming
#[derive(Serialize)]
#[serde(tag = "type")]
enum RecommendationEvent {
    Status { message: String },
    Candidate { track: Track, score: f64 },
    Complete { tracks: Vec<Track> },
    Error { message: String },
    Debug { message: String, data: Option<serde_json::Value> },
}

// Streaming MusicBrainz recommend handler using channels
async fn recommend_musicbrainz_stream_handler(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<RecommendRequest>,
) -> impl IntoResponse {
    use tokio::sync::mpsc;

    let (tx, rx) = mpsc::channel::<Result<Event, axum::Error>>(10);

    // Spawn the recommendation task
    tokio::spawn(async move {
        let result = process_recommendations(app_state, req, tx.clone()).await;
        if let Err(e) = result {
            let _ = tx
                .send(Ok(Event::default()
                    .json_data(RecommendationEvent::Error {
                        message: format!("Processing error: {}", e),
                    })
                    .unwrap()))
                .await;
        }
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    Sse::new(stream)
}

// Helper function to process recommendations and send events
async fn process_recommendations(
    app_state: Arc<AppState>,
    req: RecommendRequest,
    tx: tokio::sync::mpsc::Sender<Result<Event, axum::Error>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Send initial status
    tx.send(Ok(Event::default().json_data(
        RecommendationEvent::Status {
            message: "Starting recommendation process...".to_string(),
        },
    )?))
    .await?;

    // Resolve input tracks
    tx.send(Ok(Event::default().json_data(
        RecommendationEvent::Status {
            message: "Searching for input tracks...".to_string(),
        },
    )?))
    .await?;

    let seeds = resolve_tracks_musicbrainz(req.tracks, &app_state.rate_limiter).await?;

    tx.send(Ok(Event::default().json_data(
        RecommendationEvent::Status {
            message: format!("Found {} input tracks", seeds.len()),
        },
    )?))
    .await?;

    // Process seeds
    let mut inputs = Vec::new();
    for (i, seed) in seeds.iter().enumerate() {
        tx.send(Ok(Event::default().json_data(
            RecommendationEvent::Status {
                message: format!("Processing track {}/{}: {}", i + 1, seeds.len(), seed.name),
            },
        )?))
        .await?;
        
        // Send debug info about selected track
        tx.send(Ok(Event::default().json_data(
            RecommendationEvent::Debug {
                message: format!("Selected: {} by {}", seed.name, seed.artist),
                data: None,
            },
        )?))
        .await?;

        if let Ok(track) = aggregate_features_musicbrainz(&seed, &app_state.rate_limiter).await {
            inputs.push(track);
        }
    }

    if inputs.is_empty() {
        tx.send(Ok(Event::default().json_data(
            RecommendationEvent::Error {
                message: "No valid input tracks found".to_string(),
            },
        )?))
        .await?;
        return Ok(());
    }

    // Get similar tracks from Last.fm
    tx.send(Ok(Event::default().json_data(
        RecommendationEvent::Status {
            message: "Finding similar tracks...".to_string(),
        },
    )?))
    .await?;

    let lastfm_key = env::var("LASTFM_API_KEY")?;
    let client = reqwest::Client::new();
    let mut candidate_queries = Vec::new();

    for input in &inputs {
        let url = format!(
            "https://ws.audioscrobbler.com/2.0/?method=track.getsimilar&track={}&artist={}&api_key={}&format=json&limit=20",
            urlencoding::encode(&input.name),
            urlencoding::encode(&input.artist),
            lastfm_key
        );

        match client.get(&url).send().await {
            Ok(response) => {
                let status = response.status();
                eprintln!("Last.fm response status: {}", status);

                if status.is_success() {
                    if let Ok(res) = response.json::<LastFmSimilar>().await {
                        for sim_track in res.similartracks.track {
                            if sim_track.match_score > 0.1 {
                                let query = format!("{} by {}", sim_track.name, sim_track.artist.name);
                                candidate_queries.push(query);
                            }
                        }
                    }
                } else {
                    eprintln!("Last.fm API error: {}", status);
                    // Don't use fallback data - just continue with fewer results
                }
            }
            Err(e) => {
                eprintln!("Last.fm request error: {}", e);
                continue;
            }
        }
    }

    tx.send(Ok(Event::default().json_data(
        RecommendationEvent::Status {
            message: format!(
                "Found {} similar tracks to process",
                candidate_queries.len()
            ),
        },
    )?))
    .await?;

    // If no candidates found, return error
    if candidate_queries.is_empty() {
        tx.send(Ok(Event::default().json_data(
            RecommendationEvent::Error {
                message: "Could not find similar tracks. Please ensure you have a valid Last.fm API key.".to_string(),
            },
        )?))
        .await?;
        return Ok(());
    }

    // Process candidates in batches
    let seed_mbids: HashSet<String> = seeds.iter().filter_map(|s| s.mbid.clone()).collect();

    let mut all_candidates = Vec::new();
    let mut not_found_count = 0;
    let batch_size = 10; // Increased for faster processing

    for (batch_num, chunk) in candidate_queries.chunks(batch_size).enumerate() {
        tx.send(Ok(Event::default().json_data(
            RecommendationEvent::Status {
                message: format!(
                    "Processing batch {}/{}",
                    batch_num + 1,
                    (candidate_queries.len() + batch_size - 1) / batch_size
                ),
            },
        )?))
        .await?;

        let batch_ids =
            match resolve_tracks_musicbrainz(chunk.to_vec(), &app_state.rate_limiter).await {
                Ok(ids) => ids,
                Err(_) => continue,
            };

        for id in batch_ids {
            if let Some(mbid) = &id.mbid {
                if seed_mbids.contains(mbid) {
                    continue;
                }
            }

            if let Ok(track) = aggregate_features_musicbrainz(&id, &app_state.rate_limiter).await {
                // Calculate score based on preferences
                let audio_scorer = AudioSimilarityScorer {
                    weights: hashmap! {"default".to_string() => 1.0},
                };
                let obscurity_scorer = ObscurityScorer;

                // Adjust scoring weights based on obscurity preference
                // Low obscurity (0.0) = prefer popular tracks
                // High obscurity (1.0) = prefer obscure tracks
                let obscurity_weight = req.preferences.obscurity;
                let similarity_weight = 1.0 - obscurity_weight;
                
                let score = similarity_weight * audio_scorer.score(&inputs, &track)
                    + obscurity_weight * obscurity_scorer.score(&inputs, &track);

                // Send candidate immediately
                tx.send(Ok(Event::default().json_data(
                    RecommendationEvent::Candidate {
                        track: track.clone(),
                        score,
                    },
                )?))
                .await?;

                all_candidates.push((track, score));
            } else {
                not_found_count += 1;
            }
        }
    }

    // Send summary debug info
    tx.send(Ok(Event::default().json_data(
        RecommendationEvent::Debug {
            message: format!(
                "Summary: {} candidates searched, {} tracks found, {} not found in MusicBrainz",
                candidate_queries.len(), all_candidates.len(), not_found_count
            ),
            data: None,
        },
    )?))
    .await?;

    // Sort and send top results
    all_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let top_tracks: Vec<Track> = all_candidates
        .into_iter()
        .take(20) // Show more results since we're not filtering
        .map(|(track, _)| track)
        .collect();

    tx.send(Ok(Event::default().json_data(
        RecommendationEvent::Complete { tracks: top_tracks },
    )?))
    .await?;

    Ok(())
}

// Original MusicBrainz recommend handler (kept for compatibility)
async fn recommend_musicbrainz_handler(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<RecommendRequest>,
) -> impl IntoResponse {
    eprintln!("Recommendation request: {:?}", req);

    // Resolve input tracks using MusicBrainz
    let seeds = match resolve_tracks_musicbrainz(req.tracks, &app_state.rate_limiter).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error resolving tracks: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error resolving tracks: {}", e),
            )
                .into_response();
        }
    };

    eprintln!("Resolved {} seeds", seeds.len());

    let mut inputs = Vec::new();
    for seed in &seeds {
        eprintln!("Processing seed: {:?}", seed);
        if let Ok(track) = aggregate_features_musicbrainz(&seed, &app_state.rate_limiter).await {
            eprintln!("Added input track: {} by {}", track.name, track.artist);
            inputs.push(track);
        }
    }

    if inputs.is_empty() {
        return (StatusCode::BAD_REQUEST, "No valid input tracks".to_string()).into_response();
    }

    // Generate candidates using Last.fm (doesn't require Spotify token)
    let lastfm_key = match env::var("LASTFM_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "LASTFM_API_KEY not set".to_string(),
            )
                .into_response()
        }
    };

    let client = reqwest::Client::new();
    let mut candidate_queries = Vec::new();

    for input in &inputs {
        let url = format!(
            "https://ws.audioscrobbler.com/2.0/?method=track.getsimilar&track={}&artist={}&api_key={}&format=json&limit=50",
            urlencoding::encode(&input.name),
            urlencoding::encode(&input.artist),
            lastfm_key
        );

        eprintln!(
            "Fetching similar tracks from Last.fm for: {} by {}",
            input.name, input.artist
        );

        match client.get(&url).send().await {
            Ok(response) => {
                let status = response.status();
                eprintln!("Last.fm response status: {}", status);

                if let Ok(res) = response.json::<LastFmSimilar>().await {
                    eprintln!("Found {} similar tracks", res.similartracks.track.len());
                    for sim_track in res.similartracks.track {
                        if sim_track.match_score > 0.1 {
                            let query = format!("{} {}", sim_track.name, sim_track.artist.name);
                            candidate_queries.push(query);
                        }
                    }
                } else {
                    eprintln!("Failed to parse Last.fm response");
                }
            }
            Err(e) => {
                eprintln!("Last.fm request failed: {}", e);
                continue;
            }
        }
    }

    eprintln!("Found {} candidate queries", candidate_queries.len());

    // Resolve candidates using MusicBrainz
    let candidate_ids =
        match resolve_tracks_musicbrainz(candidate_queries, &app_state.rate_limiter).await {
            Ok(ids) => ids,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error resolving candidates: {}", e),
                )
                    .into_response()
            }
        };

    eprintln!("Resolved {} candidate tracks", candidate_ids.len());

    // Filter out seeds
    let seed_mbids: HashSet<String> = seeds.iter().filter_map(|s| s.mbid.clone()).collect();

    let mut candidates = Vec::new();
    for id in candidate_ids {
        if let Some(mbid) = &id.mbid {
            if seed_mbids.contains(mbid) {
                eprintln!("Skipping seed track: {}", mbid);
                continue;
            }
        }

        if let Ok(track) = aggregate_features_musicbrainz(&id, &app_state.rate_limiter).await {
            // Don't filter - include all tracks
            candidates.push(track);
        }
    }

    eprintln!("Found {} candidates after filtering", candidates.len());

    // Score candidates
    let audio_scorer = AudioSimilarityScorer {
        weights: hashmap! {"default".to_string() => 1.0},
    };
    let obscurity_scorer = ObscurityScorer;

    // Adjust scoring weights based on obscurity preference
    let obscurity_weight = req.preferences.obscurity;
    let similarity_weight = 1.0 - obscurity_weight;

    let mut scored: Vec<(Track, f64)> = candidates
        .into_iter()
        .map(|cand| {
            let score = similarity_weight * audio_scorer.score(&inputs, &cand)
                + obscurity_weight * obscurity_scorer.score(&inputs, &cand);
            (cand, score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let top = scored
        .into_iter()
        .take(20) // Show more results
        .map(|(t, _)| t)
        .collect::<Vec<_>>();

    (StatusCode::OK, Json(top)).into_response()
}

// Legacy Spotify recommend handler
async fn recommend_handler(
    State(app_state): State<Arc<AppState>>,
    Json(req): Json<RecommendRequest>,
) -> impl IntoResponse {
    let token_manager = app_state
        .spotify_token_manager
        .as_ref()
        .expect("Spotify token manager not initialized");
    let token = token_manager.get_token().await;
    let seeds = match resolve_tracks(req.tracks, &token).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error resolving tracks: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error resolving tracks: {}", e),
            )
                .into_response();
        }
    };
    let mut inputs = Vec::new();
    for seed in &seeds {
        if let Some(spotify_id) = &seed.spotify {
            if let Ok(track) = aggregate_features(spotify_id, &token).await {
                inputs.push(track);
            }
        }
    }
    if inputs.is_empty() {
        return (StatusCode::BAD_REQUEST, "No valid input tracks".to_string()).into_response();
    }
    let candidates =
        match generate_candidates(seeds.clone(), &inputs, &req.preferences, &token).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error generating candidates: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error generating candidates: {}", e),
                )
                    .into_response();
            }
        };
    // Fetch features for candidates if needed for scoring
    let mut candidates_with_features = Vec::new();
    for cand in candidates {
        candidates_with_features.push(cand);
    }
    // Score
    let audio_scorer = AudioSimilarityScorer {
        weights: hashmap! {"default".to_string() => 1.0},
    };
    let obscurity_scorer = ObscurityScorer;
    let mut scored: Vec<(Track, f64)> = candidates_with_features
        .into_iter()
        .map(|cand| {
            let score = 0.6 * audio_scorer.score(&inputs, &cand)
                + 0.4 * obscurity_scorer.score(&inputs, &cand);
            (cand, score)
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let top = scored
        .into_iter()
        .take(20) // Show more results
        .map(|(t, _)| t)
        .collect::<Vec<_>>();
    (StatusCode::OK, Json(top)).into_response()
}

// MusicBrainz search handler
async fn search_musicbrainz_handler(
    State(app_state): State<Arc<AppState>>,
    Path(query): Path<String>,
) -> impl IntoResponse {
    let recordings = match search_musicbrainz(&query, &app_state.rate_limiter).await {
        Ok(recs) => recs,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Search error: {}", e),
            )
                .into_response()
        }
    };

    // Collect all MBIDs for popularity lookup
    let mbids: Vec<String> = recordings.iter().map(|r| r.id.clone()).collect();

    // Fetch real popularity data from ListenBrainz
    let popularity_map = match fetch_listenbrainz_popularity(mbids).await {
        Ok(map) => map,
        Err(e) => {
            eprintln!("Failed to fetch popularity: {}", e);
            HashMap::new()
        }
    };

    // Group by artist and title to remove duplicates
    let mut seen_tracks = HashSet::new();
    let mut tracks: Vec<Track> = Vec::new();

    for rec in recordings {
        let artist_name = rec
            .artist_credit
            .as_ref()
            .and_then(|credits| credits.first())
            .map(|credit| credit.artist.name.clone())
            .unwrap_or_else(|| "Unknown Artist".to_string());

        let track_key = format!(
            "{} - {}",
            artist_name.to_lowercase(),
            rec.title.to_lowercase()
        );

        if !seen_tracks.contains(&track_key) && tracks.len() < 10 {
            seen_tracks.insert(track_key);

            // Get real popularity or default to 0
            let popularity = popularity_map.get(&rec.id).copied().unwrap_or(0);

            tracks.push(Track {
                id: rec.id,
                name: rec.title,
                artist: artist_name,
                features: HashMap::new(),
                popularity,
                album_art: None, // Could fetch art here if needed
            });
        }
    }

    // Sort by popularity (descending)
    tracks.sort_by(|a, b| b.popularity.cmp(&a.popularity));

    // Add metadata about features availability
    let response = serde_json::json!({
        "tracks": tracks,
        "metadata": {
            "source": "MusicBrainz + ListenBrainz",
            "audio_features_available": false,
            "popularity_data": "Real listen counts from ListenBrainz",
            "note": "Use track IDs with /mb/recommend for audio features via AcousticBrainz"
        }
    });

    Json(response).into_response()
}

// Legacy Spotify search handler
async fn search_handler(
    State(app_state): State<Arc<AppState>>,
    Path(query): Path<String>,
) -> impl IntoResponse {
    let token_manager = app_state
        .spotify_token_manager
        .as_ref()
        .expect("Spotify token manager not initialized");
    let token = token_manager.get_token().await;
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.spotify.com/v1/search?q={}&type=track&limit=10",
        urlencoding::encode(&query)
    );
    let res = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .unwrap()
        .json::<SpotifySearchResponse>()
        .await
        .unwrap();
    (StatusCode::OK, Json(res)).into_response()
}

// Main
#[tokio::main]
async fn main() {
    // Check for required environment variables
    let use_spotify =
        env::var("SPOTIFY_CLIENT_ID").is_ok() && env::var("SPOTIFY_CLIENT_SECRET").is_ok();
    let _genius_key = env::var("GENIUS_API_KEY").ok();
    let _lastfm_key =
        env::var("LASTFM_API_KEY").expect("LASTFM_API_KEY required for recommendations");

    println!("Starting NextTrack API...");
    println!("MusicBrainz endpoints:");
    println!("  - GET  /mb/search/:query");
    println!("  - POST /mb/recommend");
    println!("  - POST /mb/recommend/stream (Server-Sent Events)");

    // Create app state
    let spotify_token_manager = if use_spotify {
        let client_id = env::var("SPOTIFY_CLIENT_ID").unwrap();
        let client_secret = env::var("SPOTIFY_CLIENT_SECRET").unwrap();
        println!(
            "Spotify credentials found - enabling legacy endpoints: /search/:query, /recommend"
        );
        Some(Arc::new(TokenManager::new(client_id, client_secret)))
    } else {
        println!("No Spotify credentials - only MusicBrainz endpoints available");
        None
    };

    let app_state = Arc::new(AppState {
        rate_limiter: Arc::new(RateLimiter::new()),
        spotify_token_manager,
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        // MusicBrainz routes (always available)
        .route("/mb/search/{query}", get(search_musicbrainz_handler))
        .route("/mb/recommend", post(recommend_musicbrainz_handler))
        .route(
            "/mb/recommend/stream",
            post(recommend_musicbrainz_stream_handler),
        )
        // Legacy Spotify routes (if credentials available)
        .route("/search/{query}", get(search_handler))
        .route("/recommend", post(recommend_handler))
        .layer(cors)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
