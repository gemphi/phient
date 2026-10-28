#[allow(unused_imports)]
use super::*;

impl ApiSource {

    /// Fetch list of plain definition strings for training.
    pub fn fetch_word(&self, word: &str) -> Vec<String> {
        let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", word);

        let client = match reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Phiano/1.0")
            .build()
        {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let response = match client.get(&url).send() {
            Ok(r) => r,
            Err(_) => return vec![],
        };

        if !response.status().is_success() {
            return vec![];
        }

        let api_entries: Vec<ApiEntry> = match response.json() {
            Ok(e) => e,
            Err(_) => return vec![],
        };

        let mut definitions = Vec::new();
        for entry in api_entries {
            for meaning in entry.meanings {
                for def in meaning.definitions {
                    definitions.push(def.definition);
                }
            }
        }

        if !definitions.is_empty() {
            self.cache_word(word, &definitions);
        }

        definitions
    }

    /// Append a word's definitions to the local cache file.
    pub(super) fn cache_word(&self, word: &str, definitions: &[String]) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.cache_path)
        {
            for def in definitions {
                let _ = writeln!(file, "{}: {}", word, def);
            }
        }
    }
}

impl DictionarySource for ApiSource {
    fn fetch_all(&self) -> Vec<(String, String)> {
        let local = LocalSource::new(&self.cache_path);
        local.fetch_all()
    }

    fn fetch_definitions(&self, word: &str) -> Vec<String> {
        let local = LocalSource::new(&self.cache_path);
        let cached = local.fetch_definitions(word);
        if !cached.is_empty() {
            return cached;
        }

        self.fetch_word(word)
    }
}
