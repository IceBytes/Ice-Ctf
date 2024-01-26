use sandy::Server;
use sandy::TemplateEngine;
use std::collections::HashMap;
use urlencoding::decode;

fn generate_payload(words: Vec<&str>) -> Option<String> {
    if words.iter().all(|&word| word.len() >= 4) {
        let payload: String = words.iter().map(|word| word.chars().last().unwrap()).collect();
        Some(payload)
    } else {
        None
    }
}

fn filter_payload(payload: &str) -> String {
    payload
        .chars()
        .filter(|&c| c.is_alphanumeric() || ['<', '>', '(', ')', '/', ' ', '='].contains(&c))
        .collect()
}

fn main() {
    let mut server = Server::new();
    let flag = "CTF(\"Ice Projects Flag\")".to_string();

    server.route("/", move |_, _, method, data| {
        let mut result: Option<String> = None;
        let mut flag_message: Option<String> = None;

        if method == "POST" {
            if let Some(payload) = data.get("payload").map(|s| {
                decode(s).map_err(|e| {
                    eprintln!("Error decoding payload: {}", e);
                    "HTTP/1.1 200 OK\n\nFailed to decode payload"
                })
            }) {
                let decoded = match payload {
                    Ok(decoded) => decoded.clone(),
                    Err(_) => {
                        eprintln!("Failed to decode payload");
                        return Ok("HTTP/1.1 500 Internal Server Error\n\nFailed to decode payload".to_string());
                    }
                };

                let payload_words: Vec<&str> = decoded.split('+').collect();
                if payload_words.iter().all(|&word| word.len() >= 4) {
                    if let Some(generated_payload) = generate_payload(payload_words) {
                        let filtered_result = filter_payload(&generated_payload);
                        result = Some(filtered_result.clone());

                        if filtered_result == "<script>alert(1)</script>" {
                            flag_message = Some("Congratulations! You have successfully solved the challenge.".to_string());
                        } else {
                            flag_message = Some("Failed to solve the challenge: Check the generated code.".to_string());
                        }
                    } else {
                        flag_message = Some("Failed to generate data: Check the input words.".to_string());
                    }
                } else {
                    flag_message = Some("Failed to generate data: Make sure each word has at least 4 characters.".to_string());
                }
            }
        }

        let mut context = HashMap::new();
        context.insert("result", result.as_ref().map(|s| s.as_str()).unwrap_or_default());
        context.insert("flag_message", flag_message.as_ref().map(|s| s.as_str()).unwrap_or_default());
        let flag_clone = flag.clone();
        context.insert("flag", &flag_clone);

        match TemplateEngine::render_template("index1.html", &context) {
            Ok(rendered) => Ok(format!("HTTP/1.1 200 OK\n\n{}", rendered)),
            Err(err) => {
                println!("Error: {}", err);
                Ok(format!("HTTP/1.1 500 Internal Server Error\n\n{}", err))
            }
        }
    });

    server.run("0.0.0.0", "8080");
}
