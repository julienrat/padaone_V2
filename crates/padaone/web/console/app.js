// Console de l'animateur : préparer le déroulé, lancer, révéler, figer, modérer, passer à la suite.
// L'animateur voit tous les résultats, jamais qui a répondu quoi.

import { connecter } from "/commun/connexion.js";
import { el, codeLisible, logo, pluriel, reglerMinuterie, NOMS_ACTIVITES, SIGNAUX } from "/commun/outils.js";
import { FORMULAIRES, RESULTATS } from "/console/activites.js";

const $ = (id) => document.getElementById(id);
$("marque").prepend(logo(30));

const cle = new URLSearchParams(location.search).get("cle") || "";
$("lien-ecran").href = `/ecran?cle=${encodeURIComponent(cle)}`;
$("lien-wifi").href = `/console/reseau.html?cle=${encodeURIComponent(cle)}`;

let vue = null;
let minuterieAlerte = null;
/** Nombre d'étapes au moment d'un ajout : le formulaire n'est vidé qu'une fois l'ajout accepté. */
let ajoutEnAttente = null;
/** Formulaire du type choisi : { elements, lire }. */
let formulaire = null;

const lien = connecter({
  bonjour: () => ({ role: "console", cle }),
  surConnexion: (ok) => alerte(ok ? "" : "Connexion au serveur perdue, on réessaie…", ok ? 0 : -1),
  surErreurFatale: (message) => alerte(message, -1),
  surMessage: (m) => {
    if (m.t === "bienvenue" || m.t === "reseau") {
      const r = m.t === "reseau" ? m : m.reseau;
      // SVG produit par le serveur (bibliothèque qrcode), jamais par un participant.
      $("qr").innerHTML = r?.qr ?? "";
      $("adresse").textContent = r ? `Adresse : ${r.adresse}${r.wifi ? ` · Wi-Fi « ${r.wifi.nom} »` : ""}` : "";
    } else if (m.t === "etat") {
      vue = m;
      if (ajoutEnAttente !== null && vue.deroule.length > ajoutEnAttente) {
        ajoutEnAttente = null;
        preparerFormulaire();
      }
      afficher();
    } else if (m.t === "erreur") {
      ajoutEnAttente = null;
      alerte(m.message);
    }
  },
});

/** Affiche un message ; `duree` en ms (0 : effacer, -1 : jusqu'au prochain message). */
function alerte(texte, duree = 6000) {
  clearTimeout(minuterieAlerte);
  $("alerte").hidden = !texte;
  $("alerte").textContent = texte;
  if (texte && duree > 0) minuterieAlerte = setTimeout(() => alerte(""), duree);
}

function commande(c, parametres = {}) {
  if (!lien.envoyer({ t: "commande", c, ...parametres })) {
    alerte("Pas de connexion au serveur : commande non envoyée.");
  }
}

// --- Affichage -------------------------------------------------------------

function afficher() {
  if (!vue) return;
  $("titre").textContent = vue.titre;
  document.title = `${vue.titre} · console Padaone`;
  $("code").textContent = `Code ${codeLisible(vue.code)}`;
  $("connectes").textContent = pluriel(vue.connectes, "connecté");
  $("inscrits").textContent = `${pluriel(vue.inscrits, "téléphone")} venu${vue.inscrits > 1 ? "s" : ""} depuis le début.`;
  if (document.activeElement !== $("nouveau-titre")) $("nouveau-titre").value = vue.titre;
  $("accueil").setAttribute("aria-pressed", String(vue.courante === null));
  const ap = vue.point_acces;
  $("lien-wifi").textContent = ap?.actif
    ? `● Wi-Fi « ${ap.actif.nom_reseau} »${ap.appareils !== null && ap.appareils !== undefined ? ` · ${pluriel(ap.appareils, "appareil")}` : ""}`
    : ap?.en_cours
      ? "Wi-Fi : opération en cours…"
      : "Wi-Fi : point d'accès";

  const focus = document.activeElement?.dataset?.focus;
  afficherDeroule();
  afficherPrincipal();
  afficherSignaux();
  if (focus) document.querySelector(`[data-focus="${CSS.escape(focus)}"]`)?.focus();
}

function afficherDeroule() {
  if (vue.deroule.length === 0) {
    $("etapes").replaceChildren(el("li", { class: "vide" }, "Aucune activité : ajoutez-en une ci-dessous."));
    return;
  }
  $("etapes").replaceChildren(
    ...vue.deroule.map((e, i) => {
      const enCours = e.id === vue.courante;
      const etats = [pluriel(e.reponses, e.type === "questions" ? "participant" : "réponse")];
      if (enCours) etats.unshift("en cours");
      if (e.revelee) etats.push("révélé");
      if (e.figee) etats.push("figé");
      return el(
        "li",
        { class: `etape${enCours ? " en-cours" : ""}` },
        el("span", { class: "nom-etape" }, `${i + 1}. ${e.question}`),
        el("span", { class: "note" }, `${NOMS_ACTIVITES[e.type] ?? e.type} · ${etats.join(" · ")}`),
        el(
          "div",
          { class: "boutons" },
          el(
            "button",
            {
              type: "button",
              class: "b petit",
              disabled: enCours,
              "data-focus": `lancer-${e.id}`,
              onclick: () => commande("lancer", { id: e.id }),
            },
            enCours ? "Projetée" : "Lancer",
          ),
          el(
            "button",
            {
              type: "button",
              class: "b petit",
              "aria-label": `Supprimer « ${e.question} »`,
              onclick: () => {
                if (confirm(`Supprimer « ${e.question} » et ses réponses ?`)) commande("supprimer", { id: e.id });
              },
            },
            "Supprimer",
          ),
        ),
      );
    }),
  );
}

