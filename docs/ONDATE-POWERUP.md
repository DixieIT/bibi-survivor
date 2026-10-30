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

Tank e maghi riutilizzano lo sprite scheletrico disponibile, distinguendosi per colore e statistiche.

## Ricompense

Dopo ogni ondata completata il gioco offre tre potenziamenti tra cui scegliere. I potenziamenti scelti restano attivi per la partita.

- **Dash:** scatto direzionale con invulnerabilità durante lo scatto.
- **Life steal:** una parte dei danni inflitti viene convertita in vita.
- **Vortice di spadate:** attacco radiale che colpisce in tutte le direzioni. Al livello iniziale infligge il doppio del danno base della spada; i miglioramenti riducono il costo in stamina o aumentano i danni.
- **Stamina:** aumenta la stamina massima.

## Controlli e interfaccia

Durante la scelta del potenziamento il combattimento si ferma. I tasti 1, 2 e 3 selezionano una proposta. Il tasto E attiva il dash e Q attiva il vortice, quando disponibile. L'interfaccia mostra l'ondata, la fase di pausa, le proposte e i livelli dei potenziamenti.

## Regole

- Il giocatore non riceve una ricompensa se è morto prima del completamento dell'ondata.
- Le ricompense di fine ondata si aggiungono ai cuori raccolti e alla progressione esistente di slash e scudo per uccisioni.
- La progressione delle composizioni continua oltre l'ondata 6, aumentando i nemici speciali senza rimuovere il limite simultaneo.
