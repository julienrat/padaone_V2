// Téléphone des participants : rejoindre la séance, répondre à l'activité en cours,
// et la barre « Signal discret » toujours accessible en bas.
// Le jeton anonyme est gardé dans le navigateur pour retrouver sa place après une coupure.

import { connecter } from "/commun/connexion.js";
import { el, codeLisible, logo, NOMS_ACTIVITES, SIGNAUX } from "/commun/outils.js";
import { RENDUS } from "/p/activites.js";

const contenu = document.getElementById("contenu");
const bandeau = document.getElementById("bandeau");
const barreSignaux = document.getElementById("signaux");
document.getElementById("marque").prepend(logo(28));

// Stockage local : peut être refusé (navigation privée), on s'en passe alors.
const stock = {
  lire(cle) {
    try {
      return localStorage.getItem(`padaone.${cle}`);
    } catch {
      return null;
    }
  },
  ecrire(cle, valeur) {
    try {
      if (valeur === null) localStorage.removeItem(`padaone.${cle}`);
      else localStorage.setItem(`padaone.${cle}`, valeur);
    } catch {
      /* rien */
    }
  },
};

// --- Texte agrandi ---------------------------------------------------------

const agrandir = document.getElementById("agrandir");
function appliquerTaille(grand) {
  document.documentElement.classList.toggle("grand-texte", grand);
  agrandir.setAttribute("aria-pressed", String(grand));
}
appliquerTaille(stock.lire("grand-texte") === "1");
agrandir.addEventListener("click", () => {
  const grand = !document.documentElement.classList.contains("grand-texte");
  appliquerTaille(grand);
  stock.ecrire("grand-texte", grand ? "1" : null);
});

// --- Séance ----------------------------------------------------------------

const parametres = new URLSearchParams(location.search);
let code = (parametres.get("code") || stock.lire("code") || "").replace(/\D/g, "").slice(0, 6);
let lien = null;
let derniereVue = "";
let vue = null;

/** Saisies en cours, par activité : conservées entre deux rendus. */
const brouillons = new Map();
/** Activités en mode « Modifier ». */
const enEdition = new Set();
/** Valeurs locales par activité (réponse tapotée pas encore confirmée, envoi en cours…). */
const locaux = new Map();
/** Signaux tapotés, en attente de confirmation. */
let signauxLocaux = null;

function jeton() {
  return stock.lire(`jeton.${code}`);
}

function montrerBandeau(texte, erreur = false) {
  bandeau.hidden = !texte;
  bandeau.textContent = texte || "";
  bandeau.classList.toggle("erreur", erreur);
}

function rejoindre() {
  if (lien) return;
  montrerBandeau("Connexion…");
  lien = connecter({
    bonjour: () => ({ role: "participant", code, jeton: jeton() }),
    surConnexion: (ok) => montrerBandeau(ok ? "" : "Connexion perdue, on réessaie…"),
    surErreurFatale: (message) => {
      lien = null;
      stock.ecrire(`jeton.${code}`, null);
      barreSignaux.hidden = true;
      afficherAccueil(message);
    },
    surMessage,
  });
}

function local(id) {
  if (!locaux.has(id)) locaux.set(id, {});
  return locaux.get(id);
}

function surMessage(m) {
  if (m.t === "bienvenue") {
    stock.ecrire("code", code);
    if (m.jeton) stock.ecrire(`jeton.${code}`, m.jeton);
    document.title = `${m.seance.titre} · Padaone`;
    barreSignaux.hidden = false;
  } else if (m.t === "etat") {
    const texte = JSON.stringify(m);
    if (texte === derniereVue) return; // rien de neuf pour ce téléphone
    derniereVue = texte;
    vue = m;
    signauxLocaux = null;
    const a = m.activite;
    if (a) {
      // La réponse tapotée est confirmée : la valeur locale n'est plus utile.
      const l = local(a.id);
      if (l.choix !== undefined && a.ma_reponse?.choix === l.choix) delete l.choix;
      if (l.valeur !== undefined && a.ma_reponse?.valeur === l.valeur) delete l.valeur;
    }
    afficher();
  } else if (m.t === "erreur") {
    if (vue?.activite) {
      const l = local(vue.activite.id);
      delete l.choix;
      delete l.valeur;
    }
    signauxLocaux = null;
    montrerBandeau(m.message, true);
    afficher();
  }
}

// --- Rejoindre -------------------------------------------------------------

/** Page ouverte par la fenêtre « Se connecter au réseau » du téléphone (portail captif). */
const parPortail = parametres.get("portail") === "1";
if (parPortail) {
  const aide = document.createElement("p");
  aide.className = "aide-portail";
  aide.textContent =
    "Vous êtes connecté·e au Wi-Fi de la séance. Gardez cette fenêtre ouverte pendant la séance : " +
    "si elle se ferme, rouvrez-la par la notification « Se connecter au réseau ».";
  document.querySelector(".entete").after(aide);
}

