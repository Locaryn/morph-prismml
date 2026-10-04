//! `locaryn-bonsai-launch` : ce que l'hôte exécute pour démarrer le moteur.
//!
//!   serve --port N --model CHEMIN   installe le runtime au besoin, puis lance llama-server
//!   install                         installe le runtime sans rien lancer
//!   status                          dit ce qui serait utilisé, en JSON

use locaryn_plugin_bonsai::{args, backend, proc, runtime, settings};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("[bonsai] {e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<u8, String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let commande = argv.first().map(String::as_str).unwrap_or("");
    let reglages = settings::Settings::load()?;
    let (os, arch) = (std::env::consts::OS, std::env::consts::ARCH);
    let pilote = backend::nvidia_driver_major();
    let choix = backend::choose(&reglages.backend, os, pilote)?;
    let data = settings::data_dir();
    let tag = reglages.tag().to_string();

    match commande {
        "status" => {
            let dossier = runtime::install_dir(&data, &tag, choix);
            let serveur =
                runtime::find_server(&dossier).filter(|_| dossier.join(".ready").exists());
            let etat = serde_json::json!({
                "runtime_tag": tag,
                "backend": choix.name(),
                "nvidia_driver": pilote,
                "installed": serveur.is_some(),
                "server": serveur,
                "settings": reglages,
            });
            println!("{etat}");
            Ok(0)
        }
        "install" => {
            let serveur = runtime::ensure(&data, &tag, choix, os, arch)?;
            eprintln!("[bonsai] runtime prêt : {}", serveur.display());
            Ok(0)
        }
        "serve" => serve(&argv[1..], &reglages, choix, &data, &tag, os, arch),
        autre => Err(format!(
            "commande « {autre} » inconnue (serve, install, status)"
        )),
    }
}

fn valeur<'a>(argv: &'a [String], drapeau: &str) -> Option<&'a str> {
    argv.iter()
        .position(|a| a == drapeau)
        .and_then(|i| argv.get(i + 1))
        .map(String::as_str)
}

fn serve(
    argv: &[String],
    reglages: &settings::Settings,
    choix: backend::Backend,
    data: &std::path::Path,
    tag: &str,
    os: &str,
    arch: &str,
) -> Result<u8, String> {
    let port: u16 = valeur(argv, "--port")
        .ok_or("--port requis")?
        .parse()
        .map_err(|_| "--port doit être un nombre")?;
    let modele = PathBuf::from(valeur(argv, "--model").ok_or("--model requis")?);
    if !modele.is_file() {
        return Err(format!(
            "modèle introuvable : {} (téléchargez-le depuis le catalogue)",
            modele.display()
        ));
    }
    eprintln!(
        "[bonsai] backend {} · modèle {} · contexte {}",
        choix.name(),
        modele.display(),
        reglages.context_size
    );
    let serveur = runtime::ensure(data, tag, choix, os, arch)?;
    let mmproj = args::find_mmproj(&modele);
    if reglages.vision && mmproj.is_none() {
        eprintln!(
            "[bonsai] vision demandée mais aucun projecteur d'images à côté du modèle : texte seul"
        );
    }
    let ligne = args::build(&modele, port, reglages, mmproj.as_deref());
    let mut enfant = proc::spawn(&serveur, &ligne)?;
    let statut = enfant
        .wait()
        .map_err(|e| format!("attente du serveur : {e}"))?;
    Ok(statut.code().map_or(1, |c| c.clamp(0, 255) as u8))
}
