// Projection : accueil (QR code et code de la séance) ou résultat de l'activité en cours.
// Rien ne s'affiche sans le geste de l'animateur, ni sous 3 réponses (le serveur l'assure).

import { connecter } from "/commun/connexion.js";
import { el, codeLisible, logo, elementMinuterie, reglerMinuterie, NOMS_ACTIVITES } from "/commun/outils.js";
import { RENDUS } from "/ecran/activites.js";

const scene = document.getElementById("scene");
const contenu = document.getElementById("contenu");
const titre = document.getElementById("titre");
const compteur = document.getElementById("compteur");
const piedAdresse = document.getElementById("pied-adresse");
const horsLigne = document.getElementById("hors-ligne");
document.getElementById("marque").prepend(logo(40));

// Mise à l'échelle de la scène 1280 × 720 à la taille de la fenêtre.
function ajuster() {
  const echelle = Math.min(innerWidth / 1280, innerHeight / 720);
  scene.style.setProperty("--echelle", String(echelle));
}
addEventListener("resize", ajuster);
ajuster();

const cle = new URLSearchParams(location.search).get("cle") || "";
let reseau = null;
let vue = null;

connecter({
  bonjour: () => ({ role: "ecran", cle }),
  surConnexion: (ok) => {
    horsLigne.hidden = ok;
    horsLigne.textContent = ok ? "" : "Connexion au serveur perdue, on réessaie…";
  },
  surErreurFatale: (message) => {
    contenu.replaceChildren(el("p", { class: "message-plein" }, message));
  },
  surMessage: (m) => {
    if (m.t === "bienvenue" || m.t === "reseau") {
      reseau = m.t === "reseau" ? m : m.reseau;
      afficher();
    } else if (m.t === "etat") {
      vue = m;
      afficher();
    }
  },
});

function afficher() {
  if (!vue) return;
  titre.textContent = vue.titre;
  document.title = `${vue.titre} · écran Padaone`;
  compteur.replaceChildren(
    el("strong", {}, String(vue.connectes)),
    vue.connectes > 1 ? "connectés" : "connecté",
  );
  const a = vue.activite;
  piedAdresse.textContent = a
    ? `Pour rejoindre : ${reseau?.wifi ? `Wi-Fi ${reseau.wifi.nom}${reseau.wifi.mot_de_passe ? ` (${reseau.wifi.mot_de_passe})` : ""} · ` : ""}${reseau?.adresse ?? ""} · code ${codeLisible(vue.code)}`
    : "Anonyme par défaut. Tout reste sur l'ordinateur de l'animateur, dans cette salle.";
  if (!a) afficherAccueil();
  else afficherActivite(a);
}

function afficherAccueil() {
  // SVG produits par le serveur (bibliothèque qrcode), jamais par un participant.
  const qr = (svg, libelle, petit) => {
    const d = el("div", { class: `qr${petit ? " petit" : ""}`, role: "img", "aria-label": libelle });
    d.innerHTML = svg ?? "";
    return d;
  };
  const wifi = reseau?.wifi;
  const carteSeance = el(
    "section",
    { class: "carte-qr" },
    wifi ? el("span", { class: "numero", "aria-hidden": "true" }, "2") : null,
    qr(reseau?.qr, "QR code pour ouvrir la séance sur son téléphone", Boolean(wifi)),
    el(
      "div",
      { class: "infos" },
      el("h2", {}, "La séance"),
      el("span", { class: "libelle" }, "Adresse"),
      el("span", { class: "valeur" }, reseau?.adresse ?? ""),
      el("span", { class: "libelle" }, "Code"),
      el("span", { class: "code" }, codeLisible(vue.code)),
    ),
  );
  if (!wifi) {
    contenu.replaceChildren(
      el("h1", { class: "grand-titre" }, "Sortez vos téléphones !"),
      el("p", { class: "accroche" }, "Pas d'appli, pas de compte : un scan et c'est parti."),
      carteSeance,
    );
    return;
  }
  const carteWifi = el(
    "section",
    { class: "carte-qr" },
    el("span", { class: "numero premier", "aria-hidden": "true" }, "1"),
    qr(wifi.qr, `QR code pour rejoindre le Wi-Fi ${wifi.nom}`, true),
    el(
      "div",
      { class: "infos" },
      el("h2", {}, "Le Wi-Fi"),
      el("span", { class: "libelle" }, "Réseau"),
      el("span", { class: "valeur" }, wifi.nom),
      el("span", { class: "libelle" }, wifi.mot_de_passe ? "Mot de passe" : "Réseau ouvert"),
      wifi.mot_de_passe ? el("span", { class: "mot-de-passe" }, wifi.mot_de_passe) : null,
    ),
  );
  contenu.replaceChildren(
    el("h1", { class: "grand-titre" }, "Sortez vos téléphones !"),
    el("p", { class: "accroche" }, "Pas d'appli, pas de compte : deux scans et c'est parti."),
    el("div", { class: "deux-cartes" }, carteWifi, carteSeance),
  );
}

function afficherActivite(a) {
  reglerMinuterie(a.reste_ms, a.id);
  const entete = el(
    "div",
    { class: "ligne-activite" },
    el("span", { class: `pastille type-${a.type}` }, NOMS_ACTIVITES[a.type] ?? a.type),
    a.figee ? el("span", { class: "pastille fermee" }, "Réponses fermées") : null,
    a.reste_ms !== undefined && !a.resultat ? elementMinuterie("minuterie-ecran") : null,
  );
  // Les questions longues (ou le nuage, qui a besoin de place) prennent moins de hauteur.
  const compacte = a.type === "nuage" || a.question.length > 45;
  const question = el("h1", { class: `question${compacte ? " compacte" : ""}` }, a.question);
  const rendu = RENDUS[a.type];

  if (!a.resultat || !rendu) {
    const moderee = a.type === "ouverte" || a.type === "questions";
    let texte;
    if (!a.revelee) {
      texte = moderee
        ? "Répondez sur votre téléphone. L'animateur choisira ce qui s'affiche ici."
        : "Répondez sur votre téléphone. Les résultats arrivent bientôt.";
    } else {
      texte = "Encore un peu de patience : il faut au moins 3 participants pour afficher le résultat.";
    }
    const mot = a.type === "questions" ? ["participant", "participants"] : ["réponse", "réponses"];
    contenu.replaceChildren(
      entete,
      question,
      el(
        "div",
        { class: "attente-resultat", "aria-live": "polite" },
        el("strong", {}, String(a.reponses)),
        el("p", {}, a.reponses > 1 ? mot[1] : mot[0], el("br"), texte),
      ),
    );
    return;
  }
  contenu.replaceChildren(entete, question, ...rendu(a).filter(Boolean));
}
