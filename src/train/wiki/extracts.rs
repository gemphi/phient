#[allow(unused_imports)]
use super::*;

impl WikiBulk {
/// Fetches a Wikipedia article extract via the REST API.
async fn fetch_wiki_extract(client: &reqwest::Client, topic: &str) -> Result<(String, String), String> {
    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&prop=extracts&exintro=false&explaintext=true&titles={}&format=json&redirects=1",
        topic.replace(' ', "_")
    );

    let resp = client.get(&url).send().await
        .map_err(|e| format!("request failed: {}", e))?;

    if resp.status().as_u16() == 429 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let resp2 = client.get(&url).send().await
            .map_err(|e| format!("retry failed: {}", e))?;
        if !resp2.status().is_success() {
            return Err(format!("status: {}", resp2.status()));
        }
        return Self::parse_wiki_response(resp2.text().await.map_err(|e| e.to_string())?, topic);
    }

    if !resp.status().is_success() {
        return Err(format!("status: {}", resp.status()));
    }

    Self::parse_wiki_response(resp.text().await.map_err(|e| e.to_string())?, topic)
}

fn parse_wiki_response(text: String, topic: &str) -> Result<(String, String), String> {
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("json parse: {}", e))?;

    let pages = json.get("query")
        .and_then(|q| q.get("pages"))
        .and_then(|p| p.as_object())
        .ok_or("no pages in response")?;

    for (_, page) in pages {
        let title = page.get("title")
            .and_then(|t| t.as_str())
            .unwrap_or(topic);
        if let Some(extract) = page.get("extract").and_then(|e| e.as_str()) {
            if !extract.is_empty() {
                return Ok((title.to_string(), extract.to_string()));
            }
        }
    }

    Err("no extract found".to_string())
}

/// Fetches Wikipedia extracts without holding a facet lock.
pub async fn fetch_curriculum_extracts(max_topics: Option<usize>) -> (Vec<(String, String)>, Vec<String>) {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("PhianoBot/0.1 (educational research)")
        .build()
        .expect("failed to build HTTP client");

    let mut all_topics: Vec<&str> = Vec::new();
    for group in CURRICULUM_TOPICS {
        for topic in *group {
            all_topics.push(topic);
        }
    }
    if let Some(max) = max_topics {
        all_topics.truncate(max);
    }

    let total = all_topics.len();
    let mut extracts = Vec::new();
    let mut errors = Vec::new();
    for (i, topic) in all_topics.iter().enumerate() {
        print!("  [ingest {}/{}] {}... ", i + 1, total, topic);
        match Self::fetch_wiki_extract(&client, topic).await {
            Ok((title, extract)) => {
                println!("OK");
                extracts.push((title, extract));
            }
            Err(e) => {
                println!("FAIL: {}", e);
                errors.push(format!("{}: {}", topic, e));
            }
        }
        if (i + 1) % 10 == 0 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }
    (extracts, errors)
}

/// Trains a facet on already-fetched Wikipedia extracts (sync, lock-safe).
pub fn train_extracts(facet: &mut Facet, trainer: &Trainer, extracts: &[(String, String)]) -> usize {
    let mut total_tokens = 0;
    for (_title, extract) in extracts {
        let truncated = if extract.len() > 5000 { &extract[..5000] } else { extract.as_str() };
        for sentence in Tokenizer::split_sentences(truncated) {
            if Tokenizer::tokenize(&sentence).len() >= 4 {
                total_tokens += trainer.train_sentence(facet, &sentence);
            }
        }
    }
    total_tokens
}

/// Runs bulk Wikipedia ingestion: downloads articles for all curriculum
/// topics and trains the facet on them.
pub async fn bulk_ingest(
    facet: &mut Facet,
    trainer: &Trainer,
    _chunk_store: &ChunkStore,
    max_topics: Option<usize>,
) -> IngestionResult {
    let vocab_before = facet.vocabulary_size();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("PhianoBot/0.1 (educational research)")
        .build()
        .expect("failed to build HTTP client");

    let mut all_topics: Vec<&str> = Vec::new();
    for group in CURRICULUM_TOPICS {
        for topic in *group {
            all_topics.push(topic);
        }
    }

    if let Some(max) = max_topics {
        all_topics.truncate(max);
    }

    let total = all_topics.len();
    let mut succeeded = 0usize;
    let mut total_tokens = 0usize;
    let mut errors = Vec::new();

    for (i, topic) in all_topics.iter().enumerate() {
        print!("  [ingest {}/{}] {}... ", i + 1, total, topic);

        match Self::fetch_wiki_extract(&client, topic).await {
            Ok((_title, extract)) => {
                let truncated = if extract.len() > 5000 {
                    &extract[..5000]
                } else {
                    &extract
                };
                let mut tokens = 0;
                for sentence in Tokenizer::split_sentences(truncated) {
                    if Tokenizer::tokenize(&sentence).len() >= 4 {
                        tokens += trainer.train_sentence(facet, &sentence);
                    }
                }
                total_tokens += tokens;
                succeeded += 1;
                println!("OK ({} tokens)", tokens);
            }
            Err(e) => {
                println!("FAIL: {}", e);
                errors.push(format!("{}: {}", topic, e));
            }
        }

        // Small delay to be respectful to Wikipedia API
        if (i + 1) % 10 == 0 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }

    IngestionResult {
        topics_attempted: total,
        topics_succeeded: succeeded,
        total_tokens_trained: total_tokens,
        vocabulary_before: vocab_before,
        vocabulary_after: facet.vocabulary_size(),
        errors,
    }
}
}
