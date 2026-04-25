use claunch_rs::{LaunchOptions, Launcher};
use std::collections::HashMap;
use std::env;
use std::path::Path;

fn main() {
    // Inicializar logger para ver detalles
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("=== CLaunch Example (estructura con shared/) ===\n");

    // ===== CONFIGURACIÓN =====
    // Ruta base (la raíz donde están shared/ e instances/)
    let base_dir = env::var("BASE_DIR").unwrap_or_else(|_| "/home/santiagolxx/.cubic".to_string());
    let version_id = "1.12.2";

    let shared_dir = Path::new(&base_dir).join("shared");
    let version_json = shared_dir
        .join("versions")
        .join(version_id)
        .join(format!("{}.json", version_id));
    let instance_dir = Path::new(&base_dir).join("instances").join(version_id);

    let java_path = "/usr/lib/jvm/java-17-openjdk/bin/java";

    println!("Configuración:");
    println!("  Base dir:      {}", base_dir);
    println!("  Shared dir:    {}", shared_dir.display());
    println!("  Version JSON:  {}", version_json.display());
    println!("  Instance dir:  {}", instance_dir.display());
    println!("  Java:          {}", java_path);
    println!();

    // ===== VERIFICACIÓN DE ESTRUCTURA =====
    println!("Verificando directorios...");
    verify_structure(&base_dir, &version_id, &version_json);
    println!();

    // ===== OPCIONES DE LANZAMIENTO =====
    let options = LaunchOptions::new().with_demo(false);
    let mut custom_env = HashMap::new();
    custom_env.insert("DRI_PRIME".to_string(), "1".to_string());

    // ===== LANZAR =====
    match Launcher::launch_with_options(
        &version_json,
        &shared_dir, // segundo argumento: directorio que contiene libraries/, assets/, versions/
        &instance_dir, // tercer argumento: directorio de la instancia (mundo, config)
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
            println!("\n✓ Juego terminado correctamente!");
        }
        Err(e) => {
            eprintln!("\n❌ Error al lanzar: {}", e);
            eprintln!("\nPosibles causas:");
            eprintln!("1. El JSON de versión no existe o está corrupto");
            eprintln!(
                "2. Las librerías no se han descargado en: {}/libraries",
                shared_dir.display()
            );
            eprintln!(
                "3. El JAR de la versión no está en: {}/versions/{}/{}.jar",
                shared_dir.display(),
                version_id,
                version_id
            );
            eprintln!("4. La versión de Java no es compatible");
            std::process::exit(1);
        }
    }
}

/// Verifica que existan los directorios y archivos necesarios
fn verify_structure(base_dir: &str, version_id: &str, version_json: &Path) {
    let mut errors = Vec::new();

    // 1. Version JSON
    if !version_json.exists() {
        errors.push(format!(
            "❌ Version JSON no encontrado: {}",
            version_json.display()
        ));
    } else {
        println!("  ✓ Version JSON existe");
    }

    // 2. Directorio de librerías (dentro de shared/)
    let lib_dir = Path::new(base_dir).join("shared").join("libraries");
    if !lib_dir.exists() {
        errors.push(format!(
            "❌ Directorio libraries no encontrado: {}",
            lib_dir.display()
        ));
    } else {
        println!("  ✓ Directorio libraries existe");
        // Contar cuántos .jar hay (opcional)
        if let Ok(entries) = std::fs::read_dir(&lib_dir) {
            let count = entries.count();
            if count == 0 {
                errors.push("⚠️  El directorio libraries está vacío".to_string());
            } else {
                println!("  ✓ Se encontraron {} elementos en libraries/", count);
            }
        }
    }

    // 3. Directorio de versions (dentro de shared/)
    let versions_dir = Path::new(base_dir).join("shared").join("versions");
    if !versions_dir.exists() {
        errors.push(format!(
            "❌ Directorio versions no encontrado: {}",
            versions_dir.display()
        ));
    } else {
        println!("  ✓ Directorio versions existe");
    }

    // 4. JAR de la versión
    let version_jar = versions_dir
        .join(version_id)
        .join(format!("{}.jar", version_id));
    if !version_jar.exists() {
        errors.push(format!(
            "❌ JAR de versión no encontrado: {}",
            version_jar.display()
        ));
    } else {
        println!("  ✓ JAR de versión existe");
    }

    // 5. Directorio de assets
    let assets_dir = Path::new(base_dir).join("shared").join("assets");
    if !assets_dir.exists() {
        errors.push(format!(
            "⚠️  Directorio assets no encontrado: {}",
            assets_dir.display()
        ));
    } else {
        println!("  ✓ Directorio assets existe");
    }

    if !errors.is_empty() {
        eprintln!("\nErrores encontrados:");
        for err in errors {
            eprintln!("  {}", err);
        }
        eprintln!("\nCorrige los problemas antes de lanzar.");
        std::process::exit(1);
    }
}
