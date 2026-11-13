// Rendu de chaque activité sur le téléphone. Chaque fonction reçoit l'activité en cours
// (vue participant) et un contexte, et renvoie les éléments à afficher.
//
// Contexte `ctx` :
//   envoyer(reponse)      envoie {t: "reponse", activite, ...reponse} ; vrai si parti
//   brouillon(cle, defaut) / garder(cle, valeur)   saisie en cours, conservée entre deux rendus
//   enEdition / editer(on)   mode « Modifier » après un envoi
//   rendre()              redessine

import { el, lettre, carteOrdre, elementMinuterie, CERTITUDES, COULEURS_OPTIONS, ENCRES_OPTIONS } from "/commun/outils.js";

/** Couleurs des cartons (fond) et encre lisible dessus. */
const CARTONS = [
  ["#2dd4a7", "#1b1530"],
  ["#ff6b4a", "#1b1530"],
  ["#ffd23f", "#1b1530"],
  ["#49a8ff", "#1b1530"],
  ["#ff7ac6", "#1b1530"],
  ["#6c3bff", "#ffffff"],
];

function bouton(texte, onclick, attributs = {}) {
  return el("button", { type: "button", class: "bouton-principal", onclick, ...attributs }, texte);
}

function boutonSecondaire(texte, onclick, attributs = {}) {
  return el("button", { type: "button", class: "bouton-secondaire", onclick, ...attributs }, texte);
}

function statut(texte, ferme = false) {
  return el("p", { class: `statut${ferme ? " ferme" : ""}`, role: "status" }, texte);
}

function fermee(a) {
  return a.ouverte ? null : statut("Les réponses sont fermées.", true);
}

/** Écran « Envoyé ! » avec un rappel de la réponse et un bouton pour la modifier. */
function envoye(a, ctx, titre, rappel, detail) {
  return [
    el(
      "div",
      { class: "envoye", role: "status" },
      el("span", { class: "coche-ronde", "aria-hidden": "true" }, "✓"),
      el("p", { class: "envoye-titre" }, titre),
      rappel ? el("p", { class: "rappel" }, rappel) : null,
      detail ? el("p", { class: "aide" }, detail) : null,
    ),
    a.ouverte ? boutonSecondaire("Modifier", () => ctx.editer(true), { "data-focus": "modifier" }) : fermee(a),
  ];
}

/** Liste de boutons à choix unique (options, valeurs, certitudes). */
function choix(liste, actif, surChoix, { desactive = false, prefixe = "choix", classe = "option" } = {}) {
  return el(
    "ul",
    { class: "options", role: "list" },
    liste.map((item, i) =>
      el(
        "li",
        {},
        el(
          "button",
          {
            type: "button",
            class: classe,
            "aria-pressed": String(actif === i),
            disabled: desactive,
            "data-focus": `${prefixe}-${i}`,
            onclick: () => surChoix(i),
          },
          item,
          actif === i ? el("span", { class: "coche" }, "✓ Mon choix") : null,
        ),
      ),
    ),
  );
}

function pastilleOption(i) {
  return el(
    "span",
    {
      class: "pastille-option",
      "aria-hidden": "true",
      style: { background: COULEURS_OPTIONS[i], color: ENCRES_OPTIONS[i] },
    },
    lettre(i),
  );
}

// --- Sondage -----------------------------------------------------------------

function sondage(a, ctx) {
  const choisi = ctx.local.choix ?? a.ma_reponse?.choix ?? null;
  const items = a.options.map((texte, i) => [pastilleOption(i), el("span", { class: "texte" }, texte)]);
  return [
    el("h1", { class: "question" }, a.question),
    choix(items, choisi, (i) => ctx.envoyer({ choix: i }, { choix: i }), { desactive: !a.ouverte }),
    fermee(a) ??
      (choisi !== null
        ? statut("C'est noté ! Vous pouvez encore changer d'avis.")
        : el("p", { class: "aide" }, "Touchez une réponse.")),
  ];
}

// --- Cartons -----------------------------------------------------------------

