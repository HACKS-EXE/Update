use std::env;
use std::fs::{self, File};
use std::path::Path;
use std::process::Command;
use sha2::{Sha256, Digest};
use serde::Deserialize;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Cole aqui o link direto do seu "update.json" hospedado nas Releases do GitHub
// Exemplo: https://github.com/seu-usuario/seu-repo/releases/download/v1.0.0/update.json
const MANIFEST_URL: &str = "https://github.com/HACKS-EXE/Update";

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

    // 2. Baixa o manifesto JSON do GitHub
    if let Ok(response) = reqwest::blocking::get(MANIFEST_URL) {
        if let Ok(manifest) = response.json::<UpdateManifest>() {
            
            if is_newer(CURRENT_VERSION, &manifest.version) {
                println!(">>> Nova versão encontrada no GitHub: v{}!", manifest.version);
                println!(">>> Baixando nova versão...");

                // 3. Renomeia o executável atual para liberar o arquivo
                if fs::rename(&current_exe, &old_exe).is_ok() {
                    
                    let mut success = false;
                    // Baixa o novo executável diretamente da URL fornecida no JSON do GitHub
                    if let Ok(mut download_resp) = reqwest::blocking::get(&manifest.download_url) {
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
                                Command::new(&current_exe).spawn().expect("Falha ao iniciar nova versão");
                                std::process::exit(0);
                            } else {
                                println!(">>> Erro: O hash do arquivo baixado não confere. Cancelando...");
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
        }
    } else {
        println!(">>> Não foi possível conectar ao GitHub para checar atualizações.");
    }

    // --- SUA LÓGICA PRINCIPAL AQUI ---
    println!(">>> Executando o sistema normalmente...");
    
    let mut input = String::new();
    println!("Pressione ENTER para fechar.");
    std::io::stdin().read_line(&mut input).unwrap();
}