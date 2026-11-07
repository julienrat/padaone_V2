//! Point d'entrée de Padaone.
//!
//! - `padaone` : lance le serveur local (pages `/p`, `/ecran`, `/console`, WebSocket `/ws`)
//!   et affiche les liens à ouvrir, dont ceux de l'écran et de la console avec la clé secrète.
//! - `padaone point-acces …` : diagnostic, démarrage et arrêt du point d'accès Wi-Fi
//!   depuis le terminal, et aide pour les réglages qui demandent les droits administrateur.

mod alea;
mod etat;
mod journal;
mod pages;
mod portail;
mod protocole;
mod reseau;
mod ws;

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use axum::routing::get;
use axum::Router;
use clap::{Args, Parser, Subcommand, ValueEnum};
use padaone_core::Seance;
use padaone_net::diagnostic::{Diagnostic, Droit, Niveau};
use padaone_net::point_acces;
use padaone_net::portail::{
    configuration_dnsmasq, configuration_sysctl, verifier_installation, FICHIER_DNSMASQ,
    FICHIER_SYSCTL,
};
use padaone_net::{
    suggestion, Bande, ChoixAdresse, Demande, Executeur, Securite, Simulation, Systeme,
};

use crate::etat::Partage;
use crate::journal::Journal;
use crate::reseau::EtatReseau;

/// Padaone : les smartphones des participants deviennent des outils de participation.
#[derive(Parser)]
#[command(name = "padaone", version, about)]
struct Options {
    #[command(subcommand)]
    commande: Option<SousCommande>,

    /// Port d'écoute du serveur.
    #[arg(long, default_value_t = 8080)]
    port: u16,

    /// Adresse IP annoncée aux téléphones (par défaut : détectée automatiquement).
    #[arg(long)]
    adresse: Option<IpAddr>,

    /// Titre de la séance, affiché sur les téléphones et à l'écran.
    #[arg(long, default_value = "Séance Padaone")]
    titre: String,

    /// Reprend une séance à partir de son journal (après un plantage, par exemple).
    #[arg(long, value_name = "JOURNAL")]
    reprendre: Option<PathBuf>,

    /// Dossier des journaux de séance.
    #[arg(long, value_name = "DOSSIER")]
    journaux: Option<PathBuf>,

    /// Ne pas écrire de journal (essais).
    #[arg(long, conflicts_with = "reprendre")]
    sans_journal: bool,

    /// Sans portail captif (ni DNS local, ni port 80).
    #[arg(long)]
    sans_portail: bool,

    /// Tire une nouvelle clé secrète pour l'écran et la console (les anciens liens ne marchent plus).
    #[arg(long)]
    nouvelle_cle: bool,

    /// Ouvre la console dans le navigateur une fois le serveur prêt (adresse locale).
    #[arg(long)]
    ouvrir_console: bool,

    /// Simule le réseau : le point d'accès s'essaie sans rien modifier sur le PC.
    #[arg(long, global = true)]
    reseau_simule: bool,
}

#[derive(Subcommand)]
enum SousCommande {
    /// Point d'accès Wi-Fi : diagnostic, démarrage, arrêt, droits administrateur.
    #[command(subcommand)]
    PointAcces(PointAcces),
}

#[derive(Subcommand)]
enum PointAcces {
    /// Cartes Wi-Fi, bandes, capacité et droits (lecture seule, ne modifie rien).
    Diag,
    /// Démarre le point d'accès (modifie le réseau : coupe le Wi-Fi actuel de la carte choisie).
    Demarrer(ArgumentsDemarrer),
    /// Arrête le point d'accès et rallume éventuellement une connexion.
    Arreter {
        /// Connexion à rallumer ensuite, par exemple le Wi-Fi de la salle.
        #[arg(long, value_name = "CONNEXION")]
        revenir: Option<String>,
    },
    /// Explique les droits nécessaires et comment les accorder (ne modifie rien).
    Droits,
    /// Portail captif et padaone.lan : état et commandes sudo à copier (ne modifie rien).
    #[command(alias = "dns")]
    Portail,
}