function cartons(a, ctx) {
  const choisi = ctx.local.choix ?? a.ma_reponse?.choix ?? null;
  if (choisi !== null && !ctx.enEdition) {
    const [fond, encre] = CARTONS[choisi];
    return [
      el(
        "div",
        { class: "carton-plein", style: { background: fond, color: encre }, role: "status" },
        el("span", { class: "carton-libelle" }, a.options[choisi]),
        el("span", { class: "carton-consigne" }, "Levez votre téléphone !"),
        a.ouverte
          ? el(
              "button",
              { type: "button", class: "bouton-secondaire", "data-focus": "modifier", onclick: () => ctx.editer(true) },
              "Changer de carton",
            )
          : el("span", { class: "carton-consigne" }, "Les réponses sont fermées."),
      ),
    ];
  }
  return [
    el("h1", { class: "question" }, a.question),
    el(
      "ul",
      { class: "options", role: "list" },
      a.options.map((texte, i) =>
        el(
          "li",
          {},
          el(
            "button",
            {
              type: "button",
              class: "option carton",
              style: { background: CARTONS[i][0], color: CARTONS[i][1] },
              "aria-pressed": String(choisi === i),
              disabled: !a.ouverte,
              "data-focus": `carton-${i}`,
              onclick: () => {
                ctx.envoyer({ choix: i }, { choix: i });
                ctx.editer(false);
              },
            },
            texte,
          ),
        ),
      ),
    ),
    fermee(a),
  ];
}

// --- Nuage de mots ------------------------------------------------------------

function nuage(a, ctx) {
  const max = a.max_mots ?? 3;
  if (a.ma_reponse?.mots && !ctx.enEdition) {
    return envoye(
      a,
      ctx,
      "Envoyé !",
      a.ma_reponse.mots.join(" · "),
      "Regardez l'écran : vos mots grossissent quand d'autres pensent pareil.",
    );
  }
  const mots = ctx.brouillon("mots", a.ma_reponse?.mots ? [...a.ma_reponse.mots] : []);
  const saisie = ctx.brouillon("saisie", "");
  const plein = mots.length >= max;

  const ajouter = () => {
    const mot = ctx.brouillon("saisie", "").trim().replace(/\s+/g, " ");
    if (!mot || plein) return;
    if (!mots.some((m) => m.toLowerCase() === mot.toLowerCase())) mots.push(mot);
    ctx.garder("mots", mots);
    ctx.garder("saisie", "");
    ctx.rendre();
  };

  const champ = el("input", {
    id: "mot",
    class: "champ",
    maxlength: "25",
    autocomplete: "off",
    enterkeyhint: "done",
    placeholder: plein ? "C'est complet !" : "Un mot ou une expression",
    value: saisie,
    disabled: plein || !a.ouverte,
    "data-focus": "mot",
    oninput: (ev) => ctx.garder("saisie", ev.target.value),
    onkeydown: (ev) => {
      if (ev.key === "Enter") {
        ev.preventDefault();
        ajouter();
      }
    },
  });

  return [
    el("h1", { class: "question" }, a.question),
    el("p", { class: "aide" }, max > 1 ? `Jusqu'à ${max} mots, 25 caractères chacun.` : "Un seul mot, 25 caractères au plus."),
    el(
      "div",
      { class: "carte" },
      el("label", { for: "mot" }, `Votre mot (${mots.length}/${max})`),
      el(
        "div",
        { class: "ligne-saisie" },
        champ,
        el(
          "button",
          { type: "button", class: "bouton-plus", "aria-label": "Ajouter le mot", disabled: plein || !a.ouverte, onclick: ajouter },
          "+",
        ),
      ),
      el(
        "ul",
        { class: "jetons", role: "list", "aria-label": "Mots choisis" },
        mots.map((m, i) =>
          el(
            "li",
            { class: "jeton" },
            m,
            el(
              "button",
              {
                type: "button",
                "aria-label": `Retirer ${m}`,
                onclick: () => {
                  mots.splice(i, 1);
                  ctx.garder("mots", mots);
                  ctx.rendre();
                },
              },
              "×",
            ),
          ),
        ),
      ),
    ),
    bouton(
      "Envoyer",
      () => {
        if (ctx.brouillon("saisie", "").trim()) ajouter();
        if (mots.length && ctx.envoyer({ mots })) ctx.editer(false);
      },
      { disabled: !a.ouverte || (mots.length === 0 && !saisie.trim()), "data-focus": "envoyer" },
    ),
    fermee(a),
  ];
}

// --- Échelle --------------------------------------------------------------------

