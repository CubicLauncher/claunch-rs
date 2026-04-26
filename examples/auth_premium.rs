use claunch_rs::auth::microsoft::MicrosoftAuth;
use claunch_rs::{LaunchOptions, Launcher, MinecraftUser};
use std::collections::HashMap;
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    println!("=== Minecraft Premium Auth Example ===");

    // 1. Get Device Code
    println!("Solicitando código de dispositivo...");
    let ms_auth = MicrosoftAuth::default();
    let device_code_res = ms_auth.get_device_code()?;

    println!("\nPor favor, ve a: {}", device_code_res.verification_uri);
    println!("Ingresa el código: {}", device_code_res.user_code);
    println!("\nEsperando a que completes el inicio de sesión...");

    // 2. Authenticate
    let user = ms_auth.authenticate_with_device_code(
        &device_code_res.device_code,
        device_code_res.interval,
        device_code_res.expires_in,
    )?;

    println!("\n¡Autenticación exitosa!");
    println!("Usuario: {}", user.username);
    println!("UUID: {}", user.uuid);

    // 2.1 Guardar tokens de forma segura
    println!("\nGuardando tokens de forma segura en el llavero...");
    user.save_tokens()?;

    // 2.2 Demostración de serialización segura
    let json = serde_json::to_string(&user)?;
    println!("JSON serializado (los tokens se omiten automáticamente):");
    println!("{}", json);

    // 2.3 Simulación de carga desde archivo
    println!("\nSimulando carga de usuario desde configuración...");
    let mut loaded_user: MinecraftUser = serde_json::from_str(&json)?;
    println!("Tokens antes de cargar: access='{}'", loaded_user.access_token);
    
    println!("Cargando tokens desde el llavero...");
    loaded_user.load_tokens()?;
    println!("Tokens cargados correctamente.");

    // 3. Launch (Configura tus rutas aquí)
    let base_dir = env::var("BASE_DIR").unwrap_or_else(|_| "/home/not_staff/.minecraft".to_string());
    let version_id = "1.20.1";
    let shared_dir = Path::new(&base_dir).join("shared");
    let version_json = shared_dir.join("versions").join(version_id).join(format!("{}.json", version_id));
    let instance_dir = Path::new(&base_dir).join("instances").join(version_id);
    let java_path = "/usr/bin/java";

    println!("\nLanzando el juego...");
    Launcher::launch_with_options(
        &version_json,
        &shared_dir,
        &instance_dir,
        loaded_user, // Usamos el usuario con tokens cargados
        &java_path,
        "2G",
        "4G",
        854,
        480,
        LaunchOptions::default(),
        HashMap::new(),
    )?;

    Ok(())
}
