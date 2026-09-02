use reqwest::Client;
use serde_json::Value;

pub async fn search_stackoverflow(log: &str) -> Option<String> {
    // Cari baris terakhir yang tidak kosong sebagai kueri (biasanya berisi inti error)
    let query = log.lines().rev().find(|l| !l.trim().is_empty())?.trim();
    
    // Batasi kueri maksimal 100 karakter agar API tidak menolak
    let query: String = query.chars().take(100).collect();

    let url = format!(
        "https://api.stackexchange.com/2.3/search/advanced?order=desc&sort=relevance&q={}&site=stackoverflow",
        urlencoding::encode(&query)
    );

    let client = Client::builder()
        .user_agent("tokenectomy-cli")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let response = client.get(&url).send().await.ok()?;
    let json: Value = response.json().await.ok()?;

    let mut results = String::from("Based on automated Stack Overflow search:\n");
    let mut found = false;

    if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
        // Ambil 3 diskusi teratas
        for item in items.iter().take(3) {
            if let (Some(title), Some(link), Some(is_answered)) = (item.get("title"), item.get("link"), item.get("is_answered")) {
                let title = title.as_str().unwrap_or("");
                let link = link.as_str().unwrap_or("");
                let answered = if is_answered.as_bool().unwrap_or(false) { "✅ Answered" } else { "⏳ Unanswered" };
                
                results.push_str(&format!("- [{}] {}\n  {}\n", answered, title, link));
                found = true;
            }
        }
    }

    if found {
        Some(results)
    } else {
        None
    }
}