function echelle(a, ctx) {
  const choisi = ctx.local.valeur ?? a.ma_reponse?.valeur ?? null;
  const valeurs = Array.from({ length: a.max }, (_, i) => i + 1);
  return [
    el("h1", { class: "question" }, a.question),
    el(
      "div",
      { class: `grille-echelle n${a.max}`, role: "group", "aria-label": `Votre note de 1 à ${a.max}` },
      valeurs.map((v) =>
        el(
          "button",
          {
            type: "button",
            class: "case-echelle",
            "aria-pressed": String(choisi === v),
            disabled: !a.ouverte,
            "data-focus": `valeur-${v}`,
            "aria-label": `${v} sur ${a.max}`,
            onclick: () => ctx.envoyer({ valeur: v }, { valeur: v }),
          },
          String(v),
        ),
      ),
    ),
    a.libelle_min || a.libelle_max
      ? el(
          "div",
          { class: "libelles-echelle" },
          el("span", {}, a.libelle_min ? `1 : ${a.libelle_min}` : ""),
          el("span", {}, a.libelle_max ? `${a.max} : ${a.libelle_max}` : ""),
        )
      : null,
    fermee(a) ??
      (choisi !== null
        ? statut(`C'est noté : ${choisi} sur ${a.max}. Vous pouvez changer.`)
        : el("p", { class: "aide" }, "Touchez une note.")),
  ];
}

// --- Météo à deux axes ----------------------------------------------------------

function meteo(a, ctx) {
  const point = ctx.brouillon("point", a.ma_reponse?.point ? { ...a.ma_reponse.point } : { x: 50, y: 50 });
  if (a.ma_reponse?.point && !ctx.enEdition) {
    return envoye(a, ctx, "Envoyé !", humeur(point), "Votre point rejoint ceux de la salle, sans votre nom.");
  }
  // Pas de nouveau rendu pendant un glissement : on déplace le point et les curseurs directement.
  const marqueur = el("span", { class: "point-meteo" });
  const curseurs = {};
  const placer = (x, y) => {
    point.x = Math.max(0, Math.min(100, Math.round(x)));
    point.y = Math.max(0, Math.min(100, Math.round(y)));
    ctx.garder("point", point);
    marqueur.style.left = `${point.x}%`;
    marqueur.style.bottom = `${point.y}%`;
    for (const [cle, input] of Object.entries(curseurs)) {
      if (document.activeElement !== input) input.value = String(point[cle]);
      input.setAttribute("aria-valuetext", `${point[cle]} sur 100`);
    }
  };
  const depuisPointeur = (ev) => {
    if (!a.ouverte) return;
    const r = ev.currentTarget.getBoundingClientRect();
    placer(((ev.clientX - r.left) / r.width) * 100, (1 - (ev.clientY - r.top) / r.height) * 100);
  };
  const carre = el(
    "div",
    {
      class: "carre-meteo",
      "aria-hidden": "true",
      onpointerdown: (ev) => {
        ev.currentTarget.setPointerCapture?.(ev.pointerId);
        depuisPointeur(ev);
      },
      onpointermove: (ev) => {
        if (ev.buttons) depuisPointeur(ev);
      },
    },
    el("span", { class: "axe axe-x" }, "Humeur →"),
    el("span", { class: "axe axe-y" }, "Énergie →"),
    marqueur,
  );
  const curseur = (id, nom, cle, gauche, droite) => {
    const input = el("input", {
      id,
      type: "range",
      min: "0",
      max: "100",
      step: "1",
      disabled: !a.ouverte,
      "data-focus": id,
      oninput: (ev) => (cle === "x" ? placer(+ev.target.value, point.y) : placer(point.x, +ev.target.value)),
    });
    curseurs[cle] = input;
    return el(
      "div",
      { class: "curseur" },
      el("label", { for: id }, nom),
      input,
      el("div", { class: "bornes" }, el("span", {}, gauche), el("span", {}, droite)),
    );
  };
  const elements = [
    el("h1", { class: "question" }, a.question),
    el("p", { class: "aide" }, "Touchez le carré, ou réglez les deux curseurs."),
    carre,
    curseur("humeur", "Humeur", "x", "Maussade", "Radieuse"),
    curseur("energie", "Énergie", "y", "À plat", "En forme"),
    bouton("Envoyer", () => ctx.envoyer({ point }) && ctx.editer(false), { disabled: !a.ouverte, "data-focus": "envoyer" }),
    fermee(a),
  ];
  placer(point.x, point.y);
  return elements;
}

