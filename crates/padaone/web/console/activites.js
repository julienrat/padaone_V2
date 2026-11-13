// Console : champs de création de chaque type d'activité, et affichage des résultats
// avec les outils de modération. L'animateur ne voit jamais qui a répondu quoi.

import {
  el,
  lettre,
  nombre,
  carteOrdre,
  elementMinuterie,
  formatDuree,
  COULEURS_OPTIONS,
  COULEURS_ORDRE,
} from "/commun/outils.js";

let compteurChamps = 0;
const identifiant = (prefixe) => `${prefixe}-${++compteurChamps}`;

// --- Champs de création ----------------------------------------------------------------

function champQuestion(defaut = "", libelle = "Question") {
  const id = identifiant("question");
  const input = el("input", { id, class: "champ", maxlength: "200", required: true, autocomplete: "off", value: defaut });
  return { elements: [el("label", { for: id }, libelle), input], lire: () => input.value.trim(), input };
}

/** Liste d'options de 2 à 6, avec éventuellement le choix de la bonne réponse. */
function listeOptions(defauts, { bonne = false } = {}) {
  const MAX = 6;
  const groupe = identifiant("bonne");
  const conteneur = el("div", { class: "liste-options" });
  const ajouter = el("button", { type: "button", class: "b" }, "+ Option");

  function renumeroter() {
    [...conteneur.children].forEach((ligne, i) => {
      ligne.querySelector(".champ").setAttribute("aria-label", `Option ${lettre(i)}`);
      ligne.querySelector(".retirer").setAttribute("aria-label", `Retirer l'option ${lettre(i)}`);
      const radio = ligne.querySelector("input[type=radio]");
      if (radio) radio.setAttribute("aria-label", `L'option ${lettre(i)} est la bonne réponse`);
    });
    ajouter.disabled = conteneur.children.length >= MAX;
  }

  function ligne(valeur = "") {
    const l = el(
      "div",
      { class: "ligne-option" },
      bonne ? el("input", { type: "radio", name: groupe, class: "radio-bonne", required: true }) : null,
      el("input", { class: "champ", maxlength: "60", required: true, autocomplete: "off", value: valeur }),
      el(
        "button",
        {
          type: "button",
          class: "b retirer",
          onclick: () => {
            if (conteneur.children.length > 2) {
              l.remove();
              renumeroter();
            }
          },
        },
        "×",
      ),
    );
    return l;
  }

  defauts.forEach((d) => conteneur.append(ligne(d)));
  ajouter.addEventListener("click", () => {
    if (conteneur.children.length < MAX) {
      const l = ligne();
      conteneur.append(l);
      renumeroter();
      l.querySelector(".champ").focus();
    }
  });
  renumeroter();

  return {
    elements: [
      el(
        "fieldset",
        { class: "champs-options" },
        el("legend", {}, bonne ? "Options (cochez la bonne réponse)" : "Options (2 à 6)"),
        conteneur,
      ),
      ajouter,
    ],
    lire: () => [...conteneur.querySelectorAll(".champ")].map((i) => i.value.trim()),
    bonne: () => [...conteneur.querySelectorAll(".radio-bonne")].findIndex((r) => r.checked),
  };
}

function choixSimple(libelle, valeurs, defaut) {
  const id = identifiant("choix");
  const select = el(
    "select",
    { id, class: "champ" },
    valeurs.map(([v, t]) => el("option", { value: String(v), selected: v === defaut }, t)),
  );
  return { elements: [el("label", { for: id }, libelle), select], lire: () => Number(select.value) };
}

function texteFacultatif(libelle, placeholder) {
  const id = identifiant("texte");
  const input = el("input", { id, class: "champ", maxlength: "60", autocomplete: "off", placeholder });
  return { elements: [el("label", { for: id }, libelle), input], lire: () => input.value.trim() || null };
}

