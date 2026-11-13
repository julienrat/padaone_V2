// Rendu des résultats à l'écran (1280 × 720, lisible à 10 m). Chaque fonction reçoit
// l'activité de la vue écran, dont `resultat` est présent (révélé et au moins 3 réponses).

import {
  el,
  svg,
  lettre,
  nombre,
  pluriel,
  COULEURS_OPTIONS,
  ENCRES_OPTIONS,
  COULEURS_TEXTE,
  iconeOrdre,
} from "/commun/outils.js";

const CARTONS = ["#2dd4a7", "#ff6b4a", "#ffd23f", "#49a8ff", "#ff7ac6", "#6c3bff"];
const ENCRES_CARTONS = ["#1b1530", "#1b1530", "#1b1530", "#1b1530", "#1b1530", "#ffffff"];

// --- Barres horizontales (sondage, cartons) --------------------------------------

function barres(options, effectifs, total, couleurs, encres, bonne = null) {
  const max = Math.max(1, ...effectifs);
  return el(
    "ul",
    { class: "barres", role: "list" },
    options.map((option, i) => {
      const n = effectifs[i] ?? 0;
      const pourcent = total ? Math.round((100 * n) / total) : 0;
      return el(
        "li",
        { class: `barre${bonne === i ? " bonne" : ""}` },
        el(
          "span",
          { class: "libelle" },
          el("span", { class: "lettre", "aria-hidden": "true", style: { background: couleurs[i], color: encres[i] } }, lettre(i)),
          option,
          bonne === i ? el("span", { class: "marque-bonne" }, "✓ bonne réponse") : null,
        ),
        el(
          "span",
          { class: "piste", "aria-hidden": "true" },
          el("span", { class: "remplissage", style: { display: "block", width: `${(100 * n) / max}%`, background: couleurs[i] } }),
        ),
        el("span", { class: "nombre" }, String(n), el("small", {}, `${pourcent} %`)),
      );
    }),
  );
}

function sondage(a) {
  const { effectifs, total } = a.resultat;
  return [barres(a.options, effectifs, total, COULEURS_OPTIONS, ENCRES_OPTIONS), el("p", { class: "discret" }, pluriel(total, "réponse"))];
}

function cartons(a) {
  const { effectifs, total } = a.resultat;
  return [barres(a.options, effectifs, total, CARTONS, ENCRES_CARTONS), el("p", { class: "discret" }, `${pluriel(total, "carton")} levé${total > 1 ? "s" : ""}`)];
}

// --- Nuage de mots : placement en spirale, stable ----------------------------------

const ZONE = { largeur: 1152, hauteur: 340 };
const TAILLE_MAX = 84;
const TAILLE_MIN = 24;
/** Places déjà attribuées : un mot posé garde son centre et grossit sur place. */
const placement = { activite: null, centres: new Map() };
const mesureur = document.createElement("canvas").getContext("2d");

function boite(texte, taille, centre) {
  mesureur.font = `800 ${taille}px "Bricolage Grotesque", "Lexend", system-ui, sans-serif`;
  const l = mesureur.measureText(texte).width + 16;
  const h = taille * 1.15;
  return { x: centre.x - l / 2, y: centre.y - h / 2, l, h };
}

function chevauche(b, autres) {
  return autres.some((o) => b.x < o.x + o.l && o.x < b.x + b.l && b.y < o.y + o.h && o.y < b.y + b.h);
}

function dedans(b) {
  return b.x >= 0 && b.y >= 0 && b.x + b.l <= ZONE.largeur && b.y + b.h <= ZONE.hauteur;
}

