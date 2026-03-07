# TODO

## Deployment
- [x] Authentik application aanmaken voor `warmteverlies`
- [x] `frontend/.env.production` OIDC vars geactiveerd
- [ ] OIDC login flow end-to-end testen
- [x] favicon toevoegen
- [x] CI/CD GitHub Actions workflow (vereist secrets: `DEPLOY_HOST`, `DEPLOY_USER`, `DEPLOY_SSH_KEY`)
- [x] Error handling: globale error banner in AppShell

## App — huidig
- [ ] OIDC login/logout testen op productie
- [ ] Projecten opslaan/laden testen (vereist OIDC login)
- [ ] Meer vertrekken/elementen in de UI testen

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
- [ ] 3D modeller: visueel model van het gebouw met vertrekken en constructies
- [ ] IFC/BIM import: constructies en vertrekken uit IFC-model halen
- [ ] API voor derden: berekening als service voor andere tools
- [ ] Multi-user: projecten delen, rollen (engineer/reviewer)
- [ ] Template-projecten: veelvoorkomende woningtypes als startpunt
