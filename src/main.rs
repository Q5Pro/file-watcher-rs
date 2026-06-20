//! file-watcher-rs: Belirtilen dosya(lar)ı veya klasörü değişiklik için
//! izler, bir değişiklik tespit edildiğinde belirtilen komutu otomatik
//! çalıştırır. `nodemon`, `entr` veya `watchexec`'e hafif, bağımlılıksız
//! bir alternatif — yalnızca Rust standart kütüphanesi kullanılır
//! (dosya sistemi olaylarını dinlemek için işletim sistemi API'leri
//! yerine basit polling/anketleme stratejisi izlenir, bu da taşınabilir
//! ve bağımlılıksız olmasını sağlar).
//!
//! Kullanım:
//!     file-watcher-rs src/main.rs --cmd "cargo build"
//!     file-watcher-rs ./src --cmd "cargo test" --recursive
//!     file-watcher-rs *.py --cmd "python test.py" --interval 500
//!     file-watcher-rs ./watch_dir --ext rs,toml --cmd "echo değişti"

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::thread;
use std::time::{Duration, SystemTime};

struct Args {
    paths: Vec<String>,
    command: Option<String>,
    interval_ms: u64,
    recursive: bool,
    extensions: Option<Vec<String>>,
    clear_screen: bool,
}

fn parse_args(raw: &[String]) -> Args {
    let mut paths = Vec::new();
    let mut command = None;
    let mut interval_ms = 1000;
    let mut recursive = false;
    let mut extensions = None;
    let mut clear_screen = false;

    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--cmd" => {
                command = raw.get(i + 1).cloned();
                i += 1;
            }
            "--interval" => {
                if let Some(val) = raw.get(i + 1) {
                    interval_ms = val.parse().unwrap_or(1000);
                }
                i += 1;
            }
            "--recursive" | "-r" => recursive = true,
            "--clear" => clear_screen = true,
            "--ext" => {
                if let Some(val) = raw.get(i + 1) {
                    extensions = Some(val.split(',').map(|s| s.trim().to_string()).collect());
                }
                i += 1;
            }
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            other => paths.push(other.to_string()),
        }
        i += 1;
    }

    Args { paths, command, interval_ms, recursive, extensions, clear_screen }
}

fn print_help() {
    println!("file-watcher-rs — dosya değişikliklerini izleyip komut çalıştırır\n");
    println!("Kullanım:");
    println!("  file-watcher-rs <yol...> --cmd \"<komut>\" [seçenekler]\n");
    println!("Seçenekler:");
    println!("  --cmd <komut>     Değişiklikte çalıştırılacak kabuk komutu (zorunlu)");
    println!("  --interval <ms>   Kontrol aralığı, milisaniye (varsayılan: 1000)");
    println!("  --recursive, -r   Klasörlerde alt klasörleri de tara");
    println!("  --ext <liste>     Sadece belirtilen uzantılarla ilgilen (örn. 'rs,toml')");
    println!("  --clear           Her çalıştırmadan önce terminali temizle");
    println!("  --help, -h        Bu yardım mesajını göster");
}

/// Bir dosyanın son değişiklik zamanını döndürür. Hata durumunda None.
fn get_mtime(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok()?.modified().ok()
}

/// Verilen yolların altındaki tüm dosyaları toplar (klasörse, isteğe
/// bağlı olarak alt klasörlere de iner). Uzantı filtresi varsa uygular.
fn collect_files(paths: &[String], recursive: bool, extensions: &Option<Vec<String>>) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for p in paths {
        let path = Path::new(p);
        if path.is_file() {
            if matches_extension(path, extensions) {
                files.push(path.to_path_buf());
            }
        } else if path.is_dir() {
            collect_dir(path, recursive, extensions, &mut files);
        }
    }

    files
}

fn collect_dir(dir: &Path, recursive: bool, extensions: &Option<Vec<String>>, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Yaygın gürültü kaynaklarını atla
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name == "target" || name == "node_modules" || name == ".git" {
                    continue;
                }
            }
            if recursive {
                collect_dir(&path, recursive, extensions, out);
            }
        } else if matches_extension(&path, extensions) {
            out.push(path);
        }
    }
}

fn matches_extension(path: &Path, extensions: &Option<Vec<String>>) -> bool {
    match extensions {
        None => true,
        Some(exts) => {
            match path.extension().and_then(|e| e.to_str()) {
                Some(ext) => exts.iter().any(|e| e == ext),
                None => false,
            }
        }
    }
}

fn run_command(cmd: &str, clear_screen: bool) {
    if clear_screen {
        print!("\x1B[2J\x1B[1;1H"); // ANSI: ekranı temizle, imleci sola üste al
    }

    println!("\n▶ Değişiklik tespit edildi, çalıştırılıyor: {}", cmd);
    println!("{}", "-".repeat(50));

    let status = Command::new("sh").arg("-c").arg(cmd).status();

    match status {
        Ok(s) if s.success() => println!("{}\n✓ Komut başarıyla tamamlandı.", "-".repeat(50)),
        Ok(s) => println!("{}\n✗ Komut hata koduyla bitti: {:?}", "-".repeat(50), s.code()),
        Err(e) => println!("✗ Komut çalıştırılamadı: {}", e),
    }
}

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    let args = parse_args(&raw_args);

    if args.paths.is_empty() {
        eprintln!("Hata: En az bir dosya veya klasör yolu belirtmelisiniz.");
        print_help();
        process::exit(1);
    }

    let command = match &args.command {
        Some(c) => c.clone(),
        None => {
            eprintln!("Hata: --cmd parametresi zorunludur.");
            process::exit(1);
        }
    };

    println!("👀 İzleniyor: {}", args.paths.join(", "));
    println!("⏱  Kontrol aralığı: {}ms", args.interval_ms);
    if let Some(exts) = &args.extensions {
        println!("📎 Uzantı filtresi: {}", exts.join(", "));
    }
    println!("🛑 Durdurmak için Ctrl+C\n");

    let mut last_mtimes: HashMap<PathBuf, SystemTime> = HashMap::new();

    // Başlangıç durumunu kaydet (ilk taramada komut çalıştırılmaz,
    // sadece referans alınır)
    for file in collect_files(&args.paths, args.recursive, &args.extensions) {
        if let Some(mtime) = get_mtime(&file) {
            last_mtimes.insert(file, mtime);
        }
    }

    loop {
        thread::sleep(Duration::from_millis(args.interval_ms));

        let current_files = collect_files(&args.paths, args.recursive, &args.extensions);
        let mut changed = false;
        let mut changed_file = String::new();

        for file in &current_files {
            if let Some(mtime) = get_mtime(file) {
                match last_mtimes.get(file) {
                    Some(old_mtime) if *old_mtime != mtime => {
                        changed = true;
                        changed_file = file.display().to_string();
                        last_mtimes.insert(file.clone(), mtime);
                    }
                    None => {
                        // Yeni oluşturulan dosya
                        changed = true;
                        changed_file = file.display().to_string();
                        last_mtimes.insert(file.clone(), mtime);
                    }
                    _ => {}
                }
            }
        }

        // Silinen dosyaları tespit et (artık current_files içinde değil)
        let current_set: std::collections::HashSet<_> = current_files.iter().collect();
        last_mtimes.retain(|path, _| current_set.contains(path) || {
            false // silinmiş dosyaları haritadan çıkar
        });

        if changed {
            println!("📝 Değişen dosya: {}", changed_file);
            run_command(&command, args.clear_screen);
        }
    }
}
