<p align="center">
  <h1 align="center">Eidra</h1>
  <p align="center"><strong>Voyez exactement ce que vos outils d'IA sont en train de divulguer. Puis arrêtez-le.</strong></p>
  <p align="center">
    <a href="https://github.com/hanabi-jpn/eidra/actions"><img src="https://github.com/hanabi-jpn/eidra/workflows/CI/badge.svg" alt="CI"></a>
    <a href="https://github.com/hanabi-jpn/eidra/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
    <a href="https://github.com/hanabi-jpn/eidra/stargazers"><img src="https://img.shields.io/github/stars/hanabi-jpn/eidra?style=social" alt="GitHub Stars"></a>
  </p>
  <p align="center">
    <a href="../README.md"><kbd>English</kbd></a>
    <a href="README.ja.md"><kbd>日本語</kbd></a>
    <a href="README.zh.md"><kbd>简体中文</kbd></a>
    <a href="README.ko.md"><kbd>한국어</kbd></a>
    <a href="README.es.md"><kbd>Español</kbd></a>
    <br>
    <a href="README.pt.md"><kbd>Português</kbd></a>
    <a href="README.fr.md"><kbd><strong>Français</strong></kbd></a>
    <a href="README.de.md"><kbd>Deutsch</kbd></a>
    <a href="README.ru.md"><kbd>Русский</kbd></a>
    <a href="README.hi.md"><kbd>हिन्दी</kbd></a>
  </p>
</p>

---

> Cette page est la version d'onboarding multilingue pour GitHub. Les détails les plus récents sur les fonctionnalités et les intégrations sont maintenus dans le [README anglais](../README.md).

