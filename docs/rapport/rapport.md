# Rapport technique - CanBankX (Phase 1)

## 1. Introduction

Le projet CanBankX vise la réalisation d’un système bancaire modulaire fondé sur une architecture à microservices. Dans un contexte de génie logiciel, l’objectif est de concevoir une plateforme capable de gérer des parcours métier critiques (authentification client, consultation de comptes, historique et exécution de virements), tout en maintenant des propriétés non fonctionnelles mesurables.

L’objectif principal de la Phase 1 est double :
- mettre en place une architecture distribuée opérationnelle ;
- instrumenter la plateforme pour observer son comportement sous charge et valider des cibles de performance.

Dans un domaine bancaire, l’architecture, la performance et l’observabilité sont essentielles. Une architecture claire facilite l’évolution et l’isolation des responsabilités métier. La performance conditionne l’expérience utilisateur et la robustesse opérationnelle. L’observabilité fournit une preuve objective du comportement du système via des métriques de latence, débit, erreurs et saturation, nécessaires à la prise de décision technique.

## 2. Architecture du système

CanBankX est implémenté selon une architecture microservices, exposée par une passerelle API. Le flux nominal est le suivant :

`Client -> edge-load-balancer (NGINX) -> API Gateway (KrakenD) -> microservices métier -> PostgreSQL/Redis`

### 2.1 Composants principaux

- `api-gateway` (KrakenD) : point d’entrée logique des API, routage vers les services, validation JWT, propagation d’en-têtes de traçabilité (`X-Trace-Id`, `X-Request-Id`).
- `edge-load-balancer` (NGINX) : point d’entrée public `:8080`, répartition de charge entre plusieurs réplicas de `api-gateway` (stratégie `least_conn`).
- `user-service` : gestion de l’identité applicative et des informations client (`/api/v1/auth/me`, `/api/v1/customers/me`, parcours KYC).
- `account-service` : gestion des comptes, consultation de soldes et liste des comptes.
- `transfer-service` : exécution et consultation des virements ; dépend de `account-service` pour des validations métier.
- `postgres` : persistance relationnelle, avec une base dédiée par service (isolation logique des données).
- `redis` : couche de cache optionnelle pour `account-service` et `transfer-service`.

### 2.2 Rôle des trois services métier

- Service Utilisateur/Client (`user-service`) :
  authentifie l’utilisateur via le jeton, expose le profil client et l’état KYC. Il constitue l’entrée métier des informations identitaires.

- Service Compte (`account-service`) :
  gère les comptes clients (création, consultation, solde, compte par défaut). Il est central pour les opérations de consultation fréquente.

- Service Virement/Transaction (`transfer-service`) :
  gère le cycle de vie des transactions (création d’un virement, consultation d’un virement, historique). Il orchestre des interactions inter-services pour garantir la cohérence métier.

### 2.3 Flux de requêtes

1. Le client envoie une requête HTTP au point d’entrée public `http://localhost:8080`.
2. NGINX (`edge-load-balancer`) sélectionne une instance de `api-gateway`.
3. KrakenD vérifie les contraintes de sécurité (JWT selon les routes) et route vers le service métier cible.
4. Le microservice traite la requête, accède à PostgreSQL (et Redis si activé), puis renvoie la réponse via la passerelle.

Ce découplage permet de faire évoluer indépendamment les services et de tester la scalabilité de façon contrôlée.

## 3. Infrastructure et déploiement

L’infrastructure de Phase 1 est exécutée entièrement dans des conteneurs Docker, avec orchestration via `docker-compose`.

### 3.1 Conteneurisation et orchestration

- Chaque composant majeur (gateway, services, base, cache, observabilité) est encapsulé dans son conteneur.
- Le fichier `docker-compose.yml` décrit dépendances, healthchecks, variables d’environnement et réseau.
- Un réseau unique `can-bank-x-network` assure la communication inter-services.

### 3.2 Réseau et équilibreur

- `edge-load-balancer` expose le port `8080` côté hôte.
- Le pool NGINX cible `api-gateway:8080` et supporte le repartitionnement dynamique lors du scale.
- La stratégie `least_conn` et `proxy_next_upstream` limitent l’impact d’une instance indisponible.

### 3.3 Scalabilité expérimentale (1 à 4 instances)

Le système est exécuté et évalué en configurations :
- 1 instance
- 2 instances
- 3 instances
- 4 instances

Le paramètre `--scale api-gateway=N` permet d’étudier l’impact du load balancing sur la latence, le débit et la stabilité sous charge. Cette matrice est indispensable pour quantifier la montée en charge et identifier les points de saturation.

## 4. Observabilité et monitoring

### 4.1 Pile d’observabilité

