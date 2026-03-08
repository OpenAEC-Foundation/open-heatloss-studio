# TODO

## Deployment
- [x] Authentik application aanmaken voor `warmteverlies`
- [x] `frontend/.env.production` OIDC vars geactiveerd
- [x] OIDC login flow end-to-end testen
- [x] favicon toevoegen
- [x] CI/CD GitHub Actions workflow (vereist secrets: `DEPLOY_HOST`, `DEPLOY_USER`, `DEPLOY_SSH_KEY`)
- [x] Error handling: globale error banner in AppShell

## App — huidig
- [x] OIDC login/logout testen op productie
- [x] Projecten opslaan/laden testen (vereist OIDC login)
- [x] Meer vertrekken/elementen in de UI testen
- [ ] Materialen: inline bewerken van bestaande materialen
- [ ] Materialen: lambda nat (lambdaWet) waarden invullen voor standaardmaterialen
- [ ] Materialen: zoekwoorden (keywords) meegeven bij custom materialen

## Roadmap — EASY modus (snelle berekening)
- [ ] BAG-data import: oppervlak, bouwjaar, adres ophalen via postcode + huisnummer
- [ ] Referentie-U-waarden per bouwjaar (forfaitair): automatisch constructies genereren
- [ ] BENG/UNIEC rapport import (EP-Online koppeling)
- [ ] Quick-calc wizard: 5-10 min berekening met minimale invoer

## Roadmap — PRO features
- [ ] ISSO 53 (utiliteitsgebouwen) naast ISSO 51 (woningen)
- [ ] ISSO 57 (vloerverwarming dimensionering)
- [ ] Vloerverwarming: lus-afstand, vloerbedekking, aanvoertemperatuur → vermogen + oppervlaktetemperatuur
- [ ] Radiatorselectie: merken-database (Radson, Henrad, Vasco, Jaga, Stelrad)
- [ ] Hydraulische balancering: circuits, leidinglengtes, debieten
- [ ] PDF rapport: complete rapportage conform ISSO format

## Roadmap — platform
- [x] 2D modeller: canvas met pan/zoom, grid, ruimte-polygonen, wanden, ramen
- [x] 3D modeller: ThatOpen viewer met geëxtrudeerde ruimtes, wanden, ramen
- [x] Ribbon toolbar (Revit-stijl) met Model/Annotatie/Beeld/Invoegen tabs
- [x] IFCX (IFC5 JSON) als intern bestandsformaat
- [x] ISSO51 namespace extensies voor berekeningen in IFCX
- [x] 3D: ramen vereenvoudigd, wanden bijna wit, vloeren gefixt
- [x] Teken-functionaliteit: rechthoek/polygoon tekenen met snap + preview
- [x] Raam plaatsen tool: klik op wand
- [x] Constructiebibliotheek koppelen aan wanden/vloeren/daken (inline picker)
- [x] Onderlegger import (afbeelding als achtergrond)
- [x] Zustand store met undo/redo, bewerkbare eigenschappen
- [ ] Ruimte verplaatsen/resizen + vertex editing
- [ ] PDF onderlegger + DWG import
- [ ] IFC import/export via ThatOpen IfcLoader
- [ ] Schuine daken en dakkapellen
- [ ] Modeller data ↔ project store synchronisatie
- [ ] IFC/BIM import: constructies en vertrekken uit IFC-model halen
- [ ] API voor derden: berekening als service voor andere tools
- [ ] Multi-user: projecten delen, rollen (engineer/reviewer)
- [ ] Template-projecten: veelvoorkomende woningtypes als startpunt
