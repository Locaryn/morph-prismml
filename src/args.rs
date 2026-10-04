//! La ligne de commande de `llama-server`.

use crate::settings::Settings;
use std::path::{Path, PathBuf};

/// Le projecteur d'images posé à côté du modèle, s'il existe.
///
/// Les dépôts Bonsai le nomment `<famille>-mmproj-<quant>.gguf`, à côté de
/// `<famille>-<quant>.gguf`. On prend le plus léger quand il y en a plusieurs.
pub fn find_mmproj(modele: &Path) -> Option<PathBuf> {
    let dossier = modele.parent()?;
    let nom = modele.file_stem()?.to_string_lossy().to_string();
    // « Bonsai-27B-Q1_0 » → famille « Bonsai-27B ».
    let famille = nom
        .rsplit_once('-')
        .map_or(nom.as_str(), |(f, _)| f)
        .to_lowercase();
    let mut trouves: Vec<(u64, PathBuf)> = std::fs::read_dir(dossier)
        .ok()?
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().to_lowercase();
            (n.contains("mmproj") && n.ends_with(".gguf") && n.starts_with(&famille))
                .then(|| (e.metadata().map_or(u64::MAX, |m| m.len()), e.path()))
        })
        .collect();
    trouves.sort();
    trouves.into_iter().next().map(|(_, p)| p)
}

/// Les arguments, sans l'exécutable.
pub fn build(modele: &Path, port: u16, s: &Settings, mmproj: Option<&Path>) -> Vec<String> {
    let mut a: Vec<String> = vec![
        "-m".into(),
        modele.to_string_lossy().into(),
        "--host".into(),
        "127.0.0.1".into(),
        "--port".into(),
        port.to_string(),
        "-c".into(),
        s.context_size.to_string(),
        "-ngl".into(),
        s.gpu_layers.to_string(),
        "--parallel".into(),
        s.parallel.max(1).to_string(),
        // Les appels d'outils d'une conversation passent par le gabarit de chat.
        "--jinja".into(),
    ];
    if matches!(s.thinking.as_str(), "on" | "off" | "auto") {
        a.push("--reasoning".into());
        a.push(s.thinking.clone());
    }
    if s.vision {
        if let Some(p) = mmproj {
            a.push("--mmproj".into());
            a.push(p.to_string_lossy().into());
        }
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_ligne_de_commande_tient_sur_une_petite_carte() {
        let a = build(Path::new("m.gguf"), 8189, &Settings::default(), None);
        let joint = a.join(" ");
        assert!(
            joint.contains("--host 127.0.0.1"),
            "jamais exposé au réseau"
        );
        assert!(joint.contains("-c 8192"));
        assert!(joint.contains("--parallel 1"));
        assert!(joint.contains("--jinja"));
        assert!(joint.contains("--reasoning off"));
        assert!(!joint.contains("--mmproj"));
    }

    #[test]
    fn le_projecteur_n_est_charge_que_sur_demande() {
        let mut s = Settings::default();
        let p = Path::new("m-mmproj.gguf");
        assert!(!build(Path::new("m.gguf"), 1, &s, Some(p)).contains(&"--mmproj".to_string()));
        s.vision = true;
        assert!(build(Path::new("m.gguf"), 1, &s, Some(p)).contains(&"--mmproj".to_string()));
    }

    #[test]
    fn le_projecteur_se_trouve_a_cote_du_modele() {
        let d = std::env::temp_dir().join(format!("bonsai-mm-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        for f in [
            "Bonsai-27B-Q1_0.gguf",
            "Bonsai-27B-mmproj-BF16.gguf",
            "Bonsai-27B-mmproj-Q8_0.gguf",
            "Ternary-Bonsai-27B-mmproj-Q8_0.gguf",
        ] {
            std::fs::write(
                d.join(f),
                vec![0u8; if f.contains("BF16") { 20 } else { 10 }],
            )
            .unwrap();
        }
        let trouve = find_mmproj(&d.join("Bonsai-27B-Q1_0.gguf")).unwrap();
        assert_eq!(trouve.file_name().unwrap(), "Bonsai-27B-mmproj-Q8_0.gguf");
        assert!(find_mmproj(&d.join("Autre-Q4.gguf")).is_none());
        std::fs::remove_dir_all(&d).unwrap();
    }
}