/** Calcule la position et la taille des mots, dans l'ordre d'arrivée. */
function disposer(idActivite, mots) {
  if (placement.activite !== idActivite) {
    placement.activite = idActivite;
    placement.centres = new Map();
  }
  const max = Math.max(1, ...mots.map((m) => m.n));
  const taille = (n) => Math.max(TAILLE_MIN, Math.round(TAILLE_MAX * Math.sqrt(n / max)));
  const poses = [];
  const resultat = [];
  for (const [i, mot] of mots.entries()) {
    const t = taille(mot.n);
    let centre = placement.centres.get(mot.cle);
    if (!centre) {
      const cx = ZONE.largeur / 2;
      const cy = ZONE.hauteur / 2;
      for (let pas = 0; pas < 3000; pas++) {
        const angle = pas * 0.3;
        const rayon = 3 * angle;
        const essai = { x: cx + rayon * Math.cos(angle) * 1.7, y: cy + rayon * Math.sin(angle) };
        const b = boite(mot.texte, t, essai);
        if (dedans(b) && !chevauche(b, poses)) {
          centre = essai;
          break;
        }
      }
      if (!centre) continue; // plus de place : le mot attend (le moins cité)
      placement.centres.set(mot.cle, centre);
    }
    poses.push(boite(mot.texte, t, centre));
    resultat.push({ mot, centre, taille: t, couleur: COULEURS_TEXTE[i % COULEURS_TEXTE.length] });
  }
  return resultat;
}

function nuageDeMots(idActivite, mots, zone = ZONE) {
  const conteneur = el("div", {
    class: "nuage",
    role: "list",
    "aria-label": "Nuage de mots",
    style: { width: `${zone.largeur}px`, height: `${zone.hauteur}px` },
  });
  for (const { mot, centre, taille, couleur } of disposer(idActivite, mots)) {
    conteneur.append(
      el(
        "span",
        {
          class: "mot",
          role: "listitem",
          "aria-label": `${mot.texte}, ${pluriel(mot.n, "fois", "fois")}`,
          style: { left: `${centre.x}px`, top: `${centre.y}px`, fontSize: `${taille}px`, color: couleur },
        },
        mot.texte,
      ),
    );
  }
  return conteneur;
}

function nuage(a) {
  const r = a.resultat;
  return [
    nuageDeMots(a.id, r.mots),
    el("p", { class: "discret" }, `${pluriel(r.participants, "participant")} · ${pluriel(r.mots.length, "mot")} · les mots proches sont regroupés`),
  ];
}

// --- Échelle : histogramme et médiane --------------------------------------------------

function colonnes(valeurs, effectifs, { hauteur = 220, marque = null, couleur = "var(--violet)" } = {}) {
  const max = Math.max(1, ...effectifs);
  return el(
    "ol",
    { class: "colonnes", role: "list", style: { height: `${hauteur + 70}px` } },
    valeurs.map((v, i) =>
      el(
        "li",
        { class: `colonne${marque === i ? " marquee" : ""}` },
        el("span", { class: "effectif" }, String(effectifs[i])),
        el("span", {
          class: "baton",
          "aria-hidden": "true",
          style: { height: `${Math.max(4, (hauteur * effectifs[i]) / max)}px`, background: couleur },
        }),
        el("span", { class: "valeur" }, String(v)),
      ),
    ),
  );
}

function echelle(a) {
  const r = a.resultat;
  const valeurs = Array.from({ length: a.max }, (_, i) => i + 1);
  const mediane = r.mediane;
  const marque = Number.isInteger(mediane) ? mediane - 1 : null;
  return [
    el(
      "div",
      { class: "disposition-echelle" },
      el(
        "div",
        {},
        colonnes(valeurs, r.effectifs, { marque }),
        a.libelle_min || a.libelle_max
          ? el("div", { class: "libelles-echelle" }, el("span", {}, a.libelle_min ?? ""), el("span", {}, a.libelle_max ?? ""))
          : null,
      ),
      el(
        "div",
        { class: "chiffre-cle" },
        el("span", { class: "libelle-chiffre" }, "Médiane"),
        el("strong", {}, nombre(mediane)),
        el("span", { class: "discret" }, `sur ${a.max} · moyenne ${nombre(r.moyenne)}`),
        el("span", { class: "discret" }, pluriel(r.total, "réponse")),
      ),
    ),
  ];
}

// --- Météo : nuage de points ---------------------------------------------------------