function humeur(p) {
  const h = p.x < 34 ? "humeur maussade" : p.x > 66 ? "humeur radieuse" : "humeur moyenne";
  const e = p.y < 34 ? "énergie basse" : p.y > 66 ? "pleine énergie" : "énergie moyenne";
  return `${h}, ${e}`;
}

// --- Pari de certitude ------------------------------------------------------------

function pari(a, ctx) {
  const mien = a.ma_reponse?.pari ?? null;
  if (a.revelee && a.bonne !== undefined) {
    const juste = mien && mien.choix === a.bonne;
    return [
      el("h1", { class: "question" }, a.question),
      el(
        "div",
        { class: "envoye", role: "status" },
        el("p", { class: "envoye-titre" }, "La bonne réponse"),
        el("p", { class: "rappel" }, `${lettre(a.bonne)} · ${a.options[a.bonne]}`),
        mien
          ? el(
              "p",
              {},
              juste ? "✓ Vous aviez juste" : "✗ Ce n'était pas votre réponse",
              ` (${CERTITUDES[mien.certitude].toLowerCase()}).`,
            )
          : el("p", { class: "aide" }, "Vous n'aviez pas parié."),
      ),
    ];
  }
  if (mien && !ctx.enEdition) {
    return envoye(
      a,
      ctx,
      "Pari enregistré !",
      `${lettre(mien.choix)} · ${a.options[mien.choix]} — ${CERTITUDES[mien.certitude]}`,
      "Les yeux sur l'écran : la répartition de la salle arrive.",
    );
  }
  const brouillon = ctx.brouillon("pari", mien ? { ...mien } : { choix: null, certitude: null });
  const maj = (champ, v) => {
    brouillon[champ] = v;
    ctx.garder("pari", brouillon);
    ctx.rendre();
  };
  const items = a.options.map((t, i) => [pastilleOption(i), el("span", { class: "texte" }, t)]);
  return [
    el("h1", { class: "question" }, a.question),
    choix(items, brouillon.choix, (i) => maj("choix", i), { desactive: !a.ouverte }),
    el("p", { class: "sous-question", id: "certitude" }, "Et vous en êtes sûr·e ?"),
    el(
      "div",
      { class: "certitudes", role: "group", "aria-labelledby": "certitude" },
      CERTITUDES.map((nom, i) =>
        el(
          "button",
          {
            type: "button",
            class: "case-certitude",
            "aria-pressed": String(brouillon.certitude === i),
            disabled: !a.ouverte || brouillon.choix === null,
            "data-focus": `certitude-${i}`,
            onclick: () => maj("certitude", i),
          },
          nom,
        ),
      ),
    ),
    bouton(
      "Je parie !",
      () => {
        if (ctx.envoyer({ pari: { choix: brouillon.choix, certitude: brouillon.certitude } })) ctx.editer(false);
      },
      { disabled: !a.ouverte || brouillon.choix === null || brouillon.certitude === null, "data-focus": "envoyer" },
    ),
    fermee(a),
  ];
}

// --- ROTI et mot de la fin ----------------------------------------------------------