/** Pour chaque type : les champs du formulaire et la lecture de la configuration. */
export const FORMULAIRES = {
  sondage() {
    const q = champQuestion();
    const o = listeOptions(["", ""]);
    return { elements: [...q.elements, ...o.elements], lire: () => ({ question: q.lire(), options: o.lire() }) };
  },
  cartons() {
    const q = champQuestion("Êtes-vous d'accord ?");
    const o = listeOptions(["D'accord", "Pas d'accord", "Je ne sais pas"]);
    return { elements: [...q.elements, ...o.elements], lire: () => ({ question: q.lire(), options: o.lire() }) };
  },
  nuage() {
    const q = champQuestion("En un mot : ");
    const m = choixSimple("Mots par personne", [[1, "1 mot"], [2, "2 mots"], [3, "3 mots"]], 3);
    return { elements: [...q.elements, ...m.elements], lire: () => ({ question: q.lire(), max_mots: m.lire() }) };
  },
  echelle() {
    const q = champQuestion();
    const m = choixSimple("Échelle", [[5, "De 1 à 5"], [10, "De 1 à 10"]], 5);
    const bas = texteFacultatif("Libellé du 1 (facultatif)", "Pas du tout");
    const haut = texteFacultatif("Libellé du maximum (facultatif)", "Tout à fait");
    return {
      elements: [...q.elements, ...m.elements, ...bas.elements, ...haut.elements],
      lire: () => ({ question: q.lire(), max: m.lire(), libelle_min: bas.lire(), libelle_max: haut.lire() }),
    };
  },
  meteo() {
    const q = champQuestion("Comment ça va, là, maintenant ?");
    return { elements: q.elements, lire: () => ({ question: q.lire() }) };
  },
  pari() {
    const q = champQuestion();
    const o = listeOptions(["", "", ""], { bonne: true });
    return {
      elements: [...q.elements, ...o.elements],
      lire: () => ({ question: q.lire(), options: o.lire(), bonne: o.bonne() < 0 ? null : o.bonne() }),
    };
  },
  roti() {
    const q = champQuestion("Ce temps a-t-il été utile ?");
    return { elements: q.elements, lire: () => ({ question: q.lire() }) };
  },
  ouverte() {
    const q = champQuestion();
    return { elements: q.elements, lire: () => ({ question: q.lire() }) };
  },
  questions() {
    const q = champQuestion("Vos questions", "Consigne");
    return { elements: q.elements, lire: () => ({ question: q.lire() }) };
  },
  ordre() {
    const q = champQuestion("Remettez les étapes dans l'ordre", "Consigne");
    const e = listeElementsOrdre();
    const m = minuterieFormulaire(120);
    return {
      elements: [...q.elements, ...e.elements, ...m.elements],
      lire: () => ({ question: q.lire(), elements: e.lire(), duree: m.lire() }),
    };
  },
};

// --- Champs de la remise en ordre ---------------------------------------------------------