function afficherAccueil(erreur = "") {
  const champ = el("input", {
    id: "code",
    class: "champ-code",
    inputmode: "numeric",
    autocomplete: "off",
    maxlength: "7",
    value: codeLisible(code),
    "aria-describedby": "aide-code",
    required: true,
  });
  const formulaire = el(
    "form",
    {
      class: "carte",
      onsubmit: (ev) => {
        ev.preventDefault();
        const saisi = champ.value.replace(/\D/g, "");
        if (saisi.length !== 6) {
          montrerBandeau("Le code a 6 chiffres : il est affiché à l'écran.", true);
          champ.focus();
          return;
        }
        code = saisi;
        rejoindre();
      },
    },
    el("label", { for: "code" }, "Code de la séance"),
    champ,
    el("p", { class: "aide", id: "aide-code" }, "Le code est projeté à l'écran, sous le QR code."),
    el("button", { type: "submit", class: "bouton-principal" }, "C'est parti !"),
  );
  contenu.replaceChildren(
    el("div", {}, el("h1", {}, "Bienvenue !"), el("p", { class: "sous-titre" }, "Anonyme, sans compte, sans appli.")),
    formulaire,
  );
  montrerBandeau(erreur, Boolean(erreur));
}

// --- Activité en cours -------------------------------------------------------

function afficher() {
  if (!vue) return;
  // Garder le focus clavier (et le curseur de saisie) sur le même élément après le rendu.
  const actif = document.activeElement;
  const focus = actif?.dataset?.focus;
  const selection =
    actif && typeof actif.selectionStart === "number" ? [actif.selectionStart, actif.selectionEnd] : null;
  rendre();
  afficherSignaux();
  if (focus) {
    const cible = contenu.querySelector(`[data-focus="${focus}"]`);
    if (cible) {
      cible.focus();
      if (selection && typeof cible.setSelectionRange === "function") {
        try {
          cible.setSelectionRange(...selection);
        } catch {
          /* type de champ sans sélection */
        }
      }
    }
  }
}

function rendre() {
  const a = vue.activite;
  if (!a) {
    contenu.replaceChildren(
      el(
        "div",
        { class: "attente" },
        el("div", { class: "bulle", "aria-hidden": "true" }),
        el("h1", {}, "Vous êtes dans la séance !"),
        el("p", { class: "sous-titre" }, vue.titre),
        el("p", {}, "L'animateur va lancer la prochaine activité. Gardez cette page ouverte."),
      ),
    );
    return;
  }
  const rendu = RENDUS[a.type];
  if (!rendu) {
    contenu.replaceChildren(el("p", {}, "Cette activité n'est pas encore disponible sur ce téléphone."));
    return;
  }
  const cle = (c) => `${a.id}.${c}`;
  const ctx = {
    local: local(a.id),
    enEdition: enEdition.has(a.id),
    envoyer(reponse, optimiste) {
      if (!lien || !lien.envoyer({ t: "reponse", activite: a.id, ...reponse })) {
        montrerBandeau("Pas de connexion : réessayez dans un instant.", true);
        return false;
      }
      montrerBandeau("");
      if (optimiste) {
        Object.assign(local(a.id), optimiste);
        afficher();
      }
      return true;
    },
    brouillon(c, defaut) {
      if (!brouillons.has(cle(c))) brouillons.set(cle(c), defaut);
      return brouillons.get(cle(c));
    },
    garder(c, valeur) {
      brouillons.set(cle(c), valeur);
    },
    editer(on) {
      if (on) enEdition.add(a.id);
      else enEdition.delete(a.id);
      afficher();
    },
    rendre: afficher,
  };
  contenu.replaceChildren(
    el("span", { class: `pastille etiquette type-${a.type}` }, NOMS_ACTIVITES[a.type] ?? a.type),
    ...rendu(a, ctx).filter(Boolean),
  );
}

// --- Signal discret ----------------------------------------------------------

function afficherSignaux() {
  const actifs = new Set(signauxLocaux ?? vue?.mes_signaux ?? []);
  barreSignaux.replaceChildren(
    ...SIGNAUX.map(({ type, nom }) =>
      el(
        "button",
        {
          type: "button",
          class: "signal",
          "aria-pressed": String(actifs.has(type)),
          onclick: () => {
            if (!lien || !lien.envoyer({ t: "signal", type })) return;
            const nouveaux = new Set(actifs);
            if (nouveaux.has(type)) nouveaux.delete(type);
            else nouveaux.add(type);
            signauxLocaux = [...nouveaux];
            afficherSignaux();
            barreSignaux.querySelector(`[data-type="${type}"]`)?.focus();
          },
          "data-type": type,
        },
        el("span", { class: "icone-signal", "aria-hidden": "true" }, ICONES[type]),
        nom,
      ),
    ),
  );
}

const ICONES = { vite: "»", perdu: "?", exemple: "…", pause: "Ⅱ" };

// --- Démarrage -------------------------------------------------------------

if (code.length === 6 && jeton()) {
  rejoindre(); // retour sur la page : on reprend sa place directement
} else {
  afficherAccueil();
}
