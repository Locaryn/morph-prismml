//! Quel build du runtime convient à cette machine.

use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Cuda,
    Vulkan,
    Cpu,
    /// macOS : Metal est dans le build unique.
    Metal,
}

impl Backend {
    pub fn name(self) -> &'static str {
        match self {
            Backend::Cuda => "cuda",
            Backend::Vulkan => "vulkan",
            Backend::Cpu => "cpu",
            Backend::Metal => "metal",
        }
    }
}

/// Pilote NVIDIA minimal pour le build CUDA 12.4.
const PILOTE_CUDA_MIN: u32 = 551;

/// Version majeure du pilote NVIDIA, si une carte répond.
pub fn nvidia_driver_major() -> Option<u32> {
    let out = Command::new("nvidia-smi")
        .args(["--query-gpu=driver_version", "--format=csv,noheader"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    major_du_pilote(&String::from_utf8_lossy(&out.stdout))
}

fn major_du_pilote(sortie: &str) -> Option<u32> {
    sortie
        .lines()
        .next()?
        .trim()
        .split('.')
        .next()?
        .parse()
        .ok()
}

/// Le backend à utiliser. `demande` vient des réglages (`auto` par défaut).
pub fn choose(demande: &str, os: &str, pilote_nvidia: Option<u32>) -> Result<Backend, String> {
    if os == "macos" {
        return Ok(Backend::Metal);
    }
    match demande.trim() {
        "" | "auto" => Ok(match pilote_nvidia {
            Some(v) if v >= PILOTE_CUDA_MIN => Backend::Cuda,
            // Sans NVIDIA exploitable : Vulkan sert les cartes AMD et Intel, et
            // retombe sur le processeur quand aucune carte ne répond.
            _ => Backend::Vulkan,
        }),
        "cuda" => match pilote_nvidia {
            Some(v) if v >= PILOTE_CUDA_MIN => Ok(Backend::Cuda),
            Some(v) => Err(format!(
                "pilote NVIDIA {v} trop ancien pour CUDA 12.4 (il faut {PILOTE_CUDA_MIN} ou plus) : mettez-le à jour, ou choisissez « vulkan »."
            )),
            None => Err("aucune carte NVIDIA ne répond à nvidia-smi : choisissez « vulkan » ou « cpu ».".into()),
        },
        "vulkan" => Ok(Backend::Vulkan),
        "cpu" => Ok(Backend::Cpu),
        autre => Err(format!(
            "backend « {autre} » inconnu (auto, cuda, vulkan, cpu)"
        )),
    }
}

/// Les archives à télécharger, dans l'ordre. `tag` est celui du dépôt, il
/// porte déjà le préfixe `prism-`.
pub fn assets(tag: &str, backend: Backend, os: &str, arch: &str) -> Result<Vec<String>, String> {
    let sans_support = || {
        format!(
            "pas de runtime Bonsai publié pour {os} {arch} avec le backend {}",
            backend.name()
        )
    };
    let x64 = arch == "x86_64";
    let arm = arch == "aarch64";
    let noms = match (os, backend) {
        ("windows", Backend::Cuda) if x64 => vec![
            format!("llama-{tag}-bin-win-cuda-12.4-x64.zip"),
            "cudart-llama-bin-win-cuda-12.4-x64.zip".to_string(),
        ],
        ("windows", Backend::Vulkan) if x64 => vec![format!("llama-{tag}-bin-win-vulkan-x64.zip")],
        ("windows", Backend::Cpu) if x64 => vec![format!("llama-{tag}-bin-win-cpu-x64.zip")],
        ("windows", Backend::Cpu) if arm => vec![format!("llama-{tag}-bin-win-cpu-arm64.zip")],
        ("linux", Backend::Cuda) if x64 => {
            vec![format!("llama-{tag}-bin-linux-cuda-12.4-x64.tar.gz")]
        }
        ("linux", Backend::Vulkan) if x64 => {
            vec![format!("llama-{tag}-bin-ubuntu-vulkan-x64.tar.gz")]
        }
        ("linux", Backend::Cpu) if x64 => vec![format!("llama-{tag}-bin-ubuntu-x64.tar.gz")],
        ("linux", Backend::Cpu | Backend::Vulkan) if arm => {
            vec![format!("llama-{tag}-bin-ubuntu-arm64.tar.gz")]
        }
        ("macos", Backend::Metal) if arm => vec![format!("llama-{tag}-bin-macos-arm64.tar.gz")],
        ("macos", Backend::Metal) if x64 => vec![format!("llama-{tag}-bin-macos-x64.tar.gz")],
        _ => return Err(sans_support()),
    };
    Ok(noms)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAG: &str = "prism-b10754-2459f68";

    #[test]
    fn le_pilote_se_lit() {
        assert_eq!(major_du_pilote("616.56\n"), Some(616));
        assert_eq!(major_du_pilote("550.54.14\n550.54.14\n"), Some(550));
        assert_eq!(major_du_pilote(""), None);
        assert_eq!(major_du_pilote("N/A"), None);
    }

    #[test]
    fn nvidia_recent_donne_cuda_sinon_vulkan() {
        assert_eq!(choose("auto", "windows", Some(616)), Ok(Backend::Cuda));
        assert_eq!(choose("auto", "windows", Some(535)), Ok(Backend::Vulkan));
        assert_eq!(choose("auto", "linux", None), Ok(Backend::Vulkan));
        assert_eq!(choose("auto", "macos", None), Ok(Backend::Metal));
    }

    #[test]
    fn un_choix_impossible_est_dit_pas_ignore() {
        assert!(choose("cuda", "windows", None)
            .unwrap_err()
            .contains("nvidia-smi"));
        assert!(choose("cuda", "windows", Some(535))
            .unwrap_err()
            .contains("trop ancien"));
        assert!(choose("rocm", "linux", None)
            .unwrap_err()
            .contains("inconnu"));
    }

    #[test]
    fn windows_cuda_demande_aussi_les_bibliotheques_cuda() {
        let a = assets(TAG, Backend::Cuda, "windows", "x86_64").unwrap();
        assert_eq!(a[0], "llama-prism-b10754-2459f68-bin-win-cuda-12.4-x64.zip");
        assert_eq!(a[1], "cudart-llama-bin-win-cuda-12.4-x64.zip");
    }

    #[test]
    fn une_plateforme_sans_build_le_dit() {
        let e = assets(TAG, Backend::Cuda, "macos", "x86_64").unwrap_err();
        assert!(e.contains("macos"), "{e}");
    }
}
