// Client WebSocket partagé par les trois interfaces.
// Se reconnecte seul (attente croissante de 0,5 à 5 s), dit « bonjour » à chaque
// ouverture et envoie un « ping » toutes les 15 s pour garder la connexion.

const ATTENTE_MIN = 500;
const ATTENTE_MAX = 5000;
const INTERVALLE_PING = 15000;

/**
 * @param {object} p
 * @param {() => object} p.bonjour  champs du message « bonjour » (rappelé à chaque connexion)
 * @param {(m: object) => void} p.surMessage
 * @param {(connecte: boolean) => void} [p.surConnexion]
 * @param {(message: string) => void} [p.surErreurFatale]  clé ou code refusé : plus de reconnexion
 */
export function connecter({ bonjour, surMessage, surConnexion = () => {}, surErreurFatale = () => {} }) {
  let ws = null;
  let attente = ATTENTE_MIN;
  let ping = null;
  let arrete = false;

  function ouvrir() {
    const protocole = location.protocol === "https:" ? "wss" : "ws";
    ws = new WebSocket(`${protocole}://${location.host}/ws`);
    ws.addEventListener("open", () => {
      attente = ATTENTE_MIN;
      ws.send(JSON.stringify({ t: "bonjour", ...bonjour() }));
      surConnexion(true);
      ping = setInterval(() => envoyer({ t: "ping" }), INTERVALLE_PING);
    });
    ws.addEventListener("message", (ev) => {
      let m;
      try {
        m = JSON.parse(ev.data);
      } catch {
        return;
      }
      if (m.t === "erreur" && m.fatale) {
        arrete = true;
        surErreurFatale(m.message);
        return;
      }
      surMessage(m);
    });
    ws.addEventListener("close", () => {
      clearInterval(ping);
      if (arrete) return;
      surConnexion(false);
      setTimeout(ouvrir, attente);
      attente = Math.min(attente * 2, ATTENTE_MAX);
    });
  }

  function envoyer(message) {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(message));
      return true;
    }
    return false;
  }

  ouvrir();
  return { envoyer };
}
