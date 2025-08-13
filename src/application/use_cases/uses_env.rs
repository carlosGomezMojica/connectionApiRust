use std::fs;
use std::path::{Path, PathBuf};

pub fn validar_env() -> bool {
    let env_path = ".env";

    // 1) Verificar que el archivo exista
    if !Path::new(env_path).exists() {
        return false;
    }

    // 2) Leer el contenido (si hay error de lectura, retorna false)
    let content = match fs::read_to_string(env_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    // 3) Variables de control
    let mut api_key_ok = false;
    let mut synthetic_index_ok = false;

    // 4) Buscar las líneas con datos
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("api_key=") {
            if !value.trim().is_empty() {
                api_key_ok = true;
            }
        }
        if let Some(value) = line.strip_prefix("synthetic_index=") {
            if !value.trim().is_empty() {
                synthetic_index_ok = true;
            }
        }
    }
    // 5) Retornar true solo si ambas variables están presentes y con valor
    api_key_ok && synthetic_index_ok
}

pub fn upsert_env_api_keys(api_key: &str, synthetic_index: &str) -> Result<(), String> {
    // Siempre en la raíz del proyecto (donde está Cargo.toml)
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(".env");

    // Si no existe, créalo con ambas variables
    if !path.exists() {
        let content = format!("api_key={}\nsynthetic_index={}\n", api_key, synthetic_index);
        fs::write(&path, content).map_err(|e| format!("No pude crear {:?}: {}", path, e))?;
        return Ok(());
    }

    // Si existe, lee su contenido
    let original =
        fs::read_to_string(&path).map_err(|e| format!("No pude leer {:?}: {}", path, e))?;

    let mut had_api_key = false;
    let mut had_synth = false;
    let mut lines: Vec<String> = Vec::with_capacity(64);

    for line in original.lines() {
        let trimmed = line.trim_start(); // preserva indentación a la izquierda si la hubiera
        // ignora comentarios
        if trimmed.starts_with('#') {
            lines.push(line.to_string());
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("api_key=") {
            had_api_key = true;
            // Si está vacío -> escribe el valor; si no, deja tal cual
            if rest.trim().is_empty() {
                // reconstruye respetando el posible espacio a la izquierda
                let indent_len = line.len() - trimmed.len();
                let indent = &line[..indent_len];
                lines.push(format!("{}api_key={}", indent, api_key));
            } else {
                lines.push(line.to_string());
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("synthetic_index=") {
            had_synth = true;
            if rest.trim().is_empty() {
                let indent_len = line.len() - trimmed.len();
                let indent = &line[..indent_len];
                lines.push(format!("{}synthetic_index={}", indent, synthetic_index));
            } else {
                lines.push(line.to_string());
            }
            continue;
        }

        // Cualquier otra línea, se preserva
        lines.push(line.to_string());
    }

    // Si no existían, agrégalas al final
    if !had_api_key {
        lines.push(format!("api_key={}", api_key));
    }
    if !had_synth {
        lines.push(format!("synthetic_index={}", synthetic_index));
    }

    // Si no hicimos ningún cambio, no reescribas el archivo
    let new_content = {
        let mut s = String::with_capacity(original.len().max(64));
        for (i, l) in lines.iter().enumerate() {
            if i > 0 {
                s.push('\n');
            }
            s.push_str(l);
        }
        // Asegura newline final (convención típica de .env)
        if !s.ends_with('\n') {
            s.push('\n');
        }
        s
    };

    if new_content != original {
        fs::write(&path, new_content).map_err(|e| format!("No pude escribir {:?}: {}", path, e))?;
    }

    Ok(())
}
