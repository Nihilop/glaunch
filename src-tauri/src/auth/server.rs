use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use url::Url;

pub struct AuthServer {
    port: u16,
}

impl AuthServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    fn get_redirect_html(redirect_url: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
          <html>
          <head>
              <meta charset="UTF-8">
              <title>GLaunch - Authentification</title>
              <style>
                  body {{
                      background-color: #0f172a;
                      color: white;
                      font-family: system-ui, -apple-system, sans-serif;
                      display: flex;
                      align-items: center;
                      justify-content: center;
                      height: 100vh;
                      margin: 0;
                      text-align: center;
                  }}
                  .container {{
                      padding: 2rem;
                      border-radius: 1rem;
                      background-color: #1e293b;
                      box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
                  }}
                  .title {{
                      font-size: 1.5rem;
                      font-weight: bold;
                      margin-bottom: 1rem;
                  }}
                  .countdown {{
                      font-size: 3rem;
                      font-weight: bold;
                      margin: 1rem 0;
                      color: #3b82f6;
                  }}
                  .message {{
                      color: #94a3b8;
                  }}
              </style>
          </head>
          <body>
              <div class="container">
                  <div class="title">Authentification Réussie</div>
                  <div id="countdown" class="countdown">5</div>
                  <p class="message">Vous pouvez fermer cette fenêtre arrivé à 0...</p>
              </div>
              <script>

                  // Lancer le deep link immédiatement
                  window.location.href = '{redirect_url}';

                  // Démarrer le compte à rebours
                  let count = 5;
                  const countdown = setInterval(() => {{
                      count--;
                      document.getElementById('countdown').textContent = count;
                      if (count <= 0) {{
                          clearInterval(countdown);
                      }}
                  }}, 1000);
              </script>
          </body>
          </html>"#
        )
    }

    pub fn start(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))?;

        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut reader = BufReader::new(&stream);
                let mut request_line = String::new();
                reader.read_line(&mut request_line)?;

                if request_line.starts_with("GET") {
                    let redirect_url =
                        if let Some(identity) = self.extract_steam_identity(&request_line) {
                            format!("glaunch://auth/steam/callback?openid.identity={}", identity)
                        } else if let Some(code) = self.extract_epic_code(&request_line) {
                            format!("glaunch://auth/epic/callback?code={}", code)
                        } else if let Some(code) = self.extract_battlenet_code(&request_line) {
                            format!("glaunch://auth/battlenet/callback?code={}", code)
                        } else {
                            continue;
                        };

                    let html = Self::get_redirect_html(&redirect_url);
                    let response = format!(
                        "HTTP/1.1 200 OK\r\n\
                       Content-Type: text/html; charset=utf-8\r\n\
                       Content-Length: {}\r\n\
                       \r\n\
                       {}",
                        html.len(),
                        html
                    );

                    let _ = stream.write_all(response.as_bytes());
                }
            }
        }
        Ok(())
    }

    fn extract_url_parts(&self, request_line: &str) -> Option<Url> {
        let url_part = request_line.split_whitespace().nth(1)?;
        Url::parse(&format!("http://localhost{}", url_part)).ok()
    }

    fn extract_steam_identity(&self, request_line: &str) -> Option<String> {
        self.extract_url_parts(request_line).and_then(|url| {
            if url.path().contains("/auth/steam/callback") {
                url.query_pairs()
                    .find(|(key, _)| key == "openid.identity")
                    .map(|(_, value)| value.into_owned())
            } else {
                None
            }
        })
    }

    fn extract_epic_code(&self, request_line: &str) -> Option<String> {
        self.extract_url_parts(request_line).and_then(|url| {
            if url.path().contains("/auth/epic/callback") {
                url.query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, value)| value.into_owned())
            } else {
                None
            }
        })
    }

    fn extract_battlenet_code(&self, request_line: &str) -> Option<String> {
        self.extract_url_parts(request_line).and_then(|url| {
            if url.path().contains("/auth/battlenet/callback") {
                url.query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, value)| value.into_owned())
            } else {
                None
            }
        })
    }
}