function meteo(a) {
  const r = a.resultat;
  const T = 340;
  const px = (v) => (v / 100) * T;
  const coins = [
    [12, 26, "start", "Tendu·e"],
    [T - 12, 26, "end", "Plein d'élan"],
    [12, T - 14, "start", "À plat"],
    [T - 12, T - 14, "end", "Serein·e"],
  ];
  const dessin = svg(
    "svg",
    { width: T, height: T, viewBox: `0 0 ${T} ${T}`, role: "img", "aria-label": `${pluriel(r.total, "point")} dans le carré humeur et énergie` },
    svg("rect", { x: 1.5, y: 1.5, width: T - 3, height: T - 3, rx: 20, fill: "#fff", stroke: "#1b1530", "stroke-width": 3 }),
    svg("line", { x1: T / 2, y1: 0, x2: T / 2, y2: T, stroke: "#e4daff", "stroke-width": 3 }),
    svg("line", { x1: 0, y1: T / 2, x2: T, y2: T / 2, stroke: "#e4daff", "stroke-width": 3 }),
    coins.map(([x, y, ancre, t]) => svg("text", { x, y, "text-anchor": ancre, class: "coin" }, t)),
    r.points.map((p) => svg("circle", { cx: px(p.x), cy: T - px(p.y), r: 12, fill: "#6c3bff", "fill-opacity": 0.55, stroke: "#1b1530", "stroke-width": 2 })),
    r.moyenne
      ? svg("rect", {
          x: px(r.moyenne[0]) - 14,
          y: T - px(r.moyenne[1]) - 14,
          width: 28,
          height: 28,
          fill: "#ffd23f",
          stroke: "#1b1530",
          "stroke-width": 3,
          transform: `rotate(45 ${px(r.moyenne[0])} ${T - px(r.moyenne[1])})`,
        })
      : null,
  );
  return [
    el(
      "div",
      { class: "disposition-meteo" },
      el("div", { class: "axes-meteo" }, el("span", { class: "axe-y" }, "Énergie ↑"), dessin, el("span", { class: "axe-x" }, "Humeur →")),
      el(
        "div",
        { class: "legende" },
        el("p", {}, el("span", { class: "puce point" }), pluriel(r.total, "personne")),
        el("p", {}, el("span", { class: "puce losange" }), "le point moyen de la salle"),
      ),
    ),
  ];
}

// --- Pari de certitude -----------------------------------------------------------------

function pari(a) {
  const r = a.resultat;
  const caseGrille = (n, nom, classe) =>
    el("div", { class: `case-grille ${classe}` }, el("strong", {}, String(n)), el("span", {}, nom));
  const pire = r.fausses_certitudes_par_option.indexOf(Math.max(...r.fausses_certitudes_par_option));
  return [
    el("p", { class: "bonne-reponse" }, "La bonne réponse : ", el("span", {}, `${lettre(r.bonne)} · ${a.options[r.bonne]}`)),
    el(
      "div",
      { class: "disposition-pari" },
      el(
        "div",
        { class: "grille-pari", role: "table", "aria-label": "Réponses croisées avec la certitude" },
        el("span", {}),
        el("span", { class: "entete-grille" }, "Réponse juste"),
        el("span", { class: "entete-grille" }, "Réponse fausse"),
        el("span", { class: "entete-ligne" }, "J'en étais sûr·e"),
        caseGrille(r.acquis_solide, "Acquis solide", "solide"),
        caseGrille(r.fausses_certitudes, "Fausses certitudes", "alerte"),
        el("span", { class: "entete-ligne" }, "Pas sûr·e"),
        caseGrille(r.a_consolider, "À consolider", "consolider"),
        caseGrille(r.doute_utile, "Doute utile", "doute"),
      ),
      el(
        "aside",
        { class: "repartition" },
        el("h2", {}, "Ce que la salle a répondu"),
        barres(a.options, r.effectifs, r.total, COULEURS_OPTIONS, ENCRES_OPTIONS, r.bonne),
        r.fausses_certitudes > 0
          ? el(
              "p",
              { class: "a-discuter" },
              el("strong", {}, "À discuter : "),
              `${r.fausses_certitudes_par_option[pire]} des ${r.fausses_certitudes} fausses certitudes portent sur la réponse ${lettre(pire)}.`,
            )
          : null,
      ),
    ),
  ];
}

