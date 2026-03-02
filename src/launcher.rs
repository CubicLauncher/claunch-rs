// Copyright (C) 2025 Santiagolxx, Notstaff and CubicLauncher contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::models::{LaunchOptions, VersionInfo};
use crate::resolvers::{CommandBuilder, DependencyResolver};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

/// Universal Minecraft Launcher – Versión con logging detallado
pub struct Launcher;

impl Launcher {
    /// Lanzamiento simple (mantenido por compatibilidad)
    pub fn launch(
        version_json_path: impl AsRef<Path>,
        game_dir: impl AsRef<Path>,
        instance_dir: impl AsRef<Path>,
        username: &str,
        java_path: impl AsRef<Path>,
        min_ram: &str,
        max_ram: &str,
        width: u32,
        height: u32,
        cracked: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Self::launch_with_options(
            version_json_path,
            game_dir,
            instance_dir,
            username,
            java_path,
            min_ram,
            max_ram,
            width,
            height,
            cracked,
            LaunchOptions::default(),
            HashMap::new(),
        )
    }

    /// Lanzamiento con opciones – con logging detallado
    pub fn launch_with_options(
        version_json_path: impl AsRef<Path>,
        shared_dir: impl AsRef<Path>,
        game_dir: impl AsRef<Path>,
        username: &str,
        java_path: impl AsRef<Path>,
        min_ram: &str,
        max_ram: &str,
        width: u32,
        height: u32,
        cracked: bool,
        options: LaunchOptions,
        custom_env: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("========== CUBICLAUNCHER CLAUNCH ==========");
        debug!("[1] Parámetros recibidos:");
        debug!(
            "    version_json_path: {}",
            version_json_path.as_ref().display()
        );
        debug!("    shared_dir:        {}", shared_dir.as_ref().display());
        debug!("    game_dir:          {}", game_dir.as_ref().display());
        debug!("    username:          {}", username);
        debug!("    java_path:         {}", java_path.as_ref().display());
        debug!("    min_ram:           {}", min_ram);
        debug!("    max_ram:           {}", max_ram);
        debug!("    width x height:    {}x{}", width, height);
        debug!("    cracked:           {}", cracked);
        debug!("    demo_mode:         {}", options.demo_mode);
        if !custom_env.is_empty() {
            debug!("    custom_env:        {:?}", custom_env);
        }

        // ------------------------------------------------------------
        // Crear VersionInfo (aquí se construyen las rutas internas)
        // ------------------------------------------------------------
        debug!("[2] Creando VersionInfo...");
        let info = VersionInfo::new(version_json_path, shared_dir.as_ref(), game_dir.as_ref())?;
        debug!("    version_id:        {}", info.version_id);
        debug!("    lib_dir:           {}", info.lib_dir.display());
        debug!("    assets_dir:        {}", info.assets_dir.display());
        debug!("    natives_dir:       {}", info.natives_dir.display());
        debug!("    shared_dir:        {}", info.shared_dir.display());
        debug!("    instance_dir:      {}", info.instance_dir.display());

        // Verificar existencia de los directorios clave
        debug!("[3] Verificando directorios críticos:");
        check_dir_exists(&info.lib_dir, "libraries");
        check_dir_exists(&info.assets_dir, "assets");
        check_dir_exists(&info.natives_dir, "natives");

        // Contar JARs en libraries (recursivamente)
        let jar_count = count_jars_recursive(&info.lib_dir);
        debug!("    Total de archivos .jar en libraries: {}", jar_count);
        if jar_count == 0 {
            warn!("    ⚠️  No hay ningún JAR en libraries. El classpath podría quedar vacío.");
        }

        // Crear directorios adicionales (assets/virtual, config)
        Self::prepare_directories(&info)?;

        // ------------------------------------------------------------
        // Obtener mainClass
        // ------------------------------------------------------------
        debug!("[4] Buscando mainClass...");
        let main_class = info
            .get_property("mainClass")
            .ok_or("Main class not found")?;
        debug!("    mainClass: {}", main_class);

        // ------------------------------------------------------------
        // Construir classpath
        // ------------------------------------------------------------
        debug!("[5] Construyendo classpath...");
        let classpath = Self::build_classpath(&info)?;
        if classpath.is_empty() {
            error!("    ❌ Classpath vacío");
            return Err("Classpath is empty".into());
        }
        debug!(
            "    ✅ classpath construido, longitud: {} caracteres",
            classpath.len()
        );

        debug!("[6] Construyendo variables de plantilla...");
        let vars = Self::build_variables(&info, username, game_dir.as_ref());
        for (k, v) in &vars {
            debug!("    {} = {}", k, v);
        }

        // ------------------------------------------------------------
        // Construir comando final
        // ------------------------------------------------------------
        debug!("[7] Construyendo línea de comandos...");
        let command = Self::build_command(
            &info, vars, options, &java_path, min_ram, max_ram, cracked, &classpath, main_class,
            width, height,
        );
        debug!("    Comando construido ({} argumentos):", command.len());
        for (i, arg) in command.iter().enumerate() {
            debug!("      [{}] {}", i, arg);
        }

        // ------------------------------------------------------------
        // Ejecutar el juego
        // ------------------------------------------------------------
        info!("[8] Lanzando proceso del juego...");
        Self::execute_game(command, game_dir.as_ref(), &java_path, custom_env)?;

        info!("========== FIN (ejecución correcta) ==========");
        Ok(())
    }

