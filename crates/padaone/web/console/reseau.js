// Page « Point d'accès » de la console : diagnostic, réglages, démarrage et arrêt.
// Rien ne modifie le réseau sans un clic sur « Démarrer » ou « Arrêter ».

import { connecter } from "/commun/connexion.js";
import { el, pluriel } from "/commun/outils.js";

const $ = (id) => document.getElementById(id);
const cle = new URLSearchParams(location.search).get("cle") || "";
$("retour").href = `/console?cle=${encodeURIComponent(cle)}`;

const CANAUX = { ghz2_4: [1, 6, 11], ghz5: [36, 40, 44, 48] };

/** Réglages choisis dans le formulaire (gardés entre deux mises à jour). */
const choix = { carte: null, securite: "wpa2", bande: "auto", canal: "", adresse: "garder" };
let pa = null; // état du point d'accès (champ `point_acces` de la vue console)
let reseau = null; // adresses et Wi-Fi (message `reseau`)
let dernierDiagnostic = "";
let suggestionAppliquee = null;
let attendSuggestion = false;

const lien = connecter({
  bonjour: () => ({ role: "console", cle }),
  surConnexion: (ok) => {
    if (!ok) {
      $("principal-ap").disabled = true;
      afficherMessage({ texte: "Connexion au serveur perdue, on réessaie…", erreur: true });
    }
  },
  surErreurFatale: (message) => {
    $("principal-ap").disabled = true;
    afficherMessage({ texte: `${message} Rouvrez la console avec le lien affiché dans le terminal de Padaone.`, erreur: true });
  },
  surMessage: (m) => {
    if (m.t === "bienvenue" || m.t === "reseau") {
      reseau = m.t === "reseau" ? m : m.reseau;
      afficherApercu();
    } else if (m.t === "etat" && m.point_acces) {
      pa = m.point_acces;
      afficher();
    } else if (m.t === "erreur") {
      afficherMessage({ texte: m.message, erreur: true });
    }
  },
});

function commande(c, parametres = {}) {
  if (!lien.envoyer({ t: "commande", c, ...parametres })) {
    afficherMessage({ texte: "Pas de connexion au serveur : commande non envoyée.", erreur: true });
  }
}

function afficherMessage(m) {
  $("message").hidden = !m;
  if (!m) return;
  $("message").textContent = m.texte;
  $("message").classList.toggle("erreur", m.erreur);
}

// --- Affichage de l'état ------------------------------------------------------------

function carteChoisie() {
  return pa?.diagnostic?.cartes.find((c) => c.nom === choix.carte) ?? null;
}

function afficher() {
  $("simule").hidden = !pa.simule;
  const actif = pa.actif;
  const etranger = !actif && pa.diagnostic?.point_acces_actif; // laissé par un essai précédent
  let statut;
  if (pa.en_cours) statut = ["en-cours", pa.en_cours];
  else if (actif) statut = ["actif", `● Actif sur ${actif.carte}`];
  else if (etranger) statut = ["actif", `● Actif sur ${etranger} (démarré hors de cette séance)`];
  else statut = ["arrete", "○ Arrêté"];
  $("statut").className = `statut-ap ${statut[0]}`;
  $("statut").textContent = statut[1];

  const bouton = $("principal-ap");
  bouton.disabled = Boolean(pa.en_cours) || (!actif && !etranger && !pa.diagnostic);
  bouton.textContent = actif || etranger ? "Arrêter le point d'accès" : "Démarrer le point d'accès";
  bouton.classList.toggle("encre", Boolean(actif || etranger));
  bouton.classList.toggle("violet", !(actif || etranger));
  $("rediagnostiquer").disabled = Boolean(pa.en_cours);

  if (pa.message) afficherMessage(pa.message);
  else if (pa.en_cours) afficherMessage({ texte: pa.en_cours, erreur: false });
  else afficherMessage(null);

  $("arret-fin").checked = pa.arret_fin_seance;
  if (document.activeElement !== $("portail")) $("portail").value = pa.portail;
  const installe = pa.diagnostic?.portail;
  const manque = [];
  if (installe && !installe.dns) manque.push("le réglage DNS");
  if (!pa.portail_port_80) manque.push("le port 80");
  if (!pa.portail_dns && !pa.simule) manque.push("le DNS local de Padaone");
  $("aide-portail").textContent =
    pa.portail === "desactive"
      ? "Aucune réponse aux tests de connexion des téléphones."
      : manque.length
        ? `Inactif pour l'instant : il manque ${manque.join(", ")}. Voir « padaone point-acces portail » ci-contre.`
        : "Actif : les téléphones qui rejoignent le Wi-Fi sont dirigés vers la séance.";

  // Suggestion de nom et de mot de passe : au premier affichage et après « Générer ».
  const s = JSON.stringify(pa.suggestion);
  if (suggestionAppliquee === null || (attendSuggestion && s !== suggestionAppliquee)) {
    $("nom").value = pa.suggestion.nom_reseau;
    $("mdp").value = pa.suggestion.mot_de_passe;
    suggestionAppliquee = s;
    attendSuggestion = false;
  }

  const diag = JSON.stringify(pa.diagnostic);
  if (pa.diagnostic && diag !== dernierDiagnostic) {
    dernierDiagnostic = diag;
    afficherCartes();
    afficherConstats();
  }

  // Pendant que le point d'accès tourne, les réglages affichés sont ceux en service.
  if (actif) {
    $("nom").value = actif.nom_reseau;
    choix.securite = actif.securite.type;
    if (actif.securite.type === "wpa2") $("mdp").value = actif.securite.mot_de_passe;
    choix.carte = actif.carte;
  }
  for (const champ of $("reglages").querySelectorAll("input, select, button")) {
    champ.disabled = Boolean(actif || pa.en_cours) && champ.id !== "arret-fin" && champ.id !== "portail";
  }
  if (!actif && !pa.en_cours) desactiverIncompatibles();

  $("actif").hidden = !actif;
  if (actif) {
    const n = pa.appareils ?? 0;
    $("nb-appareils").textContent = String(n);
    $("lib-appareils").textContent = `${n > 1 ? "appareils" : "appareil"} sur le réseau Padaone`;
    $("adresse-actif").textContent = `Adresse du PC : ${actif.adresse}. Participants : ${reseau?.adresse ?? ""}`;
  }
  const carte = carteChoisie();
  const ip = carte?.adresse?.split("/")[0];
  $("lib-garder").replaceChildren(
    ...(ip
      ? [el("strong", {}, `Garder l'adresse actuelle (${ip})`), " · bascule sans coupure : les pages ouvertes se reconnectent seules"]
      : [
          el("strong", {}, "Garder l'adresse actuelle"),
          " · cette carte n'est pas connectée : ce sera 10.42.0.1, et le PC reste joignable par l'autre réseau",
        ]),
  );
  afficherBascules();
  afficherApercu();
}