function roti(a, ctx) {
  const mien = a.ma_reponse?.roti ?? null;
  if (mien && !ctx.enEdition) {
    return envoye(
      a,
      ctx,
      "Merci !",
      `${mien.valeur} doigt${mien.valeur > 1 ? "s" : ""}${mien.mot ? ` · « ${mien.mot} »` : ""}`,
      null,
    );
  }
  const brouillon = ctx.brouillon("roti", mien ? { ...mien, mot: mien.mot ?? "" } : { valeur: null, mot: "" });
  const legendes = ["Temps perdu", "Bof", "Moyen", "Utile", "Très utile", "Excellent"];
  return [
    el("h1", { class: "question" }, a.question),
    el(
      "div",
      { class: "grille-roti", role: "group", "aria-label": "Combien de doigts, de 0 à 5 ?" },
      legendes.map((legende, v) =>
        el(
          "button",
          {
            type: "button",
            class: "case-echelle",
            "aria-pressed": String(brouillon.valeur === v),
            "aria-label": `${v} : ${legende}`,
            disabled: !a.ouverte,
            "data-focus": `roti-${v}`,
            onclick: () => {
              brouillon.valeur = v;
              ctx.garder("roti", brouillon);
              ctx.rendre();
            },
          },
          el("strong", {}, String(v)),
          el("small", {}, legende),
        ),
      ),
    ),
    el("label", { for: "mot-fin" }, "Un mot pour finir ", el("span", { class: "aide" }, "(facultatif)")),
    el("input", {
      id: "mot-fin",
      class: "champ",
      maxlength: "25",
      autocomplete: "off",
      value: brouillon.mot,
      disabled: !a.ouverte,
      "data-focus": "mot-fin",
      oninput: (ev) => {
        brouillon.mot = ev.target.value;
        ctx.garder("roti", brouillon);
      },
    }),
    bouton(
      "Envoyer",
      () => {
        const mot = brouillon.mot.trim();
        if (ctx.envoyer({ roti: { valeur: brouillon.valeur, ...(mot ? { mot } : {}) } })) ctx.editer(false);
      },
      { disabled: !a.ouverte || brouillon.valeur === null, "data-focus": "envoyer" },
    ),
    fermee(a),
  ];
}

// --- Question ouverte modérée -------------------------------------------------------

function zoneTexte(id, valeur, max, desactive, surSaisie) {
  const compteur = el("span", { class: "aide compteur-car", id: `${id}-compte`, "aria-live": "polite" }, `${valeur.length}/${max}`);
  const zone = el("textarea", {
    id,
    class: "champ zone",
    maxlength: String(max),
    rows: "4",
    disabled: desactive,
    "data-focus": id,
    "aria-describedby": `${id}-compte`,
    oninput: (ev) => {
      compteur.textContent = `${ev.target.value.length}/${max}`;
      surSaisie(ev.target.value);
    },
  });
  zone.value = valeur;
  return [zone, compteur];
}

function ouverte(a, ctx) {
  if (a.ma_reponse?.texte && !ctx.enEdition) {
    return envoye(a, ctx, "Envoyé !", `« ${a.ma_reponse.texte} »`, "L'animateur choisit les messages projetés. Votre nom n'apparaît nulle part.");
  }
  const texte = ctx.brouillon("texte", a.ma_reponse?.texte ?? "");
  return [
    el("h1", { class: "question" }, a.question),
    el("label", { for: "texte" }, "Votre réponse"),
    ...zoneTexte("texte", texte, 280, !a.ouverte, (v) => ctx.garder("texte", v)),
    bouton(
      "Envoyer",
      () => {
        const t = ctx.brouillon("texte", "").trim();
        if (t && ctx.envoyer({ texte: t })) ctx.editer(false);
      },
      { disabled: !a.ouverte, "data-focus": "envoyer" },
    ),
    fermee(a),
  ];
}

// --- Questions du public -------------------------------------------------------------

