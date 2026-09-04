use std::env;
use std::fs::{self, File};
use std::path::Path;
use std::process::Command;
use sha2::{Sha256, Digest};
use serde::Deserialize;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Cole aqui o link direto do seu "update.json" hospedado nas Releases do GitHub
// Exemplo: https://github.com/seu-usuario/seu-repo/releases/download/v1.0.0/update.json
const REPOSITORY_URL: &str = "https://github.com/HACKS-EXE/Update";

#[derive(Deserialize, Debug)]
struct UpdateManifest {
    version: String,
    hash: String,
    download_url: String, // O link direto para o .exe novo na Release do GitHub
}

fn parse_version(v: &str) -> Vec<u32> {
    v.split('.').filter_map(|s| s.parse().ok()).collect()
}

fn is_newer(local: &str, remote: &str) -> bool {
    parse_version(remote) > parse_version(local)
}

fn compute_hash(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).ok()?;
    Some(format!("{:x}", hasher.finalize()))
}

fn main() {
    let current_exe = env::current_exe().unwrap();
    let old_exe = current_exe.with_extension("old");

    println!(">>> Iniciando aplicativo... Versão atual: v{}", CURRENT_VERSION);

    // 1. Limpeza de resíduos de atualizações passadas
    if old_exe.exists() {
        let _ = fs::remove_file(&old_exe);
    }

    println!(">>> Verificando atualizações no GitHub...");

    // 2. Consulta o repositório no GitHub e descobre a última Release
    let api_url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        REPOSITORY_URL
            .trim_start_matches("https://github.com/")
            .trim_end_matches('/')
    );

    let client = reqwest::blocking::Client::new();

    if let Ok(response) = client
        .get(&api_url)
        .header("User-Agent", "Fenix-Launcher-Updater")
        .header("Accept", "application/vnd.github+json")
        .send()
    {
        #[derive(Deserialize)]
        struct ReleaseAsset {
            name: String,
            browser_download_url: String,
        }

        #[derive(Deserialize)]
        struct GitHubRelease {
            tag_name: String,
            assets: Vec<ReleaseAsset>,
        }

        if let Ok(release) = response.json::<GitHubRelease>() {
            // Procura o update.json dentro dos arquivos da última Release
            if let Some(manifest_asset) = release.assets.iter().find(|asset| asset.name == "update.json") {

                if let Ok(manifest_response) = client
                    .get(&manifest_asset.browser_download_url)
                    .header("User-Agent", "Fenix-Launcher-Updater")
                    .send()
                {
                    if let Ok(manifest) = manifest_response.json::<UpdateManifest>() {

                        if is_newer(CURRENT_VERSION, &manifest.version) {
                            println!(">>> Nova versão encontrada no GitHub: v{}!", manifest.version);
                            println!(">>> Baixando nova versão...");

                            // 3. Renomeia o executável atual para liberar o arquivo
                            if fs::rename(&current_exe, &old_exe).is_ok() {

                                let mut success = false;

                                // Baixa o novo executável usando a URL existente no update.json
                                if let Ok(mut download_resp) =
                                    client.get(&manifest.download_url)
                                        .header("User-Agent", "Fenix-Launcher-Updater")
                                        .send()
                                {
                                    if let Ok(mut file) = File::create(&current_exe) {
                                        if download_resp.copy_to(&mut file).is_ok() {
                                            success = true;
                                        }
                                    }
                                }

                                // 4. Validação de Hash SHA256
                                if success {
                                    if let Some(actual_hash) = compute_hash(&current_exe) {
                                        if actual_hash == manifest.hash {
                                            println!(">>> Hash validado com sucesso! Reiniciando aplicativo...");
                                            Command::new(&current_exe)
                                                .spawn()
                                                .expect("Falha ao iniciar nova versão");

                                            std::process::exit(0);
                                        } else {
                                            println!(">>> Erro: o hash do arquivo baixado não confere.");
                                            success = false;
                                        }
                                    } else {
                                        success = false;
                                    }
                                }

                                // 5. Rollback caso algo dê errado
                                if !success {
                                    println!(">>> Revertendo para a versão anterior...");
                                    let _ = fs::remove_file(&current_exe);
                                    let _ = fs::rename(&old_exe, &current_exe);
                                }
                            }
                        } else {
                            println!(">>> O aplicativo já está atualizado.");
                        }

                    } else {
                        println!(">>> Não foi possível interpretar o update.json.");
                    }
                } else {
                    println!(">>> Não foi possível baixar o update.json da Release.");
                }

            } else {
                println!(
                    ">>> A última Release ({}) não possui o arquivo update.json.",
                    release.tag_name
                );
            }

        } else {
            println!(">>> Não foi possível interpretar a resposta da API do GitHub.");
        }

    } else {
        println!(">>> Não foi possível conectar ao GitHub para verificar atualizações.");
    }

    // --- SUA LÓGICA PRINCIPAL AQUI ---
    println!(">>> Executando o sistema normalmente...");
    
    let mut input = String::new();
    println!("Pressione ENTER para fechar.");
    std::io::stdin().read_line(&mut input).unwrap();
}