- Prometheus : collecte des métriques techniques (services, système, conteneurs, base) et des métriques k6 en remote write.
- Grafana : visualisation et corrélation des indicateurs (dashboards `k6-load-testing`, `system-overview`, etc.).
- Métriques applicatives : exposition de métriques HTTP (latence, débit, codes retour, erreurs métier instrumentées côté tests k6).
- Logs structurés : logs gateway (niveau `INFO`) et logs applicatifs orientés diagnostic (cache hit/miss, avertissements en fallback).

### 4.2 Couverture des 4 Golden Signals

- Latency :
  mesurée notamment par `k6_http_req_duration_p95` et `k6_http_req_duration_p99` (dashboard k6, vue par service).

- Traffic :
  mesuré par `k6_http_reqs_total` et ses dérivés en `req/s` (throughput global et par endpoint/service).

- Errors :
  observés via les statuts HTTP et le compteur `k6_api_error_events_total` (erreurs métier/tests).

- Saturation :
  observée via CPU/mémoire hôte et conteneurs (`node_exporter`, `cadvisor`) et via les connexions PostgreSQL.

Ces quatre signaux couvrent à la fois la qualité de service perçue et la pression exercée sur l’infrastructure.

## 5. Méthodologie des tests de charge

Les tests de performance sont implémentés avec `k6` et exécutés dans le profil Docker `loadtest`. Les métriques sont exportées vers Prometheus (`experimental-prometheus-rw`) puis analysées dans Grafana.

### 5.1 Scénarios retenus

- Consultation de comptes (trafic majoritairement lecture) :
  requêtes de profil utilisateur, liste de comptes, solde, historique de virements.

- Virements (trafic écriture) :
  création d’un virement, puis vérification de l’opération.

- Charge mixte :
  combinaison lectures/écritures (par défaut 70 % lecture, 30 % transfert), représentant un usage bancaire réaliste.

### 5.2 Mesures collectées

Les campagnes mesurent :
- latence (moyenne, p95, p99) ;
- throughput (req/s) ;
- taux d’erreur ;
- saturation (CPU, mémoire, connexions DB).

La campagne comparative principale (`run-benchmarks.sh`) fait varier deux dimensions :
- cache OFF vs cache ON ;
- nombre d’instances 1, 2, 3 et 4.

## 6. Expériences de performance

Deux axes expérimentaux ont été menés.

### 6.1 Cache OFF vs Cache ON

Le cache Redis est activé dans `account-service` et `transfer-service` pour les consultations fréquentes. La comparaison OFF/ON permet de mesurer l’effet de la réduction des accès répétés à PostgreSQL, principalement sur les lectures (solde, listes).

### 6.2 1 vs 2 vs 3 vs 4 instances

La variation du nombre d’instances évalue :
- la capacité de l’équilibreur à répartir la charge ;
- la diminution potentielle de la latence sous pic ;
- le gain de débit global avant apparition d’un nouveau goulot (base de données, réseau, limites CPU).

Ces comparaisons sont essentielles pour caractériser la scalabilité horizontale effective du système.

## 7. Résultats de performance

Cette section analyse les captures Grafana présentes dans `docs/rapport/images`.

### Figure 1 - Cache activé, 1 instance

![Figure 1 - 1 instance avec cache](images/with-cache-1.png)

La Figure 1 présente le comportement de référence avec cache actif et une seule instance gateway. On observe le suivi des métriques de latence (p95/p99), de débit et de consommation CPU/mémoire dans une configuration minimale. Les trois services métier apparaissent dans les séries de throughput/latence, avec une charge encore concentrée sur peu d’instances.

### Figure 2 - Cache activé, 2 instances

![Figure 2 - 2 instances avec cache](images/with-cache-2.png)

La Figure 2 montre l’effet initial du load balancing : la charge est mieux répartie, la courbe de latence p95/p99 se stabilise et le débit soutenable augmente. Les métriques de saturation indiquent une pression plus diffuse sur les conteneurs qu’en configuration 1 instance.

### Figure 3 - Cache activé, 3 instances

![Figure 3 - 3 instances avec cache](images/with-cache-3.png)

La Figure 3 confirme la tendance : la distribution de trafic entre réplicas réduit les pointes de latence, et les valeurs p95/p99 restent plus proches de la moyenne. Cette configuration est typiquement celle où l’on observe un bon compromis entre coût d’infrastructure et stabilité de service.

### Figure 4 - Cache activé, 4 instances

![Figure 4 - 4 instances avec cache](images/with-cache-4.png)

La Figure 4 illustre le régime le plus scalable testé. Le throughput global est le plus élevé de la série avec cache, tandis que l’erreur reste contenue. Les courbes CPU/mémoire montrent que la saturation se déplace progressivement vers les ressources communes (notamment la base) plutôt que vers la gateway.

### Figure 5 - Cache désactivé, 1 instance

![Figure 5 - 1 instance sans cache](images/without-cache-1.png)

La Figure 5 constitue la baseline sans optimisation de lecture. Les latences p95/p99 sont plus sensibles à la charge et aux accès répétés en base, surtout pour les parcours de consultation. Le débit stable est plus limité qu’avec cache.