/** Éléments dans le BON ordre (3 à 8), chacun avec texte et couleur, et un aperçu. */
function listeElementsOrdre() {
  const MIN = 3;
  const MAX = 8;
  const couleurs = Object.keys(COULEURS_ORDRE);
  const conteneur = el("ol", { class: "elements-ordre" });
  const ajouter = el("button", { type: "button", class: "b" }, "+ Élément");

  function renumeroter() {
    [...conteneur.children].forEach((ligne, i) => {
      ligne.querySelector(".champ-texte").setAttribute("aria-label", `Élément ${i + 1} (dans le bon ordre)`);
      ligne.querySelector(".choix-couleur").setAttribute("aria-label", `Couleur de l'élément ${i + 1}`);
      ligne.querySelector(".retirer").setAttribute("aria-label", `Retirer l'élément ${i + 1}`);
    });
    ajouter.disabled = conteneur.children.length >= MAX;
  }

  function ligne(i) {
    const texte = el("input", { class: "champ champ-texte", maxlength: "40", required: true, autocomplete: "off" });
    const couleur = el(
      "select",
      { class: "champ choix-couleur" },
      couleurs.map((v, k) => el("option", { value: v, selected: k === i % couleurs.length }, COULEURS_ORDRE[v][2])),
    );
    const apercu = el("span", { class: "apercu-ordre", "aria-hidden": "true" });
    const maj = () => apercu.replaceChildren(carteOrdre({ texte: texte.value || "Aperçu", couleur: couleur.value }));
    for (const c of [texte, couleur]) c.addEventListener("input", maj);
    maj();
    const l = el(
      "li",
      { class: "ligne-element" },
      texte,
      el("div", { class: "reglages-element" }, couleur, apercu),
      el(
        "button",
        {
          type: "button",
          class: "b retirer",
          onclick: () => {
            if (conteneur.children.length > MIN) {
              l.remove();
              renumeroter();
            }
          },
        },
        "×",
      ),
    );
    return l;
  }

  for (let i = 0; i < MIN; i++) conteneur.append(ligne(i));
  ajouter.addEventListener("click", () => {
    if (conteneur.children.length < MAX) {
      const l = ligne(conteneur.children.length);
      conteneur.append(l);
      renumeroter();
      l.querySelector(".champ-texte").focus();
    }
  });
  renumeroter();

  return {
    elements: [
      el(
        "fieldset",
        { class: "champs-options" },
        el("legend", {}, "Éléments, dans le bon ordre (3 à 8, 40 caractères)"),
        el("p", { class: "petit" }, "Les téléphones les recevront mélangés."),
        conteneur,
      ),
      ajouter,
    ],
    lire: () =>
      [...conteneur.children].map((l) => ({
        texte: l.querySelector(".champ-texte").value.trim(),
        couleur: l.querySelector(".choix-couleur").value,
      })),
  };
}

/** Case « minuterie » et durée en minutes et secondes (10 s à 60 min). */
function minuterieFormulaire(defaut) {
  const id = `minuterie-${Math.random().toString(36).slice(2, 8)}`;
  const active = el("input", { type: "checkbox", id });
  const minutes = el("input", { class: "champ court", type: "number", min: "0", max: "60", value: String(Math.floor(defaut / 60)), "aria-label": "Minutes", disabled: true });
  const secondes = el("input", { class: "champ court", type: "number", min: "0", max: "59", step: "5", value: String(defaut % 60), "aria-label": "Secondes", disabled: true });
  active.addEventListener("change", () => {
    minutes.disabled = !active.checked;
    secondes.disabled = !active.checked;
  });
  return {
    elements: [
      el("label", { class: "case-minuterie", for: id }, active, " Minuterie"),
      el("div", { class: "duree" }, minutes, el("span", {}, "min"), secondes, el("span", {}, "s")),
      el("p", { class: "petit" }, "Elle démarre au lancement ; à zéro, les réponses sont figées."),
    ],
    lire: () => (active.checked ? (Number(minutes.value) || 0) * 60 + (Number(secondes.value) || 0) : null),
  };
}

// --- Résultats et modération ------------------------------------------------------------

function barres(libelles, effectifs, couleurs = COULEURS_OPTIONS, marque = null) {
  const max = Math.max(1, ...effectifs);
  return el(
    "ul",
    { class: "resultats", role: "list" },
    libelles.map((libelle, i) =>
      el(
        "li",
        { class: "resultat" },
        el("span", { class: "libelle" }, libelle, marque === i ? " ✓" : ""),
        el(
          "span",
          { class: "piste", "aria-hidden": "true" },
          el("span", { class: "remplissage", style: { width: `${(100 * effectifs[i]) / max}%`, background: couleurs[i % couleurs.length] } }),
        ),
        el("span", { class: "nombre" }, String(effectifs[i])),
      ),
    ),
  );
}

function chiffres(...paires) {
  return el(
    "dl",
    { class: "chiffres" },
    paires.map(([nom, valeur]) => el("div", {}, el("dt", {}, nom), el("dd", {}, valeur))),
  );
}

const optionsLettrees = (e) => e.options.map((o, i) => `${lettre(i)}. ${o}`);

/** Sélection en cours pour une fusion de mots : [activité, clé]. */
let selection = null;