function questions(a, ctx) {
  const d = a.donnees ?? { validees: [], miennes: [], restantes: 0 };
  // Le brouillon n'est vidé qu'une fois la question arrivée (en cas d'erreur, il reste).
  if (ctx.local.envoiQuestion !== undefined && d.miennes.length > ctx.local.envoiQuestion) {
    ctx.garder("question", "");
    delete ctx.local.envoiQuestion;
  }
  const texte = ctx.brouillon("question", "");
  const formulaire =
    d.restantes > 0 && a.ouverte
      ? [
          el("label", { for: "question" }, `Votre question (encore ${d.restantes})`),
          ...zoneTexte("question", texte, 200, false, (v) => ctx.garder("question", v)),
          bouton("Poser ma question", () => {
            const t = ctx.brouillon("question", "").trim();
            if (t && ctx.envoyer({ question: t })) ctx.local.envoiQuestion = d.miennes.length;
          }, { "data-focus": "envoyer" }),
        ]
      : [fermee(a) ?? el("p", { class: "aide" }, "Vous avez posé vos 3 questions. Votez pour celles des autres !")];

  return [
    el("h1", { class: "question" }, a.question),
    ...formulaire,
    d.miennes.length
      ? el(
          "section",
          { class: "carte", "aria-label": "Mes questions" },
          el("h2", {}, "Mes questions"),
          el(
            "ul",
            { class: "liste-simple", role: "list" },
            d.miennes.map((q) =>
              el("li", {}, q.texte, el("span", { class: `etat-question${q.valide ? " ok" : ""}` }, q.valide ? "✓ visible" : "en attente de l'animateur")),
            ),
          ),
        )
      : null,
    el("h2", {}, "Votez pour les questions"),
    d.validees.length
      ? el(
          "ul",
          { class: "options", role: "list" },
          d.validees.map((q) =>
            el(
              "li",
              {},
              el(
                "button",
                {
                  type: "button",
                  class: "option vote",
                  "aria-pressed": String(q.mon_vote),
                  disabled: !a.ouverte,
                  "data-focus": `vote-${q.rang}`,
                  "aria-label": `${q.texte} — ${q.votes} vote${q.votes > 1 ? "s" : ""}${q.mon_vote ? ", j'ai voté" : ""}`,
                  onclick: () => ctx.envoyer({ vote: { question: q.rang, on: !q.mon_vote } }),
                },
                el("span", { class: "compte-votes", "aria-hidden": "true" }, `▲ ${q.votes}`),
                el("span", { class: "texte" }, q.texte),
                q.mon_vote ? el("span", { class: "coche", "aria-hidden": "true" }, "✓") : null,
              ),
            ),
          ),
        )
      : el("p", { class: "aide" }, "Aucune question validée pour l'instant."),
  ];
}

// --- Remise en ordre -------------------------------------------------------------------
// Les éléments arrivent mélangés ; la proposition est la suite des positions affichées.