#[derive(Args)]
struct ArgumentsDemarrer {
    /// Carte Wi-Fi (par défaut : la carte conseillée par le diagnostic).
    #[arg(long)]
    carte: Option<String>,
    /// Nom du réseau (par défaut : un nom lisible, « Padaone-Tilleul »).
    #[arg(long)]
    nom: Option<String>,
    /// Mot de passe WPA2 de 8 à 63 caractères (par défaut : un mot de passe lisible).
    #[arg(long, conflicts_with = "ouvert")]
    mot_de_passe: Option<String>,
    /// Réseau ouvert, sans mot de passe.
    #[arg(long)]
    ouvert: bool,
    #[arg(long, value_enum, default_value_t = BandeCli::Auto)]
    bande: BandeCli,
    #[arg(long)]
    canal: Option<u32>,
    /// Adresse du PC : « garder » (l'actuelle), « defaut » (10.42.0.1) ou une adresse avec masque.
    #[arg(long, default_value = "garder")]
    adresse_ap: String,
}

#[derive(Clone, Copy, ValueEnum)]
enum BandeCli {
    #[value(name = "2.4")]
    Ghz2_4,
    #[value(name = "5")]
    Ghz5,
    Auto,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "padaone=info".into()),
        )
        .init();
    let options = Options::parse();
    let executeur: Arc<dyn Executeur> =
        if options.reseau_simule { Arc::new(Simulation::default()) } else { Arc::new(Systeme) };

    match &options.commande {
        Some(SousCommande::PointAcces(action)) => point_acces_cli(action, executeur.as_ref()),
        None => servir(&options, executeur).await,
    }
}

async fn servir(options: &Options, executeur: Arc<dyn Executeur>) -> Result<()> {
    let (seance, journal) = preparer_seance(options)?;
    let ip = options.adresse.unwrap_or_else(adresse_locale);
    let reseau =
        EtatReseau::new(&ip.to_string(), options.port, seance.code(), options.reseau_simule)?;
    let lien = reseau.info.lien.clone();
    // Clé gardée d'un lancement à l'autre : les pages ouvertes se reconnectent après une relance.
    let cle = match alea::fichier_cle() {
        Some(f) => alea::cle_persistante(&f, options.nouvelle_cle).with_context(|| {
            format!("clé secrète impossible à lire ou écrire dans {}", f.display())
        })?,
        None => alea::hexa()?,
    };
    let hote = format!("{ip}:{}", options.port);

    println!();
    println!("  Padaone {} — « {} »", env!("CARGO_PKG_VERSION"), seance.titre());
    println!("  Code de la séance : {}", code_lisible(seance.code()));
    println!();
    println!("  Participants : {lien}");
    println!("  Écran        : http://{hote}/ecran?cle={cle}");
    println!("  Console      : http://localhost:{}/console?cle={cle}", options.port);
    match &journal {
        Some(j) => println!("  Journal      : {}", j.chemin().display()),
        None => println!("  Journal      : désactivé"),
    }
    if options.reseau_simule {
        println!("  Réseau       : simulé (le point d'accès ne modifie rien)");
    }
    println!();
    println!("  Les liens de l'écran et de la console sont secrets : ne les projetez pas.");
    println!("  Ils restent les mêmes d'un lancement à l'autre ; --nouvelle-cle pour en changer.");
    println!("  Ctrl+C pour arrêter.");
    println!();

    let partage = Partage::new(seance, journal, cle, reseau, executeur);
    let app = Router::new()
        .route("/ws", get(ws::route))
        .fallback(pages::servir)
        .with_state(Arc::clone(&partage));
    if !options.sans_portail {
        tokio::spawn(portail::servir_dns(Arc::clone(&partage)));
        if options.port != portail::PORT_HTTP {
            tokio::spawn(portail::ecouter_port_80(Arc::clone(&partage), app.clone()));
        }
    }

    let ecoute = SocketAddr::from((Ipv4Addr::UNSPECIFIED, options.port));
    let auditeur = tokio::net::TcpListener::bind(ecoute)
        .await
        .with_context(|| format!("le port {} est déjà utilisé ou interdit", options.port))?;
    if options.ouvrir_console {
        ouvrir_navigateur(&format!(
            "http://localhost:{}/console?cle={}",
            options.port, partage.cle
        ));
    }
    tokio::select! {
        r = axum::serve(auditeur, app) => r?,
        _ = tokio::signal::ctrl_c() => {
            println!();
            reseau::arret_fin_de_seance(partage).await;
            println!("  Au revoir !");
        }
    }
    Ok(())
}