// --- ROTI ------------------------------------------------------------------------------

function roti(a) {
  const r = a.resultat;
  const mots = r.mots.slice(0, 14);
  const max = Math.max(1, ...mots.map((m) => m.n));
  return [
    el(
      "div",
      { class: "disposition-roti" },
      colonnes([0, 1, 2, 3, 4, 5], r.effectifs, { hauteur: 200, couleur: "var(--rose)" }),
      el(
        "div",
        { class: "chiffre-cle" },
        el("span", { class: "libelle-chiffre" }, "Moyenne"),
        el("strong", {}, nombre(r.moyenne)),
        el("span", { class: "discret" }, `doigts sur 5 · ${pluriel(r.total, "réponse")}`),
      ),
      mots.length
        ? el(
            "ul",
            { class: "mots-fin", role: "list", "aria-label": "Mots de la fin" },
            mots.map((m, i) =>
              el(
                "li",
                { style: { fontSize: `${Math.round(22 + 30 * Math.sqrt(m.n / max))}px`, color: COULEURS_TEXTE[i % COULEURS_TEXTE.length] } },
                m.texte,
              ),
            ),
          )
        : null,
    ),
  ];
}

// --- Question ouverte et questions du public -------------------------------------------

function ouverte(a) {
  const messages = a.resultat.messages;
  if (!messages.length) return [el("p", { class: "message-plein" }, "Les messages choisis par l'animateur apparaîtront ici.")];
  return [
    el(
      "ul",
      { class: `messages n${messages.length}`, role: "list" },
      messages.map((m) => el("li", { class: "message" }, m.texte)),
    ),
  ];
}

function questions(a) {
  const liste = a.resultat.questions;
  if (!liste.length) return [el("p", { class: "message-plein" }, "Les questions validées apparaîtront ici, triées par votes.")];
  return [
    el(
      "ol",
      { class: "questions-public", role: "list" },
      liste.map((q, i) =>
        el(
          "li",
          {},
          el("span", { class: "rang-question", "aria-hidden": "true" }, String(i + 1)),
          el("span", { class: "texte-question" }, q.texte),
          el("span", { class: "votes" }, `▲ ${q.votes}`, el("small", {}, q.votes > 1 ? " votes" : " vote")),
        ),
      ),
    ),
  ];
}

// --- Remise en ordre : le bon ordre et la part de la salle qui a bien placé chaque élément --

function ordre(a) {
  const r = a.resultat;
  const bonOrdre = (a.solution ?? []).map((p) => a.elements[p]);
  const n = bonOrdre.length;
  return [
    el(
      "p",
      { class: "bilan-ordre" },
      el("strong", {}, String(r.parfaits)),
      ` proposition${r.parfaits > 1 ? "s" : ""} parfaite${r.parfaits > 1 ? "s" : ""} sur ${r.total}`,
      el("span", { class: "discret" }, ` · en moyenne ${nombre(r.moyenne)} élément${r.moyenne > 1 ? "s" : ""} sur ${n} bien placés`),
    ),
    el(
      "ol",
      { class: `bon-ordre n${n}`, role: "list", "aria-label": "Le bon ordre" },
      bonOrdre.map((e, k) => {
        const part = r.total ? Math.round((100 * r.bien_places[k]) / r.total) : 0;
        return el(
          "li",
          { class: "ligne-bon-ordre" },
          el("span", { class: "rang-ordre", "aria-hidden": "true" }, String(k + 1)),
          el("span", { class: "texte-ordre" }, iconeOrdre(e), e.texte),
          el("span", { class: "piste", "aria-hidden": "true" }, el("span", { class: "remplissage", style: { display: "block", width: `${part}%`, background: "var(--menthe)" } })),
          el("span", { class: "part-ordre" }, `${part} %`, el("small", {}, " bien placé")),
        );
      }),
    ),
  ];
}

export const RENDUS = { sondage, cartons, nuage, echelle, meteo, pari, roti, ouverte, questions, ordre };
