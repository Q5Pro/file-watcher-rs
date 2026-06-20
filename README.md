# 👁️ file-watcher-rs

Belirtilen dosya(lar)ı veya klasörleri değişiklik için izler, bir
değişiklik tespit edildiğinde otomatik olarak bir komut çalıştırır.
`nodemon`, `entr` veya `watchexec`'e hafif, **bağımlılıksız** bir
alternatif — sadece Rust standart kütüphanesi kullanılır.

## Özellikler

- 📁 Tek dosya, birden fazla dosya veya klasör izleme
- 🌲 `--recursive` ile alt klasörleri de dahil etme
- 📎 `--ext` ile sadece belirli uzantılarla ilgilenme (örn. sadece `.rs` dosyaları)
- ➕ Yeni oluşturulan dosyaları da otomatik tespit etme
- 🧹 `target/`, `node_modules/`, `.git/` gibi gürültü klasörlerini otomatik atlama
- 🖥️ `--clear` ile her çalıştırmadan önce terminali temizleme
- ⚡ Düşük kaynak kullanımı (basit polling, dosya sistemi olay API'lerine bağımlı değil)

## Kurulum

```bash
cargo build --release
# Derlenen ikili dosya: target/release/file-watcher-rs
```

## Kullanım

```bash
# Tek bir dosyayı izle, değiştiğinde derle
file-watcher-rs src/main.rs --cmd "cargo build"

# Bir klasörü ve alt klasörlerini izle
file-watcher-rs ./src --cmd "cargo test" --recursive

# Sadece belirli uzantıları izle
file-watcher-rs ./src --cmd "cargo build" --recursive --ext rs,toml

# Kontrol aralığını değiştir (varsayılan 1000ms)
file-watcher-rs dosya.txt --cmd "echo değişti" --interval 300

# Her çalıştırmadan önce ekranı temizle (nodemon benzeri deneyim)
file-watcher-rs ./src --cmd "cargo run" --recursive --clear
```

| Parametre | Açıklama | Varsayılan |
|---|---|---|
| `--cmd` | Değişiklikte çalıştırılacak kabuk komutu (zorunlu) | — |
| `--interval` | Kontrol aralığı (ms) | 1000 |
| `--recursive`, `-r` | Alt klasörleri de tara | kapalı |
| `--ext` | Virgülle ayrılmış uzantı listesi (örn. `rs,toml`) | tüm dosyalar |
| `--clear` | Her çalıştırmadan önce terminali temizle | kapalı |

## Örnek: Rust projesi için otomatik test

```bash
file-watcher-rs ./src ./tests --cmd "cargo test" --recursive --ext rs --clear
```

Bu komut `src/` ve `tests/` klasörlerindeki herhangi bir `.rs` dosyası
değiştiğinde otomatik olarak `cargo test` çalıştırır ve her çalıştırma
öncesi terminali temizler.

## Nasıl çalışır?

Araç, işletim sisteminin dosya sistemi olay API'lerine (inotify,
FSEvents vb.) bağımlı olmak yerine basit bir **polling (anketleme)**
stratejisi kullanır: belirtilen aralıkla izlenen dosyaların son
değişiklik zamanını (`mtime`) kontrol eder ve önceki taramayla
karşılaştırır. Bu yaklaşım biraz daha fazla CPU kullanır (özellikle çok
kısa aralıklarda) ama platformlar arası taşınabilirlik sağlar ve hiçbir
işletim sistemine özel API'ye veya harici crate'e bağımlı değildir.

Klasörler her taramada yeniden listelenir, bu sayede **yeni oluşturulan
dosyalar** da otomatik olarak izleme kapsamına girer; silinen dosyalar
ise dahili takip haritasından çıkarılır.

## Lisans

MIT


---

> Made in [discord.gg/codeshare](https://discord.gg/codeshare) · [astra-dev.com.tr](https://astra-dev.com.tr)
