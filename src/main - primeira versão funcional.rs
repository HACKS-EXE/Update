use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use sha2::{Sha256, Digest};

// Puxa a versão automaticamente do Cargo.toml
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Converte "1.0.1" em [1, 0, 1] para comparar corretamente
fn parse_version(v: &str) -> Vec<u32> {
    v.split('.').filter_map(|s| s.parse().ok()).collect()
}

fn is_newer(local: &str, remote: &str) -> bool {
    parse_version(remote) > parse_version(local)
}

fn compute_hash(path: &Path) -> Option<Vec<u8>> {
    let mut file = File::open(path).ok()?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).ok()?;
    Some(hasher.finalize().to_vec())
}

// Gera um arquivo binário customizado (ilegível para humanos)
fn generate_metadata(exe_path: &Path, meta_path: &Path, version: &str) {
    if meta_path.exists() { return; } // Gera apenas uma vez
    
    let hash = compute_hash(exe_path).expect("Falha ao gerar hash");
    let mut out = File::create(meta_path).unwrap();
    
    // Formato: [1 byte com o tamanho da versão] + [texto da versão] + [32 bytes do hash]
    out.write_all(&(version.len() as u8).to_le_bytes()).unwrap();
    out.write_all(version.as_bytes()).unwrap();
    out.write_all(&hash).unwrap();
    
    println!("Metadados seguros gerados em: {:?}", meta_path.file_name().unwrap());
}

fn read_metadata(meta_path: &Path) -> Option<(String, Vec<u8>)> {
    let mut file = File::open(meta_path).ok()?;
    
    let mut len_buf = [0u8; 1];
    file.read_exact(&mut len_buf).ok()?;
    
    let mut version_buf = vec![0u8; len_buf[0] as usize];
    file.read_exact(&mut version_buf).ok()?;
    let version = String::from_utf8(version_buf).ok()?;
    
    let mut hash_buf = vec![0u8; 32];
    file.read_exact(&mut hash_buf).ok()?;
    
    Some((version, hash_buf))
}

fn main() {
    let current_exe = env::current_exe().unwrap();
    let current_dir = current_exe.parent().unwrap();
    let exe_name = current_exe.file_name().unwrap();
    
    let old_exe = current_exe.with_extension("old");
    let meta_file = current_dir.join("app_data.bin");

    println!(">>> Iniciando aplicativo... Versão atual: {}", CURRENT_VERSION);

    // 1. Limpeza da versão antiga
    if old_exe.exists() {
        let _ = fs::remove_file(&old_exe);
        println!(">>> Resíduo da versão antiga (.old) deletado com sucesso.");
    }

    // 2. Gerar metadados do executável atual
    generate_metadata(&current_exe, &meta_file, CURRENT_VERSION);

    // 3. Simular a busca por atualização num diretório local
    let update_dir = current_dir.join("update_server");
    let remote_meta = update_dir.join("app_data.bin");
    let remote_exe = update_dir.join(exe_name);

    if let Some((remote_version, expected_hash)) = read_metadata(&remote_meta) {
        if is_newer(CURRENT_VERSION, &remote_version) {
            println!(">>> Nova versão encontrada: {}! (Atual: {})", remote_version, CURRENT_VERSION);

            if let Some(actual_hash) = compute_hash(&remote_exe) {
                if actual_hash == expected_hash {
                    println!(">>> Integridade confirmada: O Hash do executável bate com os metadados.");
                    println!(">>> Baixando/Instalando atualização...");

                    // Renomeia o processo atual e copia o novo
                    fs::rename(&current_exe, &old_exe).expect("Erro ao renomear executável ativo");
                    fs::copy(&remote_exe, &current_exe).expect("Erro ao copiar nova versão");

                    // Inicia o novo aplicativo
                    Command::new(&current_exe).spawn().expect("Falha ao abrir a nova versão");

                    println!(">>> Atualização concluída. Auto-finalizando processo obsoleto...");
                    std::process::exit(0);
                } else {
                    println!(">>> ERRO: O arquivo da nova versão está corrompido ou foi adulterado (Hash mismatch).");
                }
            }
        } else {
            println!(">>> O aplicativo já está na versão mais recente.");
        }
    } else {
        println!(">>> Nenhuma atualização localizada no diretório 'update_server'.");
    }

    // Código principal do seu aplicativo rodaria aqui
    println!(">>> Executando lógica principal do launcher...");
    
    // Pausa apenas para você conseguir ler o CMD durante os testes
    let mut input = String::new();
    println!("Pressione ENTER para fechar.");
    std::io::stdin().read_line(&mut input).unwrap();
}