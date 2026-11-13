// Petits outils partagés. Tout le texte passe par textContent : jamais d'innerHTML
// avec une saisie, pour qu'un participant ne puisse pas injecter de balisage.

/**
 * Crée un élément : el("button", { class: "x", onclick: f }, "Texte", autreElement)
 * Les attributs à `false`, `null` ou `undefined` sont ignorés.
 */
export function el(balise, attributs = {}, ...enfants) {
  const e = document.createElement(balise);
  for (const [nom, valeur] of Object.entries(attributs)) {
    if (valeur === false || valeur === null || valeur === undefined) continue;
    if (nom.startsWith("on") && typeof valeur === "function") {
      e.addEventListener(nom.slice(2), valeur);
    } else if (nom === "style" && typeof valeur === "object") {
      Object.assign(e.style, valeur);
    } else {
      e.setAttribute(nom, valeur === true ? "" : String(valeur));
    }
  }
  for (const enfant of enfants.flat()) {
    if (enfant === null || enfant === undefined || enfant === false) continue;
    e.append(enfant instanceof Node ? enfant : document.createTextNode(String(enfant)));
  }
  return e;
}

/** « 482731 » devient « 482 731 ». */
export function codeLisible(code) {
  return code && code.length === 6 ? `${code.slice(0, 3)} ${code.slice(3)}` : code ?? "";
}

/** Accord simple : pluriel(3, "réponse") donne « 3 réponses ». */
export function pluriel(n, mot, motPluriel = `${mot}s`) {
  return `${n} ${n > 1 ? motPluriel : mot}`;
}

/** Couleurs des options, dans l'ordre (le sens est toujours aussi porté par le texte). */
export const COULEURS_OPTIONS = ["#6c3bff", "#ff6b4a", "#ffd23f", "#2dd4a7", "#ff7ac6", "#49a8ff"];
/** Encre lisible sur chaque couleur d'option (contraste d'au moins 4,5:1). */
export const ENCRES_OPTIONS = ["#ffffff", "#1b1530", "#1b1530", "#1b1530", "#1b1530", "#1b1530"];

/** Logo Padaone (trois bulles), décoratif. */
export function logo(taille = 28) {
  const ns = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(ns, "svg");
  svg.setAttribute("width", taille);
  svg.setAttribute("height", taille);
  svg.setAttribute("viewBox", "0 0 32 32");
  svg.setAttribute("aria-hidden", "true");
  for (const [cx, cy, r, c] of [
    [10, 10, 7, "#6C3BFF"],
    [23, 11, 5, "#FF6B4A"],
    [15, 23, 6, "#2DD4A7"],
  ]) {
    const cercle = document.createElementNS(ns, "circle");
    cercle.setAttribute("cx", cx);
    cercle.setAttribute("cy", cy);
    cercle.setAttribute("r", r);
    cercle.setAttribute("fill", c);
    svg.append(cercle);
  }
  return svg;
}

/** Nom affiché de chaque type d'activité. */
export const NOMS_ACTIVITES = {
  sondage: "Sondage",
  cartons: "Cartons",
  nuage: "Nuage de mots",
  echelle: "Échelle",
  meteo: "Météo",
  pari: "Pari de certitude",
  roti: "ROTI",
  ouverte: "Question ouverte",
  questions: "Questions du public",
  ordre: "Remise en ordre",
};

/** Degrés de certitude du pari, de 0 à 2. */
export const CERTITUDES = ["Je devine", "Je pense", "Sûr·e"];

/** Signaux discrets, dans l'ordre de la barre. */
export const SIGNAUX = [
  { type: "vite", nom: "Trop vite" },
  { type: "perdu", nom: "Perdu·e" },
  { type: "exemple", nom: "Un exemple" },
  { type: "pause", nom: "Une pause" },
];

/** Couleurs de texte lisibles sur le fond lilas (contraste d'au moins 4,5:1), pour le nuage. */
export const COULEURS_TEXTE = ["#1b1530", "#6c3bff", "#be185d", "#0f766e", "#4a1fd1", "#b4401a"];

/** Nombre à la française : 3.5 donne « 3,5 ». */
export function nombre(n, decimales = 1) {
  if (n === null || n === undefined) return "–";
  return n.toLocaleString("fr-FR", { maximumFractionDigits: decimales });
}