function moderationMots(e, mots, commande, rendre) {
  if (!mots.length) return el("p", { class: "petit" }, "Aucun mot pour l'instant.");
  const choisi = selection && selection[0] === e.id ? selection[1] : null;
  const motChoisi = mots.find((m) => m.cle === choisi);
  return el(
    "div",
    { class: "moderation" },
    el("p", { class: "petit" }, "Touchez un mot puis un autre pour fusionner le second dans le premier. L'œil masque un mot de l'écran."),
    el(
      "ul",
      { class: "mots", role: "list" },
      mots.map((m) =>
        el(
          "li",
          { class: `mot${m.masque ? " masque" : ""}` },
          el(
            "button",
            {
              type: "button",
              class: "mot-texte",
              "aria-pressed": String(choisi === m.cle),
              "data-focus": `mot-${m.cle}`,
              onclick: () => {
                if (choisi && choisi !== m.cle) {
                  commande("fusionner", { id: e.id, de: m.cle, vers: choisi });
                  selection = null;
                } else {
                  selection = choisi === m.cle ? null : [e.id, m.cle];
                }
                rendre();
              },
            },
            m.texte,
            " ",
            el("span", { class: "n" }, String(m.n)),
            m.masque ? el("span", { class: "etat-masque" }, " · masqué") : null,
          ),
          el(
            "button",
            {
              type: "button",
              class: "mot-oeil",
              "aria-pressed": String(m.masque),
              "aria-label": m.masque ? `Réafficher « ${m.texte} » à l'écran` : `Masquer « ${m.texte} » de l'écran`,
              "data-focus": `oeil-${m.cle}`,
              onclick: () => commande("masquer_mot", { id: e.id, cle: m.cle, on: !m.masque }),
            },
            m.masque ? "◌" : "◉",
          ),
          ...(m.fusionnes ?? []).map((cle) =>
            el(
              "button",
              {
                type: "button",
                class: "mot-separer",
                "aria-label": `Séparer « ${cle} » de « ${m.texte} »`,
                onclick: () => commande("separer", { id: e.id, cle }),
              },
              `+ ${cle} ✕`,
            ),
          ),
        ),
      ),
    ),
    motChoisi
      ? el(
          "p",
          { class: "bandeau-fusion", role: "status" },
          `« ${motChoisi.texte} » choisi : touchez le mot à fusionner dedans, ou `,
          el("button", { type: "button", class: "b petit", onclick: () => ((selection = null), rendre()) }, "annulez"),
        )
      : null,
  );
}

function listeModeree(elements, commande, e, { etiquette }) {
  if (!elements.length) return el("p", { class: "petit" }, "Rien pour l'instant.");
  return el(
    "ul",
    { class: "liste-moderee", role: "list" },
    elements.map((x) =>
      el(
        "li",
        { class: x.valide ? "valide" : "" },
        el("span", { class: "texte-modere" }, x.texte),
        x.votes !== undefined ? el("span", { class: "votes" }, `▲ ${x.votes}`) : null,
        el(
          "button",
          {
            type: "button",
            class: `b petit${x.valide ? " encre" : ""}`,
            "aria-pressed": String(x.valide),
            "data-focus": `valider-${x.rang}`,
            onclick: () => commande("valider", { id: e.id, rang: x.rang, on: !x.valide }),
          },
          x.valide ? "✓ À l'écran (retirer)" : etiquette,
        ),
      ),
    ),
  );
}

