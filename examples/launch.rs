use claunch::{LaunchOptions, Launcher};
use std::collections::HashMap;
use std::env;
use std::path::Path;

fn main() {
    // Inicializar logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("=== CLaunch Working Example ===\n");

    // Estructura esperada:
    // game_dir/
    // ├── shared/
    // │   ├── libraries/    ← Aquí están las libs
    // │   ├── versions/     ← Aquí están los JARs
    // │   └── assets/       ← Aquí están los assets
    // └── instances/
    //     └── 1.20.1/       ← Directorio de la instancia

    // AJUSTA ESTOS VALORES:
    let game_dir = env::var("GAME_DIR").unwrap_or_else(|_| "/home/santiagolxx/.cubic".to_string());

    let version_id = "1.16.5";

    // Para versiones vanilla:
    let version_json = format!(
        "{}/shared/versions/{}/{}.json",
        game_dir, version_id, version_id
    );

    // Para versiones con mods (ej: Forge):
    // let version_json = format!("{}/shared/versions/1.20.1-forge-47.2.0/1.20.1-forge-47.2.0.json", game_dir);

    let instance_dir = format!("{}/instances/{}", game_dir, version_id);

    // Java path (ajusta según tu sistema)
    let java_path = "/usr/lib/jvm/java-17-openjdk/bin/java";

    println!("Configuration:");
    println!("  Game dir:      {}", game_dir);
    println!("  Version JSON:  {}", version_json);
    println!("  Instance dir:  {}", instance_dir);
    println!("  Java:          {}", java_path);
    println!();

    // Verificar estructura
    println!("Verifying directories...");
    verify_structure(&game_dir, &version_json);
    println!();

    // Launch options
    let options = LaunchOptions::new().with_demo(false);
    let mut custom_env = HashMap::new();
    custom_env.insert("DRI_PRIME".to_string(), "1".to_string());

    match Launcher::launch_with_options(
        &version_json,
        &game_dir,
        &instance_dir,
        "Player",
        &java_path,
        "2G",
        "4G",
        854,
        480,
        true, // cracked mode
        options,
        custom_env,
    ) {
        Ok(_) => {
            println!("\n✓ Game finished successfully!");
        }
        Err(e) => {
            eprintln!("\n❌ Launch failed: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("1. Make sure you've run the official launcher once to download all files");
            eprintln!("2. Check that the directory structure matches:");
            eprintln!("   {}/shared/libraries/", game_dir);
            eprintln!("   {}/shared/versions/{}/", game_dir, version_id);
            eprintln!("3. Verify the version JSON exists and is valid");
            eprintln!("4. Check the logs above for specific errors");
            std::process::exit(1);
        }
    }
}

fn verify_structure(game_dir: &str, version_json: &str) {
    let mut errors = Vec::new();

    // Check version JSON
    if !Path::new(version_json).exists() {
        errors.push(format!("❌ Version JSON not found: {}", version_json));
    } else {
        println!("  ✓ Version JSON exists");
    }

    // Check libraries
    let lib_dir = Path::new(game_dir).join("shared/libraries");
    if !lib_dir.exists() {
        errors.push(format!(
            "❌ Libraries directory not found: {}",
            lib_dir.display()
        ));
    } else {
        println!("  ✓ Libraries directory exists");

        // Count libraries
        if let Ok(entries) = std::fs::read_dir(&lib_dir) {
            let count = entries.count();
            if count == 0 {
                errors.push(format!("⚠️  Libraries directory is empty!"));
            } else {
                println!("  ✓ Found {} items in libraries/", count);
            }
        }
    }

    // Check versions
    let versions_dir = Path::new(game_dir).join("shared/versions");
    if !versions_dir.exists() {
        errors.push(format!(
            "❌ Versions directory not found: {}",
            versions_dir.display()
        ));
    } else {
        println!("  ✓ Versions directory exists");
    }

    // Check assets
    let assets_dir = Path::new(game_dir).join("shared/assets");
    if !assets_dir.exists() {
        errors.push(format!(
            "⚠️  Assets directory not found: {}",
            assets_dir.display()
        ));
    } else {
        println!("  ✓ Assets directory exists");
    }

    if !errors.is_empty() {
        eprintln!("\nErrors found:");
        for error in errors {
            eprintln!("  {}", error);
        }
        eprintln!("\nPlease fix these issues before launching.");
        std::process::exit(1);
    }
}