/// Ouvre une nouvelle séance ou reprend celle d'un journal.
fn preparer_seance(options: &Options) -> Result<(Seance, Option<Journal>)> {
    if let Some(chemin) = &options.reprendre {
        let evenements = Journal::lire(chemin)?;
        let seance = Seance::rejouer(&evenements)
            .with_context(|| format!("journal incohérent : {}", chemin.display()))?;
        tracing::info!("séance reprise : {} événements relus", evenements.len());
        return Ok((seance, Some(Journal::rouvrir(chemin)?)));
    }

    let (seance, ouverture) = Seance::ouvrir(&options.titre, &alea::code_seance()?)
        .context("titre de séance invalide (vide ou plus de 80 caractères)")?;
    if options.sans_journal {
        return Ok((seance, None));
    }
    let dossier = match options.journaux.clone().or_else(Journal::dossier_par_defaut) {
        Some(d) => d,
        None => bail!("dossier des journaux introuvable : précisez --journaux"),
    };
    let mut journal = Journal::creer(&dossier, seance.code())?;
    journal.ecrire(&ouverture)?;
    Ok((seance, Some(journal)))
}

// --- Sous-commandes point-acces ------------------------------------------------------

fn point_acces_cli(action: &PointAcces, ex: &dyn Executeur) -> Result<()> {
    match action {
        PointAcces::Diag => {
            let d = point_acces::diagnostiquer(ex)?;
            afficher_diagnostic(&d);
        }
        PointAcces::Demarrer(a) => {
            let h = getrandom::u32().unwrap_or(0);
            let securite = if a.ouvert {
                Securite::Ouvert
            } else {
                Securite::Wpa2 {
                    mot_de_passe: a
                        .mot_de_passe
                        .clone()
                        .unwrap_or_else(|| suggestion::mot_de_passe(h)),
                }
            };
            let adresse = match a.adresse_ap.as_str() {
                "garder" => ChoixAdresse::Garder,
                "defaut" => ChoixAdresse::ParDefaut,
                autre => ChoixAdresse::Fixe(autre.to_string()),
            };
            let demande = Demande {
                carte: a.carte.clone(),
                nom_reseau: a.nom.clone().unwrap_or_else(|| suggestion::nom_reseau(h)),
                securite,
                bande: match a.bande {
                    BandeCli::Ghz2_4 => Bande::Ghz2_4,
                    BandeCli::Ghz5 => Bande::Ghz5,
                    BandeCli::Auto => Bande::Auto,
                },
                canal: a.canal,
                adresse,
            };
            println!("Démarrage du point d'accès « {} »…", demande.nom_reseau);
            let pa = point_acces::demarrer(ex, &demande)?;
            println!("Point d'accès actif sur {}.", pa.carte);
            println!("  Réseau       : {}", pa.nom_reseau);
            match &pa.securite {
                Securite::Wpa2 { mot_de_passe } => println!("  Mot de passe : {mot_de_passe}"),
                Securite::Ouvert => println!("  Réseau ouvert, sans mot de passe"),
            }
            println!("  Adresse du PC : {}", pa.adresse);
            match &pa.connexion_precedente {
                Some(p) => println!(
                    "Pour arrêter et revenir au Wi-Fi précédent : padaone point-acces arreter --revenir \"{p}\""
                ),
                None => println!("Pour arrêter : padaone point-acces arreter"),
            }
        }
        PointAcces::Arreter { revenir } => match point_acces::arreter(ex, revenir.as_deref())? {
            None => println!("Point d'accès arrêté."),
            Some(avertissement) => println!("Point d'accès arrêté. {avertissement}"),
        },
        PointAcces::Droits => afficher_droits(&point_acces::diagnostiquer(ex)?),
        PointAcces::Portail => {
            let i = verifier_installation(ex);
            let etat = |ok: bool| if ok { "installé" } else { "à installer" };
            println!("Portail captif : les téléphones ouvrent la séance au lieu d'afficher « sans internet »,");
            println!(
                "et padaone.lan désigne le PC. Deux réglages, à faire une seule fois avec sudo."
            );
            println!("Padaone ne les fait pas lui-même : copiez les commandes dans un terminal.");
            println!();
            println!(
                "1. DNS du point d'accès ({}) : les noms sont résolus par Padaone.",
                etat(i.dns)
            );
            println!("   Vaut pour tous les partages de connexion de NetworkManager.");
            println!();
            println!("sudo mkdir -p /etc/NetworkManager/dnsmasq-shared.d");
            print!(
                "sudo tee {FICHIER_DNSMASQ} > /dev/null <<'FIN'\n{}FIN\n",
                configuration_dnsmasq()
            );
            println!();
            println!("2. Port 80 sans droits administrateur ({}) :", etat(i.port_80));
            println!();
            print!(
                "sudo tee {FICHIER_SYSCTL} > /dev/null <<'FIN'\n{}FIN\n",
                configuration_sysctl()
            );
            println!("sudo sysctl --system > /dev/null");
            println!();
            println!("Effet : au prochain démarrage du point d'accès pour le DNS ; dans les 15 s pour le port 80.");
            println!("Pour tout retirer : sudo rm {FICHIER_DNSMASQ} {FICHIER_SYSCTL}");
        }
    }
    Ok(())
}