function afficherPrincipal() {
  const principal = $("principal");
  const e = vue.deroule.find((x) => x.id === vue.courante);
  if (!e) {
    principal.replaceChildren(
      el("span", { class: "pastille", style: { alignSelf: "flex-start", background: "var(--menthe)" } }, "Accueil projeté"),
      el("h1", {}, "L'écran montre le QR code de la séance."),
      el("p", { class: "petit" }, "Lancez une activité du déroulé quand le groupe est prêt."),
    );
    return;
  }

  const suivante = vue.deroule[vue.deroule.indexOf(e) + 1];
  const moderee = e.type === "ouverte" || e.type === "questions";
  let statut;
  let classe = "masque";
  if (!e.revelee) statut = "À l'écran : résultat masqué. Seul le nombre de réponses est projeté.";
  else if (!e.affichable) statut = "À l'écran : révélé, mais masqué tant qu'il y a moins de 3 participants (anonymat).";
  else {
    statut = moderee ? "À l'écran : seuls les éléments validés s'affichent." : "À l'écran : résultat affiché.";
    classe = "visible";
  }
  reglerMinuterie(e.reste_ms, e.id);
  const detail = RESULTATS[e.type]?.(e, commande, afficher) ?? [];

  principal.replaceChildren(
    el(
      "div",
      { class: "ligne-haut" },
      el("span", { class: "pastille" }, `${NOMS_ACTIVITES[e.type] ?? e.type} · ${e.figee ? "figé" : "en cours"}`),
      el("span", { class: "petit" }, `${pluriel(vue.connectes, "connecté")} · ${pluriel(e.reponses, e.type === "questions" ? "participant" : "réponse")}`),
    ),
    el("h1", {}, e.question),
    el("p", { class: `statut-ecran ${classe}`, role: "status" }, statut),
    el("div", { class: "detail" }, detail),
    el(
      "div",
      { class: "commandes" },
      el(
        "button",
        {
          type: "button",
          class: "b violet",
          "aria-pressed": String(e.revelee),
          "data-focus": "reveler",
          onclick: () => commande("reveler", { id: e.id, on: !e.revelee }),
        },
        e.revelee ? "Masquer le résultat" : moderee ? "Afficher à l'écran" : "Révéler le résultat",
      ),
      el(
        "button",
        {
          type: "button",
          class: "b",
          "aria-pressed": String(e.figee),
          "data-focus": "figer",
          onclick: () => commande("figer", { id: e.id, on: !e.figee }),
        },
        e.figee ? "Rouvrir les réponses" : "Figer les réponses",
      ),
      el(
        "button",
        {
          type: "button",
          class: "b",
          onclick: () => {
            if (confirm("Effacer toutes les réponses de cette activité ?")) commande("reinitialiser", { id: e.id });
          },
        },
        "Réinitialiser",
      ),
      suivante
        ? el(
            "button",
            { type: "button", class: "b encre pousse", "data-focus": "suivante", onclick: () => commande("lancer", { id: suivante.id }) },
            "Activité suivante",
          )
        : el("button", { type: "button", class: "b encre pousse", onclick: () => commande("accueil") }, "Terminer : accueil"),
    ),
  );
}

function afficherSignaux() {
  const comptes = new Map((vue.signaux ?? []).map((s) => [s.type, s]));
  $("signaux").replaceChildren(
    ...SIGNAUX.map(({ type, nom }) => {
      const s = comptes.get(type) ?? { n: 0, alerte: false };
      return el(
        "li",
        { class: `signal${s.alerte ? " en-alerte" : ""}` },
        el("span", { class: "pastille-signal", "aria-hidden": "true" }),
        el("span", { class: "nom-signal" }, nom),
        el("span", { class: "jauge", "aria-hidden": "true" }, el("span", { style: { width: `${vue.connectes ? Math.min(100, (100 * s.n) / vue.connectes) : 0}%` } })),
        el("strong", {}, String(s.n)),
        s.alerte ? el("span", { class: "visuellement-cache" }, " (alerte)") : null,
      );
    }),
  );
  const alertes = SIGNAUX.filter(({ type }) => comptes.get(type)?.alerte);
  const texte = alertes
    .map(({ type, nom }) => `${nom} : ${pluriel(comptes.get(type).n, "personne")}`)
    .join(" · ");
  $("alerte-signal").hidden = !texte;
  $("alerte-signal").textContent = texte ? `${texte} (au moins 20 % de la salle). Les signaux s'effacent après 3 minutes.` : "";
}

// --- Formulaires -----------------------------------------------------------

function preparerFormulaire() {
  const type = $("type").value;
  formulaire = FORMULAIRES[type]();
  $("champs").replaceChildren(...formulaire.elements);
}

$("type").addEventListener("change", preparerFormulaire);

$("nouveau").addEventListener("submit", (ev) => {
  ev.preventDefault();
  ajoutEnAttente = vue ? vue.deroule.length : 0;
  commande("ajouter", { type: $("type").value, ...formulaire.lire() });
});

$("renommer").addEventListener("submit", (ev) => {
  ev.preventDefault();
  commande("titre", { titre: $("nouveau-titre").value.trim() });
  $("nouveau-titre").blur();
});

$("accueil").addEventListener("click", () => commande("accueil"));

preparerFormulaire();