function ordre(a, ctx) {
  const n = a.elements.length;
  const minuterie = a.reste_ms !== undefined ? elementMinuterie("minuterie-tel") : null;
  const mien = a.ma_reponse?.ordre ?? null;

  if (a.revelee && a.solution) {
    const bien = mien ? a.solution.filter((p, k) => mien[k] === p).length : null;
    return [
      el("h1", { class: "question" }, a.question),
      el(
        "div",
        { class: "envoye", role: "status" },
        el("p", { class: "envoye-titre" }, "Le bon ordre"),
        bien === null
          ? el("p", { class: "aide" }, "Vous n'aviez pas validé de proposition.")
          : el("p", { class: "rappel" }, bien === n ? "Bravo, tout est juste !" : `${bien} élément${bien > 1 ? "s" : ""} sur ${n} à la bonne place.`),
      ),
      el(
        "ol",
        { class: "liste-ordre", role: "list" },
        a.solution.map((p, k) =>
          el(
            "li",
            { class: "ligne-ordre" },
            el("span", { class: "numero-ordre", "aria-hidden": "true" }, String(k + 1)),
            carteOrdre(a.elements[p]),
            mien ? el("span", { class: `marque-ordre ${mien[k] === p ? "juste" : "faux"}` }, mien[k] === p ? "✓ bien placé" : "✗") : null,
          ),
        ),
      ),
    ];
  }

  if (mien && !ctx.enEdition) {
    return [
      minuterie,
      el("h1", { class: "question" }, a.question),
      el("div", { class: "envoye", role: "status" }, el("span", { class: "coche-ronde", "aria-hidden": "true" }, "✓"), el("p", { class: "envoye-titre" }, "Proposition validée !"), el("p", { class: "aide" }, "La correction arrive à l'écran.")),
      el(
        "ol",
        { class: "liste-ordre compacte", role: "list" },
        mien.map((p, k) => el("li", { class: "ligne-ordre" }, el("span", { class: "numero-ordre" }, String(k + 1)), carteOrdre(a.elements[p]))),
      ),
      a.ouverte ? el("button", { type: "button", class: "bouton-secondaire", "data-focus": "modifier", onclick: () => ctx.editer(true) }, "Modifier") : fermee(a),
    ];
  }

  const courant = ctx.brouillon("ordre", mien ? [...mien] : [...Array(n).keys()]);
  const liste = el("ol", { class: "liste-ordre", role: "list", "aria-label": "Éléments à remettre dans l'ordre" });
  const annonce = el("p", { class: "visuellement-cache", "aria-live": "polite" });

  const lire = () => [...liste.children].map((li) => Number(li.dataset.pos));
  const garder = () => {
    const nouvel = lire();
    courant.splice(0, n, ...nouvel);
    ctx.garder("ordre", courant);
    [...liste.children].forEach((li, k) => (li.querySelector(".numero-ordre").textContent = String(k + 1)));
  };
  const deplacer = (li, sens) => {
    const cible = sens < 0 ? li.previousElementSibling : li.nextElementSibling?.nextElementSibling ?? null;
    if (sens < 0 && !li.previousElementSibling) return;
    if (sens > 0 && !li.nextElementSibling) return;
    liste.insertBefore(li, cible);
    garder();
    const rang = [...liste.children].indexOf(li) + 1;
    annonce.textContent = `${a.elements[Number(li.dataset.pos)].texte} : position ${rang} sur ${n}.`;
  };

  for (const p of courant) {
    const e = a.elements[p];
    const poignee = el("button", {
      type: "button",
      class: "poignee",
      "aria-label": `Déplacer « ${e.texte} » (flèches haut et bas)`,
      disabled: !a.ouverte,
      "data-focus": `poignee-${p}`,
    }, "⠿");
    const li = el(
      "li",
      { class: "ligne-ordre", "data-pos": String(p) },
      poignee,
      el("span", { class: "numero-ordre", "aria-hidden": "true" }, ""),
      carteOrdre(e),
      el(
        "span",
        { class: "fleches" },
        el("button", { type: "button", class: "fleche", "aria-label": `Monter « ${e.texte} »`, disabled: !a.ouverte, "data-focus": `monter-${p}`, onclick: () => deplacer(li, -1) }, "↑"),
        el("button", { type: "button", class: "fleche", "aria-label": `Descendre « ${e.texte} »`, disabled: !a.ouverte, "data-focus": `descendre-${p}`, onclick: () => deplacer(li, 1) }, "↓"),
      ),
    );
    poignee.addEventListener("keydown", (ev) => {
      if (ev.key === "ArrowUp" || ev.key === "ArrowDown") {
        ev.preventDefault();
        deplacer(li, ev.key === "ArrowUp" ? -1 : 1);
        poignee.focus();
      }
    });
    // Glisser-déposer au doigt ou à la souris, par la poignée (la page défile ailleurs).
    poignee.addEventListener("pointerdown", (ev) => {
      if (!a.ouverte) return;
      ev.preventDefault();
      poignee.setPointerCapture(ev.pointerId);
      ctx.bloquer(true);
      li.classList.add("souleve");
      let depart = ev.clientY;
      const suivre = (m) => {
        const freres = [...liste.children];
        const i = freres.indexOf(li);
        const avant = li.offsetTop;
        const suiv = freres[i + 1];
        const prec = freres[i - 1];
        if (suiv && m.clientY > suiv.getBoundingClientRect().top + suiv.offsetHeight / 2) {
          liste.insertBefore(suiv, li);
        } else if (prec && m.clientY < prec.getBoundingClientRect().top + prec.offsetHeight / 2) {
          liste.insertBefore(li, prec);
        }
        depart += li.offsetTop - avant;
        li.style.transform = `translateY(${m.clientY - depart}px)`;
      };
      const lacher = () => {
        poignee.removeEventListener("pointermove", suivre);
        li.classList.remove("souleve");
        li.style.transform = "";
        garder();
        ctx.bloquer(false);
      };
      poignee.addEventListener("pointermove", suivre);
      poignee.addEventListener("pointerup", lacher, { once: true });
      poignee.addEventListener("pointercancel", lacher, { once: true });
    });
    liste.append(li);
  }
  [...liste.children].forEach((li, k) => (li.querySelector(".numero-ordre").textContent = String(k + 1)));

  return [
    minuterie,
    el("h1", { class: "question" }, a.question),
    el("p", { class: "aide" }, "Faites glisser les éléments par la poignée ⠿ (ou utilisez les flèches), puis validez."),
    liste,
    annonce,
    el("button", {
      type: "button",
      class: "bouton-principal",
      disabled: !a.ouverte,
      "data-focus": "envoyer",
      onclick: () => {
        if (ctx.envoyer({ ordre: lire() })) ctx.editer(false);
      },
    }, "Valider ma proposition"),
    fermee(a),
  ];
}

export const RENDUS = { sondage, cartons, nuage, echelle, meteo, pari, roti, ouverte, questions, ordre };

