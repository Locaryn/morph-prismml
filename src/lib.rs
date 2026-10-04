//! Lanceur du runtime Bonsai.
//!
//! Les modèles Bonsai en ternaire (Ternary Bonsai 2) exigent le fork `prism`
//! de llama.cpp : ses noyaux n'existent pas dans le llama.cpp amont. L'hôte ne
//! sait pas installer un binaire, il lance ce que l'extension lui donne : ce
//! lanceur télécharge le runtime épinglé au premier usage (empreinte vérifiée),
//! choisit le backend de la machine, puis démarre `llama-server`.

pub mod args;
pub mod backend;
pub mod proc;
pub mod runtime;
pub mod settings;