fn afficher_diagnostic(d: &Diagnostic) {
    println!("Cartes Wi-Fi");
    if d.cartes.is_empty() {
        println!("  (aucune)");
    }
    for c in &d.cartes {
        println!("  {} · pilote {} · {}", c.nom, c.pilote.as_deref().unwrap_or("?"), c.etat);
        println!(
            "    point d'accès : {} · 2,4 GHz : {} · 5 GHz : {}",
            oui_non(c.point_acces),
            oui_non(c.ghz2),
            oui_non(c.ghz5)
        );
        if let Some(cx) = &c.connexion {
            println!("    connexion active : {cx}");
        }
        if let Some(a) = &c.adresse {
            println!("    adresse : {a}");
        }
        println!("    capacité estimée : {}", c.capacite);
    }
    println!();
    println!("Constats");
    for k in &d.constats {
        let marque = match k.niveau {
            Niveau::Bon => "✓",
            Niveau::Attention => "!",
            Niveau::Bloquant => "✗",
        };
        println!("  {marque} {}", k.texte);
    }
    if let Some(c) = d.carte_conseillee() {
        println!();
        println!("Carte conseillée : {}", c.nom);
    }
}

fn afficher_droits(d: &Diagnostic) {
    let texte = |droit: Droit| match droit {
        Droit::Oui => "accordé sans mot de passe",
        Droit::Authentification => "accordé après le mot de passe administrateur",
        Droit::Non => "refusé",
        Droit::Inconnu => "inconnu",
    };
    println!("Droits NetworkManager (polkit) de l'utilisateur courant");
    println!("  piloter le réseau      : {}", texte(d.droits.controle_reseau));
    println!("  partager en Wi-Fi WPA  : {}", texte(d.droits.partage_protege));
    println!("  partager en Wi-Fi ouvert : {}", texte(d.droits.partage_ouvert));
    println!("  enregistrer le profil  : {}", texte(d.droits.modifier_systeme));
    println!();
    let droits = [d.droits.controle_reseau, d.droits.partage_protege, d.droits.modifier_systeme];
    if droits.iter().all(|x| *x == Droit::Oui) {
        println!(
            "Tout est en ordre : Padaone pilote le point d'accès sans demander de mot de passe."
        );
    } else if droits.contains(&Droit::Non) {
        println!("Un droit est refusé. Sur un PC personnel, l'utilisateur de la session ouverte");
        println!(
            "les a d'habitude. Sinon, demandez à l'administrateur du poste d'ajouter votre compte"
        );
        println!("au groupe « netdev » ou d'autoriser NetworkManager pour vous (règle polkit).");
    } else {
        println!(
            "Le système demandera le mot de passe administrateur au démarrage du point d'accès."
        );
        println!("C'est normal sur un poste partagé ; Padaone ne stocke jamais ce mot de passe.");
    }
}

fn oui_non(b: bool) -> &'static str {
    if b {
        "oui"
    } else {
        "non"
    }
}

/// Adresse IP du PC sur le réseau local.
///
/// « Connecter » une socket UDP n'envoie aucun paquet : le système choisit seulement
/// l'interface de sortie, dont on lit l'adresse. Sans réseau, on se rabat sur 127.0.0.1.
fn adresse_locale() -> IpAddr {
    let essai = || -> std::io::Result<IpAddr> {
        let s = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
        s.connect((Ipv4Addr::new(10, 255, 255, 255), 9))?;
        Ok(s.local_addr()?.ip())
    };
    match essai() {
        Ok(ip) if !ip.is_unspecified() => ip,
        _ => {
            tracing::warn!("adresse du réseau local introuvable : utilisez --adresse <IP>");
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        }
    }
}

/// Ouvre une adresse locale dans le navigateur par défaut (`xdg-open`), sans attendre.
fn ouvrir_navigateur(adresse: &str) {
    let lancement = std::process::Command::new("xdg-open")
        .arg(adresse)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    if lancement.is_err() {
        println!("  Navigateur introuvable : ouvrez vous-même le lien de la console ci-dessus.");
    }
}

/// « 482731 » devient « 482 731 ».
fn code_lisible(code: &str) -> String {
    if code.len() == 6 && code.is_ascii() {
        format!("{} {}", &code[..3], &code[3..])
    } else {
        code.to_string()
    }
}
