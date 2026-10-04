// Vérifie que chaque adresse du catalogue répond. Un dépôt privé, déplacé ou
// sous licence à accepter répond 401 ou 404 : l'installation échouerait chez
// l'utilisateur, pas chez nous.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const chemin = join(dirname(fileURLToPath(import.meta.url)), "..", "dist", "marketplace.json");
const catalogue = JSON.parse(readFileSync(chemin, "utf8"));

const adresses = new Set();
for (const modele of catalogue.models) {
  for (const variante of modele.variants) {
    adresses.add(variante.tag);
    for (const d of variante.downloads ?? []) adresses.add(d.url);
  }
}

let echecs = 0;
for (const url of [...adresses].sort()) {
  let code = 0;
  try {
    code = (await fetch(url, { method: "HEAD", redirect: "follow" })).status;
  } catch (e) {
    console.error(`  ${url} : ${e.message}`);
  }
  console.log(`${code === 200 ? "ok " : "ÉCHEC"} ${code} ${url.split("/").pop()}`);
  if (code !== 200) echecs++;
}
if (echecs) {
  console.error(`${echecs} adresse(s) du catalogue ne répondent pas.`);
  process.exit(1);
}
console.log(`${adresses.size} adresses vérifiées.`);