/** Pour chaque type : le résultat détaillé et les outils de modération. */
export const RESULTATS = {
  sondage: (e) => barres(optionsLettrees(e), e.resultat.effectifs),
  cartons: (e) => barres(optionsLettrees(e), e.resultat.effectifs, ["#2dd4a7", "#ff6b4a", "#ffd23f", "#49a8ff", "#ff7ac6", "#6c3bff"]),
  nuage: (e, commande, rendre) => moderationMots(e, e.resultat.mots, commande, rendre),
  echelle: (e) => {
    const r = e.resultat;
    return [
      chiffres(["Médiane", nombre(r.mediane)], ["Moyenne", nombre(r.moyenne)], ["Réponses", String(r.total)]),
      barres(r.effectifs.map((_, i) => String(i + 1)), r.effectifs),
    ];
  },
  meteo: (e) => {
    const r = e.resultat;
    const quart = [0, 0, 0, 0];
    for (const p of r.points) quart[(p.y >= 50 ? 0 : 2) + (p.x >= 50 ? 1 : 0)] += 1;
    return [
      chiffres(
        ["Humeur moyenne", r.moyenne ? `${nombre(r.moyenne[0])}/100` : "–"],
        ["Énergie moyenne", r.moyenne ? `${nombre(r.moyenne[1])}/100` : "–"],
        ["Points", String(r.total)],
      ),
      barres(["Tendu·e (énergie +, humeur −)", "Plein d'élan", "À plat", "Serein·e (énergie −, humeur +)"], quart),
    ];
  },
  pari: (e) => {
    const r = e.resultat;
    return [
      el("p", { class: "petit" }, `Bonne réponse : ${lettre(r.bonne)}. ${e.options[r.bonne]}`),
      chiffres(
        ["Acquis solide", String(r.acquis_solide)],
        ["À consolider", String(r.a_consolider)],
        ["Fausses certitudes", String(r.fausses_certitudes)],
        ["Doute utile", String(r.doute_utile)],
      ),
      barres(optionsLettrees(e), r.effectifs, COULEURS_OPTIONS, r.bonne),
    ];
  },
  roti: (e, commande, rendre) => {
    const r = e.resultat;
    return [
      chiffres(["Moyenne", `${nombre(r.moyenne)} / 5`], ["Réponses", String(r.total)]),
      barres(["0", "1", "2", "3", "4", "5"].map((d) => `${d} doigt${d > 1 ? "s" : ""}`), r.effectifs, ["#ff7ac6"]),
      el("h2", { class: "h3" }, "Mots de la fin"),
      moderationMots(e, r.mots, commande, rendre),
    ];
  },
  ouverte: (e, commande) => [
    el("p", { class: "petit" }, "Rien n'est projeté sans votre validation. Les auteurs restent anonymes, même pour vous."),
    listeModeree(e.resultat.messages, commande, e, { etiquette: "Projeter" }),
  ],
  ordre: (e, commande) => {
    const r = e.resultat;
    const n = e.elements.length;
    const blocs = [
      chiffres(
        ["Parfaites", `${r.parfaits} / ${r.total}`],
        ["Score moyen", r.moyenne === null || r.moyenne === undefined ? "–" : `${nombre(r.moyenne)} / ${n}`],
      ),
      el(
        "ol",
        { class: "resultats-ordre", role: "list", "aria-label": "Bon ordre et éléments bien placés" },
        e.elements.map((x, k) =>
          el(
            "li",
            {},
            el("span", { class: "rang" }, `${k + 1}.`),
            carteOrdre(x),
            el("span", { class: "petit" }, `bien placé ${r.bien_places[k]} fois sur ${r.total}`),
          ),
        ),
      ),
    ];
    if (e.duree) {
      blocs.push(
        el(
          "div",
          { class: "commandes-minuterie" },
          e.reste_ms !== undefined ? elementMinuterie() : el("span", { class: "petit" }, "Minuterie pas encore démarrée."),
          el("button", { type: "button", class: "b petit", onclick: () => commande("minuterie", { id: e.id, on: true }) }, `Relancer la minuterie (${formatDuree(e.duree * 1000)})`),
          e.reste_ms > 0 ? el("button", { type: "button", class: "b petit", onclick: () => commande("minuterie", { id: e.id, on: false }) }, "Arrêter la minuterie") : null,
        ),
      );
    }
    return blocs;
  },
  questions: (e, commande) => [
    el("p", { class: "petit" }, "Validez les questions : elles deviennent visibles sur les téléphones (pour voter) et à l'écran."),
    listeModeree(e.resultat.questions, commande, e, { etiquette: "Valider" }),
  ],
};

