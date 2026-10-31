# Specifica: ondate e power-up

## Progressione delle ondate

La partita procede per ondate finite. Ogni ondata genera un contingente di nemici; la successiva inizia dopo aver eliminato tutti i nemici del contingente e una breve pausa. Il limite di nemici contemporaneamente presenti resta attivo.

| Ondata | Composizione e difficoltà |
|---|---|
| 1 | Scheletri melee. |
| 2 | Scheletri melee e arcieri. |
| 3 | Più arcieri rispetto all'ondata 2. |
| 4 | Gli arcieri scagliano salve di più frecce. |
| 5 | Compaiono tank e maghi scheletri; i maghi lanciano sfere di fuoco in più direzioni. |
| 6+ | Aumenta progressivamente la presenza di tank e maghi. |
| 7+ | Compaiono gli sciamani (uno in più ogni due ondate): da lontano alzano un braccio al cielo e fanno cadere fiamme in più punti, una sotto il giocatore e le altre in diverse direzioni attorno a loro, ciascuna preceduta da un segnale a terra. |
| 8+ | Le sfere dei maghi sono di più e, dopo essere uscite, restano a girare intorno al mago per qualche secondo. |

Le sfere di fuoco dei maghi attraversano i muri e i maghi lanciano anche senza vedere il giocatore, così nascondersi dietro un ostacolo non basta.

Tank e maghi riutilizzano lo sprite scheletrico disponibile, distinguendosi per colore, statistiche e un accessorio: i tank portano un elmetto, i maghi un libro (aperto mentre lanciano).

## Ricompense

Dopo ogni ondata completata il gioco offre tre potenziamenti tra cui scegliere. I potenziamenti scelti restano attivi per la partita.

- **Slash:** livello 2 colpi più lunghi e alti, livello 3 danno doppio, livello 4 ogni colpo lancia un'onda. Prima si otteneva con le uccisioni.
- **Scudo:** uno scudo che assorbe 2 colpi al primo livello, uno in più per livello, ricaricato all'inizio di ogni ondata. Prima si otteneva ogni 15 uccisioni.
- **Dash:** scatto direzionale con invulnerabilità durante lo scatto.
- **Life steal:** ogni colpo andato a segno ha una probabilità di curare 1 HP (6% per livello).
- **Vortice di spadate:** attacco radiale che colpisce in tutte le direzioni. Al livello iniziale infligge il doppio del danno base della spada; i miglioramenti riducono il costo in stamina o aumentano i danni.
- **Stamina:** aumenta la stamina massima.

## Controlli e interfaccia

Durante la scelta del potenziamento il combattimento si ferma. I tasti 1, 2 e 3 selezionano una proposta. Il tasto E attiva il dash e Q attiva il vortice, quando disponibile. L'interfaccia mostra l'ondata, la fase di pausa, le proposte e i livelli dei potenziamenti.

## Regole

- Il giocatore non riceve una ricompensa se è morto prima del completamento dell'ondata.
- Slash e scudo non si ottengono più con le uccisioni: sono ricompense di fine ondata come gli altri potenziamenti. I cuori lasciati dai nemici restano.
- La progressione delle composizioni continua oltre l'ondata 6, aumentando i nemici speciali senza rimuovere il limite simultaneo.
- Ogni ondata i nemici diventano gradualmente più forti: più vita, più velocità e attacchi più frequenti, con un tetto su velocità e frequenza.