function afficherCartes() {
  const cartes = pa.diagnostic.cartes;
  if (!cartes.length) {
    $("cartes").replaceChildren(el("p", { class: "petit" }, "Aucune carte Wi-Fi détectée. Branchez une clé USB Wi-Fi puis relancez le diagnostic."));
    return;
  }
  if (!cartes.some((c) => c.nom === choix.carte && c.point_acces)) {
    const libre = cartes.find((c) => c.point_acces && !c.connexion);
    choix.carte = (libre ?? cartes.find((c) => c.point_acces))?.nom ?? null;
  }
  $("cartes").replaceChildren(
    ...cartes.map((c) => {
      const bandes = [c.ghz2 && "2,4 GHz", c.ghz5 && "5 GHz"].filter(Boolean).join(" et ") || "bande inconnue";
      const detail = [c.pilote ?? "pilote inconnu", bandes, c.connexion ? `connectée à « ${c.connexion} »` : "libre"].join(" · ");
      return el(
        "label",
        { class: `carte-wifi${c.point_acces ? "" : " incapable"}` },
        el("input", {
          type: "radio",
          name: "carte",
          value: c.nom,
          checked: c.nom === choix.carte,
          disabled: !c.point_acces,
          onchange: () => {
            choix.carte = c.nom;
            afficher();
          },
        }),
        el(
          "span",
          { class: "infos-carte" },
          el("strong", {}, c.nom),
          el("span", { class: "petit" }, detail),
          el("span", { class: "petit" }, c.point_acces ? `Capacité : ${c.capacite}` : "Ne sait pas faire point d'accès."),
        ),
      );
    }),
  );
}

function afficherConstats() {
  const marques = { bon: "✓", attention: "!", bloquant: "✗" };
  $("constats").replaceChildren(
    ...pa.diagnostic.constats.map((k) =>
      el(
        "li",
        { class: `constat ${k.niveau}` },
        el("span", { class: "marque-constat", "aria-hidden": "true" }, marques[k.niveau]),
        el("span", {}, k.texte),
      ),
    ),
  );
}

