# CanBankX

[![CI](https://github.com/amir-hammar/can-bank-x/actions/workflows/ci.yml/badge.svg)](https://github.com/amir-hammar/can-bank-x/actions/workflows/ci.yml)

## Architecture locale

- Cible de déploiement: machine locale uniquement
- Orchestration: Docker Compose
- Réseau Docker partagé: `can-bank-x-network`
- Un conteneur PostgreSQL avec 3 bases dédiées:
  - `canbankx_user`
  - `canbankx_account`
  - `canbankx_transfer`

Services:
- `website`: `http://localhost:8083`
- `user-service`
- `account-service`
- `transfer-service`
- `postgres`: `localhost:5432`
- `redis`: `localhost:6379`
- `api-gateway` (KrakenD): `http://localhost:8080`
- `keycloak`: `http://localhost:8082`
- `prometheus`: `http://localhost:9090`
- `grafana`: `http://localhost:3001`
- `seed` (application des migrations)

## Déploiement (commande unique)

- `sh can-bank-x-main/deploy/deploy.sh`

What it does:
- creates docker network
- stops previous stack
- rebuilds images
- runs `docker compose up -d --build`


## Collection Postman

Fichiers:
- Collection: `docs/collections/can-bank-x.postman_collection.json`
- Environnement: `docs/collections/can-bank-x.local.postman_environment.json`

Dossiers principaux:
- `CU-01 Inscription et vérification d'identité (KYC)`
- `CU-02 Authentification et MFA`
- `CU-03 Ouverture d'un compte bancaire`
- `CU-04 Consultation des soldes et historiques`
- `CU-05 Virement bancaire`

Exécution recommandée:
1. Importer la collection et l'environnement dans Postman.
2. Exécuter les cas d'utilisation dans l'ordre.

### CU-01 Pas à pas (Inscription et KYC)

1. Ouvrir le dossier `CU-01 Inscription et vérification d'identité (KYC)`.
2. Exécuter `01.01 Ouvrir Page Inscription`.
3. Dans l'onglet `Authorization` de la requête Postman, il faut aller en bas de la page et:
- cliquer `Clear Cookies`;
- cliquer `Get New Access Token`.
4. Dans la page Keycloak ouverte, faire défiler vers le bas et choisir `Register`.
5. Remplir les champs d'inscription et soumettre le formulaire.
6. Pour obtenir un KYC `APPROVED` avec le mock, utiliser `Full name = Postman Gateway` (ou `Test`) et `NAS = 123456789`.
7. Copier le `access_token` retourné.
8. Ouvrir `CU-02 / 02.02 Get Profile` et coller le token dans `access_token` dans la page Authorization, en s'assurant qu'il n'y a pas d'espace ou de saut de ligne à la fin (erreur très fréquente).
9. Vérifier ensuite:
- `CU-02 / 02.02 Get Profile`;
- `CU-02 / 02.03 Get KYC Status`.
10. Après la soumission KYC, attendre environ `15` à `20` secondes, puis refaire les requêtes `CU-02 / 02.02 Get Profile` et `CU-02 / 02.03 Get KYC Status` pour obtenir la décision finale (`APPROVED` ou `REJECTED`).

### CU-02 Pas à pas (Authentification et MFA)

1. Ouvrir le dossier `CU-02 Authentification et MFA`.
2. Exécuter `02.01 Ouvrir Page Connexion`.
3. Dans Postman (`Authorization`):
- cliquer `Clear Cookies`;
- cliquer `Get New Access Token`.
4. Se connecter dans Keycloak avec l'utilisateur créé au CU-01, ou avec: `demo.customer` (username) `Passw0rd!123`(password).
5. Si vous utilisez `demo.customer`, il faudra configurer son MFA (OTP) lors de la première connexion.
6. Compléter l'étape OTP/MFA.
7. Copier le `access_token`, puis le coller dans `CU-02 / 02.02 Get Profile`, en s'assurant qu'il n'y a pas d'espace ou de saut de ligne à la fin (erreur très fréquente).
8. Exécuter `02.02 Get Profile` et récupérer le `customer_id`.
9. Exécuter `02.03 Get KYC Status`.
10. Après la soumission KYC, attendre environ `15` à `20` secondes, puis refaire les requêtes `CU-02 / 02.02 Get Profile` et `CU-02 / 02.03 Get KYC Status` pour obtenir la décision finale (`APPROVED` ou `REJECTED`).

### CU-03 Pas à pas (Ouverture d'un compte bancaire)

1. Vérifier qu'un `access_token` valide est déjà disponible.
2. Ouvrir le dossier `CU-03 Ouverture d'un compte bancaire`.
3. Exécuter `03.01 Créer compte CHEQUING`.
4. Dans le body, s'assurer que `customer_id` = `{{account_customer_id}}`.
5. Résultat attendu: `201 Created`.
6. Le script Postman enregistre automatiquement:
- `{{account_id}}`;
- `{{account_customer_id}}`.
7. Exécuter `03.02 Créer compte SAVINGS` pour le même client.
8. Résultat attendu: `201 Created`.

### CU-04 Pas à pas (Consultation des soldes et historiques)

1. Vérifier qu'au moins un compte a été créé dans CU-03.
2. Ouvrir `CU-04 Consultation des soldes et historiques`.
3. Exécuter `04.01 Lister comptes client`.
4. Vérifier que le paramètre `customer_id` utilise `{{account_customer_id}}`.
5. Résultat attendu: `200 OK` avec la liste des comptes.
6. Exécuter `04.02 Consulter solde du compte` avec `{{account_id}}`.
7. Résultat attendu: `200 OK` avec `available_balance`, `ledger_balance`, `currency`.
8. Exécuter `04.03 Consulter historique des transactions`.
9. Résultat attendu: `200 OK` avec l'historique des virements.
10. Note: l'historique peut être vide tant que le CU-05 (virement) n'a pas été exécuté.

### CU-05 Pas à pas (Virement bancaire)

Préparation:
- créer deux clients distincts (expéditeur et bénéficiaire);
- chaque client doit avoir terminé CU-01, CU-02 et CU-03;
- noter le `username` du bénéficiaire.

Étapes:
1. Utiliser le `access_token` de l'expéditeur.
2. Ouvrir `CU-05 Virement bancaire`.
3. Exécuter `05.01 Effectuer virement par username`.
4. Dans le body:
- `customer_id` et `from_account_id` = compte de l'expéditeur;
- `beneficiary_username` = nom d'utilisateur du bénéficiaire.
5. Résultat attendu: `201 Created`, avec stockage de `{{transfer_id}}`.
6. Exécuter `05.02 Consulter détails du virement` pour valider le transfert.
7. Exécuter `05.03 Lister historique client` pour valider l'historique côté expéditeur.

Note:
- la méthode recommandée est le virement par `beneficiary_username`;
- la méthode legacy par `to_account_id` peut rester disponible selon la collection.

## Artéfacts API

- OpenAPI user-service: `docs/openapi-user-service.yaml`
- OpenAPI account-service: `docs/openapi-account-service.yaml`
- OpenAPI transfer-service: `docs/openapi-transfer-service.yaml`
- Collection Postman: `docs/collections`