/** Lettre d'une option : 0 donne « A ». */
export function lettre(i) {
  return String.fromCharCode(65 + i);
}

/** Crée un élément SVG : svg("circle", { cx: 1, cy: 2, r: 3 }). */
export function svg(balise, attributs = {}, ...enfants) {
  const e = document.createElementNS("http://www.w3.org/2000/svg", balise);
  for (const [nom, valeur] of Object.entries(attributs)) {
    if (valeur !== null && valeur !== undefined && valeur !== false) e.setAttribute(nom, String(valeur));
  }
  for (const enfant of enfants.flat()) {
    if (enfant === null || enfant === undefined || enfant === false) continue;
    e.append(enfant instanceof Node ? enfant : document.createTextNode(String(enfant)));
  }
  return e;
}

/** Couleurs des éléments à remettre en ordre : fond, encre lisible dessus (contraste ≥ 4,5:1), nom. */
export const COULEURS_ORDRE = {
  violet: ["#6c3bff", "#ffffff", "Violet"],
  corail: ["#ff6b4a", "#1b1530", "Corail"],
  soleil: ["#ffd23f", "#1b1530", "Soleil"],
  menthe: ["#2dd4a7", "#1b1530", "Menthe"],
  rose: ["#ff7ac6", "#1b1530", "Rose"],
  ciel: ["#49a8ff", "#1b1530", "Ciel"],
  lilas: ["#e4daff", "#1b1530", "Lilas"],
  peche: ["#ffe1d6", "#1b1530", "Pêche"],
  blanc: ["#ffffff", "#1b1530", "Blanc"],
  encre: ["#1b1530", "#ffffff", "Encre"],
};

/** Zone de texte d'un élément : rectangle arrondi, dans sa couleur (classe dans theme.css). */
export function carteOrdre(element, classe = "") {
  const [fond, encre] = COULEURS_ORDRE[element.couleur] ?? COULEURS_ORDRE.violet;
  return el(
    "span",
    { class: `carte-ordre ${classe}`, style: { background: fond, color: encre } },
    element.texte,
  );
}

/** Pastille de la couleur d'un élément, sans texte (pour l'écran projeté). */
export function iconeOrdre(element) {
  const [fond] = COULEURS_ORDRE[element.couleur] ?? COULEURS_ORDRE.violet;
  return el("span", {
    class: "icone-ordre",
    style: { background: fond },
    "aria-hidden": "true",
  });
}

// --- Minuterie -----------------------------------------------------------------
// Le serveur envoie le temps restant ; chaque interface en déduit une échéance selon sa
// propre horloge (les horloges des téléphones peuvent différer de celle du PC).

let echeance = null;
let activiteMinuterie = null;

/** Durée lisible : 83 000 ms donne « 1:23 ». */
export function formatDuree(ms) {
  const s = Math.max(0, Math.ceil(ms / 1000));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

/** Recale l'échéance à la réception d'une vue (`reste` absent : pas de minuterie). */
export function reglerMinuterie(reste, activite) {
  if (reste === undefined || reste === null) {
    echeance = null;
  } else {
    const nouvelle = Date.now() + reste;
    // On ne recale que si l'écart est sensible, pour éviter que l'affichage tressaute.
    if (echeance === null || activite !== activiteMinuterie || Math.abs(nouvelle - echeance) > 1500) {
      echeance = nouvelle;
    }
  }
  activiteMinuterie = activite;
  majMinuteries();
}

/** Temps restant selon l'horloge locale, ou `null` sans minuterie. */
export function resteMinuterie() {
  return echeance === null ? null : Math.max(0, echeance - Date.now());
}

/** Compte à rebours, mis à jour seul quatre fois par seconde. */
export function elementMinuterie(classe = "") {
  const e = el("span", { class: `minuterie ${classe}`, role: "timer", "aria-label": "Temps restant" });
  majUne(e);
  return e;
}

function majUne(e) {
  const reste = resteMinuterie();
  if (reste === null) {
    e.hidden = true;
    return;
  }
  e.hidden = false;
  e.textContent = reste > 0 ? `⏱ ${formatDuree(reste)}` : "⏱ Temps écoulé";
  e.classList.toggle("urgente", reste > 0 && reste <= 10_000);
  e.classList.toggle("finie", reste === 0);
}

function majMinuteries() {
  document.querySelectorAll(".minuterie").forEach(majUne);
}

setInterval(majMinuteries, 250);