    // Mantén los otros métodos públicos (launch_with_process, launch_with_dprime)
    // con logs similares si quieres, pero por brevedad no los repito aquí.
    // ...

    // ==================== MÉTODOS AUXILIARES (con logging) ====================

    fn prepare_directories(info: &VersionInfo) -> Result<(), Box<dyn std::error::Error>> {
        let assets_virtual = info.get_assets_virtual_dir();
        debug!(
            "    Creando directorio assets virtual: {}",
            assets_virtual.display()
        );
        fs::create_dir_all(&assets_virtual)?;
        fs::create_dir_all(&info.instance_dir)?;
        Ok(())
    }

    fn build_variables(
        info: &VersionInfo,
        username: &str,
        instance_dir: &Path,
    ) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        let uuid = uuid::Uuid::new_v4().to_string();

        vars.insert("auth_player_name".to_string(), username.to_string());
        vars.insert("version_name".to_string(), info.version_id.clone());
        vars.insert(
            "game_directory".to_string(),
            instance_dir.display().to_string(),
        );
        vars.insert(
            "assets_root".to_string(),
            info.assets_dir.display().to_string(),
        );
        vars.insert(
            "assets_index_name".to_string(),
            info.get_assets_index_name(),
        );
        vars.insert("auth_uuid".to_string(), uuid);
        vars.insert("auth_access_token".to_string(), "0".to_string());
        vars.insert("user_type".to_string(), "mojang".to_string());
        vars.insert("user_properties".to_string(), "{}".to_string());
        vars.insert(
            "version_type".to_string(),
            info.get_property("type").unwrap_or("release").to_string(),
        );

        #[cfg(windows)]
        vars.insert("classpath_separator".to_string(), ";".to_string());
        #[cfg(not(windows))]
        vars.insert("classpath_separator".to_string(), ":".to_string());

        vars.insert(
            "library_directory".to_string(),
            info.lib_dir.display().to_string(),
        );
        vars.insert(
            "natives_directory".to_string(),
            info.natives_dir.display().to_string(),
        );

        vars
    }

    fn build_classpath(info: &VersionInfo) -> Result<String, Box<dyn std::error::Error>> {
        debug!(
            "    Inicializando DependencyResolver con lib_dir = {}",
            info.lib_dir.display()
        );
        let mut resolver = DependencyResolver::new(info.lib_dir.clone(), info.natives_dir.clone());

        // Procesar versión padre (si hay herencia)
        if info.has_inheritance() {
            if let Some(base_data) = &info.base_version_data {
                resolver.process_version(base_data, false);
            }
        }

        resolver.process_version(&info.version_data, true);

        // Construir classpath
        let classpath = resolver.build_classpath(info);
        let count = resolver.library_count();
        debug!("    Se agregaron {} librerías al classpath", count);
        if count == 0 {
            warn!("    ⚠️  No se agregó ninguna librería. Revisa el JSON y las reglas.");
        }
        Ok(classpath)
    }

    fn build_command(
        info: &VersionInfo,
        vars: HashMap<String, String>,
        options: LaunchOptions,
        java_path: impl AsRef<Path>,
        min_ram: &str,
        max_ram: &str,
        cracked: bool,
        classpath: &str,
        main_class: &str,
        width: u32,
        height: u32,
    ) -> Vec<String> {
        let mut builder = CommandBuilder::new(info, vars, options);
        builder
            .add_java(java_path)
            .add_jvm_args(min_ram, max_ram, cracked)
            .add_classpath(classpath)
            .add_main_class(main_class)
            .add_game_args(width, height);

        builder.build()
    }

    fn execute_game(
        command: Vec<String>,
        game_dir: &Path,
        java_path: impl AsRef<Path>,
        custom_env: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let java_home = java_path.as_ref().parent().ok_or("Invalid Java path")?;

        debug!("[9] Comando final a ejecutar:");
        debug!("    {}", command.join(" "));
        debug!("\n    Directorio de trabajo: {}", game_dir.display());
        debug!("    JAVA_HOME: {}", java_home.display());

        let mut cmd = Command::new(&command[0]);
        cmd.args(&command[1..])
            .current_dir(game_dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env("JAVA_HOME", java_home);

        for (key, value) in custom_env {
            debug!("    Variable de entorno adicional: {}={}", key, value);
            cmd.env(key, value);
        }

        debug!("[10] Lanzando proceso...");
        let mut child = cmd.spawn()?;
        let exit_code = child.wait()?;

        if exit_code.success() {
            info!("    ✅ Juego terminado correctamente");
        } else {
            error!("    ❌ ERROR: Código de salida: {:?}", exit_code.code());
        }
        Ok(())
    }
}

fn check_dir_exists(path: &Path, name: &str) {
    if path.exists() {
        debug!("    ✅ {}: {}", name, path.display());
    } else {
        warn!("    ❌ {}: NO EXISTE ({})", name, path.display());
    }
}

fn count_jars_recursive(dir: &Path) -> usize {
    if !dir.exists() {
        return 0;
    }
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jar") {
                count += 1;
            } else if path.is_dir() {
                count += count_jars_recursive(&path);
            }
        }
    }
    count
}
