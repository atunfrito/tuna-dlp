// Comprime yt-dlp y ffmpeg en un único bloque zstd dentro de OUT_DIR para que
// src/binarios.rs lo meta en el .exe con include_bytes!. Van juntos a propósito:
// ffmpeg y ffprobe comparten casi todo su código y así solo se paga una vez.
use std::{env, fs, io::Write, path::Path};

const BINARIOS: [&str; 3] = ["yt-dlp.exe", "ffmpeg.exe", "ffprobe.exe"];

fn main() {
    let out = env::var("OUT_DIR").unwrap();
    let hilos = std::thread::available_parallelism().map_or(1, |n| n.get() as u32);

    let mut enc = zstd::Encoder::new(Vec::new(), 19).unwrap();
    enc.multithread(hilos).unwrap();
    enc.long_distance_matching(true).unwrap();
    // 2^27 = 128 MB: la ventana alcanza de ffprobe hasta el principio de ffmpeg.
    enc.window_log(27).unwrap();

    let mut indice = String::from("pub const BINARIOS: [(&str, usize); 3] = [\n");
    let mut firma: u64 = 0xcbf2_9ce4_8422_2325;
    for nombre in BINARIOS {
        let origen = Path::new("bin").join(nombre);
        println!("cargo:rerun-if-changed={}", origen.display());
        let datos = fs::read(&origen).unwrap_or_else(|_| {
            panic!("falta {}: ejecuta scripts/binarios.ps1", origen.display())
        });
        // FNV-1a: cambia la carpeta de extracción cuando cambia algún binario.
        for b in &datos {
            firma ^= *b as u64;
            firma = firma.wrapping_mul(0x100_0000_01b3);
        }
        indice += &format!("    ({nombre:?}, {}),\n", datos.len());
        enc.write_all(&datos).unwrap();
    }
    indice += "];\n";
    indice += &format!("pub const FIRMA: &str = \"{firma:016x}\";\n");

    fs::write(Path::new(&out).join("binarios.zst"), enc.finish().unwrap()).unwrap();
    fs::write(Path::new(&out).join("indice.rs"), indice).unwrap();
    tauri_build::build()
}
