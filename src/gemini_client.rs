use crate::models::Language;
use crate::aggregation::ScopeAggregation;
use serde_json::json;
use reqwest::Client;

pub async fn generate_narrative(agg: &ScopeAggregation, lang: &Language, api_key: &str) -> Result<String, String> {
    let system_prompt = match lang {
        Language::German => "Du bist ein leitender ESG-Wirtschaftsprüfer. Schreibe eine 3-Absatz-Zusammenfassung für dieses Inventar.",
        Language::English => "You are a lead ESG auditor. Write a 3-paragraph summary for this inventory.",
        Language::Hungarian => "Te egy vezető ESG könyvvizsgáló vagy. Írj egy 3 bekezdéses összefoglalót ehhez a leltárhoz.",
    };

    let fallback = match lang {
        Language::German => format!(
            "Zusammenfassung: Scope 1 beträgt {:.2} tCO2e, Scope 2 {:.2} tCO2e und Scope 3 {:.2} tCO2e.",
            agg.scope1_total_tco2e, agg.scope2_total_tco2e, agg.scope3_data.grand_total_tco2e
        ),
        Language::English => format!(
            "Summary: Scope 1 is {:.2} tCO2e, Scope 2 is {:.2} tCO2e, and Scope 3 is {:.2} tCO2e.",
            agg.scope1_total_tco2e, agg.scope2_total_tco2e, agg.scope3_data.grand_total_tco2e
        ),
        Language::Hungarian => format!(
            "Összefoglaló: Scope 1: {:.2} tCO2e, Scope 2: {:.2} tCO2e, Scope 3: {:.2} tCO2e.",
            agg.scope1_total_tco2e, agg.scope2_total_tco2e, agg.scope3_data.grand_total_tco2e
        ),
    };

    let data_prompt = format!(
        "Data: Scope 1: {:.2}, Scope 2: {:.2}, Scope 3: {:.2}. Total: {:.2}.",
        agg.scope1_total_tco2e,
        agg.scope2_total_tco2e,
        agg.scope3_data.grand_total_tco2e,
        agg.total_inventory_tco2e
    );

    let client = Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
        api_key
    );

    let body = json!({
        "contents": [{
            "parts": [{
                "text": format!("{}\n\n{}", system_prompt, data_prompt)
            }]
        }]
    });

    let res = client.post(url)
        .json(&body)
        .send()
        .await;

    match res {
        Ok(resp) => {
            if resp.status().is_success() {
                let json_res: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
                let text = json_res["candidates"][0]["content"]["parts"][0]["text"]
                    .as_str()
                    .map(|s| s.to_string());
                
                match text {
                    Some(t) => Ok(t),
                    None => Ok(fallback)
                }
            } else {
                Ok(fallback)
            }
        }
        Err(_) => Ok(fallback)
    }
}