/** Boutons de sécurité et de bande, liste des canaux. */
function afficherBascules() {
  for (const groupe of ["securite", "bande"]) {
    for (const b of $(groupe).querySelectorAll("button")) {
      b.setAttribute("aria-pressed", String(b.dataset.valeur === choix[groupe]));
    }
  }
  // Formulaire figé pendant qu'une opération tourne ou que le point d'accès est actif.
  const fige = Boolean(pa?.actif || pa?.en_cours);
  const ouvert = choix.securite === "ouvert";
  $("mdp").disabled = ouvert || fige;
  $("generer").disabled = ouvert || fige;
  $("avert-ouvert").hidden = !ouvert;

  const canaux = choix.bande === "auto" ? [] : CANAUX[choix.bande];
  if (!canaux.map(String).includes(choix.canal)) choix.canal = "";
  const select = $("canal");
  select.replaceChildren(
    el("option", { value: "", selected: choix.canal === "" }, choix.bande === "auto" ? "Auto (choisir la bande)" : "Auto"),
    ...canaux.map((c) => el("option", { value: String(c), selected: String(c) === choix.canal }, String(c))),
  );
  select.disabled = fige || choix.bande === "auto";
}

/** Le 5 GHz n'est proposé que si la carte choisie le permet. */
function desactiverIncompatibles() {
  const carte = carteChoisie();
  const b5 = $("bande").querySelector('[data-valeur="ghz5"]');
  b5.disabled = !carte?.ghz5;
  if (b5.disabled && choix.bande === "ghz5") choix.bande = "auto";
  const b24 = $("bande").querySelector('[data-valeur="ghz2_4"]');
  b24.disabled = carte ? !carte.ghz2 : false;
}

function afficherApercu() {
  const wifi = reseau?.wifi;
  const ouvert = wifi ? !wifi.mot_de_passe : choix.securite === "ouvert";
  $("apercu-nom").textContent = wifi?.nom ?? $("nom").value;
  $("apercu-lib-mdp").textContent = ouvert ? "Réseau ouvert" : "Mot de passe";
  $("apercu-mdp").textContent = ouvert ? "" : wifi?.mot_de_passe ?? $("mdp").value;
  // SVG produit par le serveur (bibliothèque qrcode).
  $("qr-wifi").innerHTML = wifi?.qr ?? "";
  $("qr-wifi").hidden = !wifi;
  $("apercu-note").hidden = Boolean(wifi);
}

// --- Gestes de l'animateur -----------------------------------------------------------

for (const groupe of ["securite", "bande"]) {
  $(groupe).addEventListener("click", (ev) => {
    const b = ev.target.closest("button");
    if (!b || b.disabled) return;
    choix[groupe] = b.dataset.valeur;
    afficherBascules();
    afficherApercu();
  });
}
$("canal").addEventListener("change", (ev) => (choix.canal = ev.target.value));
for (const r of document.querySelectorAll('input[name="adresse"]')) {
  r.addEventListener("change", () => (choix.adresse = r.value));
}
$("nom").addEventListener("input", afficherApercu);
$("mdp").addEventListener("input", afficherApercu);
$("generer").addEventListener("click", () => {
  attendSuggestion = true;
  commande("point_acces_suggerer");
});
$("rediagnostiquer").addEventListener("click", () => commande("diagnostic"));
$("arret-fin").addEventListener("change", (ev) => commande("point_acces_options", { arret_fin_seance: ev.target.checked }));
$("portail").addEventListener("change", (ev) => commande("point_acces_options", { portail: ev.target.value }));
$("reglages").addEventListener("submit", (ev) => ev.preventDefault());

$("principal-ap").addEventListener("click", () => {
  if (!pa) return;
  if (pa.actif || pa.diagnostic?.point_acces_actif) {
    const retour = pa.actif?.connexion_precedente;
    const texte = retour
      ? `Arrêter le point d'accès ? Les téléphones perdront la connexion ; le PC revient sur « ${retour} ».`
      : "Arrêter le point d'accès ? Les téléphones perdront la connexion.";
    if (confirm(texte)) commande("point_acces_arreter");
    return;
  }
  const carte = carteChoisie();
  if (!carte) {
    afficherMessage({ texte: "Choisissez une carte Wi-Fi capable de faire point d'accès.", erreur: true });
    return;
  }
  const nom = $("nom").value.trim();
  const mdp = $("mdp").value;
  if (!nom) {
    afficherMessage({ texte: "Donnez un nom au réseau.", erreur: true });
    $("nom").focus();
    return;
  }
  if (choix.securite === "wpa2" && (mdp.length < 8 || mdp.length > 63)) {
    afficherMessage({ texte: "Le mot de passe doit faire de 8 à 63 caractères.", erreur: true });
    $("mdp").focus();
    return;
  }
  if (carte.connexion && !confirm(`Démarrer le point d'accès sur ${carte.nom} coupe « ${carte.connexion} » : le PC perd internet pendant la séance. Continuer ?`)) {
    return;
  }
  commande("point_acces_demarrer", {
    carte: carte.nom,
    nom_reseau: nom,
    securite: choix.securite === "wpa2" ? { type: "wpa2", mot_de_passe: mdp } : { type: "ouvert" },
    bande: choix.bande,
    canal: choix.canal ? Number(choix.canal) : null,
    adresse: choix.adresse,
  });
});

afficherBascules();