### Figure 6 - Cache désactivé, 2 instances

![Figure 6 - 2 instances sans cache](images/without-cache-2.png)

La Figure 6 montre qu’ajouter des instances améliore la réponse même sans cache, mais une part de la latence reste due aux accès DB. Le gain de scalabilité est donc réel, mais moins prononcé que dans la série avec cache.

### Figure 7 - Cache désactivé, 3 instances

![Figure 7 - 3 instances sans cache](images/without-cache-3.png)

La Figure 7 met en évidence une amélioration progressive du throughput avec 3 instances, tout en conservant des p95/p99 plus élevés que le scénario équivalent avec cache. Les métriques de saturation montrent une pression plus forte sur les composants de persistance.

### Figure 8 - Cache désactivé, 4 instances

![Figure 8 - 4 instances sans cache](images/without-cache-4.png)

La Figure 8 atteint la meilleure capacité de la série sans cache, mais avec un écart de latence persistant vis-à-vis du mode cache ON. Cela confirme que la montée en charge horizontale et le cache sont complémentaires : l’une répartit la charge, l’autre réduit le coût unitaire de certaines requêtes.

### Synthèse transversale des métriques

- P95/P99 latence : diminuent globalement avec le nombre d’instances ; à charge comparable, elles sont meilleures en cache ON qu’en cache OFF.
- Requests per second : augmentent avec le scaling ; la série cache ON maintient un débit plus stable sous charge.
- Error rate : reste faible sur les campagnes nominales, ce qui indique une bonne robustesse fonctionnelle du chemin `client -> gateway -> services`.
- CPU/Mémoire : la pression se répartit mieux avec 2-4 instances ; sans cache, la pression remonte plus vite sur les composants d’accès données.

## 8. Validation des exigences non fonctionnelles

Les cibles imposées sont :

- Microservices :
  - Latence p95 <= 500 ms
  - Throughput >= 600 ops/s
  - Disponibilité >= 95 %

- Event-driven :
  - Latence p95 <= 250 ms
  - Throughput >= 1000 ops/s
  - Disponibilité >= 99 %

### Évaluation pour CanBankX Phase 1 (architecture microservices)

- Latence p95 :
  d’après les captures et les dashboards instrumentés (p95/p99 par service), la cible microservices est atteinte sur les campagnes de référence, en particulier avec cache actif et à partir de 2 instances.

- Throughput :
  la configuration de test vise un ordre de grandeur compatible avec l’objectif microservices (>= 600 ops/s), et les courbes de débit observables dans les captures confirment l’atteinte de ce niveau dans les configurations scalées.

- Disponibilité :
  les indicateurs `up` et le panneau de disponibilité service montrent un niveau stable au-dessus du seuil attendu durant les exécutions nominales.

Conclusion NFR (Phase 1) :
- objectifs microservices : globalement satisfaits dans les configurations optimisées (cache ON, 2-4 instances) ;
- objectifs event-driven : non applicables à cette phase, car l’architecture implémentée et évaluée est microservices.

## 9. Discussion

Les résultats mettent en évidence plusieurs comportements structurants.

- Goulots d’étranglement :
  en mode sans cache, la base de données devient plus rapidement limitante sur les charges de lecture répétitives.

- Apport du load balancing :
  l’augmentation du nombre d’instances réduit les pics de latence et augmente le débit soutenu, surtout entre 1 et 3 instances.

- Apport du cache :
  le cache diminue la latence des parcours de consultation et absorbe mieux les pointes de trafic read-heavy.

- Limites observées :
  au-delà d’un certain nombre d’instances, le gain marginal baisse, signe d’un transfert du goulot vers des ressources partagées (DB/réseau).

- Pistes d’amélioration :
  optimisation SQL et indexation fine, pool de connexions plus adaptatif, politique de cache plus ciblée (TTL par endpoint), tests de résilience plus agressifs (pannes partielles, chaos), et extension vers une architecture event-driven en Phase 2 pour comparer objectivement les cibles 250 ms / 1000 ops/s / 99 %.

## 10. Conclusion

La Phase 1 de CanBankX valide une architecture microservices complète, sécurisée et observable de bout en bout. La combinaison `NGINX + KrakenD + services Rust + PostgreSQL + Redis + Prometheus/Grafana` fournit une base robuste pour un système bancaire distribué.

Les campagnes de charge montrent que :
- la scalabilité horizontale (1 à 4 instances) améliore la stabilité et la capacité ;
- le cache apporte un gain net sur la latence et la tenue du débit ;
- l’observabilité permet d’expliquer les effets de conception à partir de mesures objectives (p95/p99, req/s, erreurs, CPU, mémoire, disponibilité).

En synthèse, l’architecture de Phase 1 répond aux exigences microservices ciblées et constitue une base solide pour les évolutions futures, notamment la comparaison avec une approche event-driven.