Claude Code peut lire votre `.env` sans vous demander. Les dépôts utilisant Copilot exposent des secrets [40 % plus souvent](https://www.knostic.ai/blog/claude-cursor-env-file-secret-leakage). Les outils MCP présentent aussi des [vulnérabilités de contournement CVSS 8.6](https://thehackernews.com/2025/12/researchers-uncover-30-flaws-in-ai.html). En pratique, vos clés API, les données de vos clients et votre code interne peuvent partir vers des serveurs que vous ne contrôlez pas sans que vous le voyiez.

**Eidra est une trust layer locale placée entre vous et vos outils d'IA.** Il analyse chaque requête, masque les secrets avant qu'ils ne quittent la machine, bloque ce qui ne devrait pas sortir et vous montre les flux en temps réel.

Pas de cloud. Pas de compte. Tout reste sur votre appareil.

<p align="center">
  <img src="demo.gif" alt="Eidra TUI Dashboard" width="800">
</p>

```bash
curl -sf eidra.dev/install | sh
eidra init
eidra doctor --json
eidra setup shell --write
eidra dashboard
```

---

## Le problème

Chaque fois que vous utilisez Cursor, Claude Code, Copilot, un SDK ou un outil MCP :

1. Le **contexte complet des fichiers**, y compris `.env`, les clés API et les identifiants, peut partir vers des API cloud
2. Les **outils MCP** peuvent toucher aux fichiers, bases de données et shell
3. Vous avez très peu de **visibilité** sur ce qui sort réellement de votre machine

Vous voulez aller vite avec l'IA, mais la frontière de confiance ne vous appartient pas.

## Ce que fait Eidra

Eidra intercepte le trafic IA au niveau du proxy et décide localement.

| Situation | Sans Eidra | Avec Eidra |
|---|---|---|
| Clé AWS dans le prompt | Envoyée au cloud | `[REDACTED:api_key:a3f2]` |
| Contenu de `.env` | Envoyé en silence | Bloqué ou masqué |
| Clé SSH privée | Envoyée au cloud | **Bloquée** (403) |
| Données personnelles | Envoyées telles quelles | Masquées pour le cloud, autorisées pour un LLM local |
| Appels MCP | Sans limite | Contrôlés par policy |

## Fonctionnalités principales

- **Visibilité des flux** : 47 règles intégrées, TUI temps réel et audit SQLite
- **Protection** : policies YAML, masquage intelligent, routage vers un LLM local et interception HTTPS
- **MCP firewall** : liste blanche de serveurs, ACL par outil, scan des réponses et rate limiting
- **Automatisation** : `doctor --json`, `scan --json`, `config validate --json` et `setup --write`

## Démarrage rapide

```bash
# Installer
curl -sf eidra.dev/install | sh
# Ou compiler depuis la source
git clone https://github.com/hanabi-jpn/eidra.git && cd eidra && cargo install --path crates/eidra-core

# Initialiser
eidra init

# Vérifier l'état local
eidra doctor

# JSON pour scripts ou CI
eidra doctor --json

# Étapes d'intégration pour votre environnement
eidra setup cursor
eidra setup cursor --write

# Démarrer avec le dashboard
eidra dashboard

# Lancer le gateway du firewall MCP
eidra gateway

# Scan ponctuel
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan

# JSON pour CI ou autres outils
echo "my key AKIAIOSFODNN7EXAMPLE" | eidra scan --json
```

### Faire confiance à la CA (pour l'interception HTTPS)

```bash
# macOS
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ~/.eidra/ca.pem

# Linux
sudo cp ~/.eidra/ca.pem /usr/local/share/ca-certificates/eidra.crt && sudo update-ca-certificates

# Puis configurez le proxy
export HTTPS_PROXY=http://127.0.0.1:8080
```

## Commandes principales

```
eidra init                    Générer la CA et la configuration initiale
eidra doctor                  Vérifier la readiness et la configuration effective
eidra doctor --json           Émettre l'état en JSON
eidra setup [target]          Afficher les étapes d'intégration pour les environnements courants
eidra setup --write           Générer des artefacts réutilisables dans ~/.eidra/generated/<target>/
eidra start                   Démarrer le proxy d'interception
eidra start -d                Démarrer proxy + TUI
eidra dashboard               Démarrer proxy + TUI
eidra gateway                 Lancer le gateway du firewall MCP
eidra stop                    Arrêter le proxy
eidra scan [file]             Scanner un fichier ou stdin
eidra scan --json             Émettre les findings en JSON
eidra escape                  Créer une salle chiffrée sans traces
eidra join <id> <port>        Rejoindre une salle chiffrée
eidra config                  Voir ou éditer la configuration
eidra config --json           Émettre les données de configuration en JSON
eidra config validate         Valider config et policy
eidra config validate --json  Émettre la validation en JSON
```

## Cibles de setup

`eidra setup <target>` imprime des étapes prêtes à copier pour les environnements les plus courants.

```bash
eidra setup shell
eidra setup cursor
eidra setup claude-code
eidra setup openai-sdk
eidra setup anthropic-sdk
eidra setup github-actions
eidra setup mcp
```

Avec `eidra setup <target> --write`, Eidra génère des fichiers réutilisables dans `~/.eidra/generated/<target>/` sans modifier directement votre shell ou votre IDE.

## CI et automatisation

`eidra scan --json`, `eidra doctor --json` et `eidra config validate --json` sont pensés pour le CI, les scripts et les autres outils d'IA qui doivent consommer la sortie sans parser du texte destiné aux humains.

## Exemple de policy

```yaml
# ~/.eidra/policy.yaml
version: "1"
default_action: allow
rules:
  - name: block_private_keys
    match:
      category: "private_key"
    action: block

  - name: mask_api_keys
    match:
      category: "api_key"
    action: mask

  - name: mask_pii_for_cloud
    match:
      category: "pii"
      destination: "cloud"
    action: mask

  - name: allow_pii_for_local
    match:
      category: "pii"
      destination: "local"
    action: allow
```

## Plus de détails

Pour les informations les plus récentes sur MCP Semantic RBAC, les règles personnalisées, les canaux sécurisés, l'architecture et la roadmap, consultez le [README anglais](../README.md). Les pages traduites privilégient un onboarding clair et les commandes essentielles.

## Contribuer

Projet sous licence MIT. Les PR sont les bienvenues. Plus d'informations dans [CONTRIBUTING.md](contributing.md).
