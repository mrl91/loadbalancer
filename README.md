# 🔄 Rust Load Balancer

Ce projet est une implémentation d'un Load Balancer en Rust, conçu pour répartir les requêtes HTTP entrantes entre plusieurs serveurs en fonction de leur disponibilité et d'une politique de limitation du débit. Le but principal est d'offrir une solution de répartition de charge efficace et sécurisée pour des environnements de test ou de production.

# ⚠️ Disclaimer

**Le Rust Load Balancer est développé avec l'intention d'être utilisé dans des scénarios légitimes et contrôlés, tels que les laboratoires de test, les environnements de développement ou avec le consentement explicite des cibles. Ce projet NE doit PAS être utilisé à des fins malveillantes ou dans tout contexte qui pourrait entraîner un accès non autorisé à des systèmes informatiques. Les auteurs déclinent toute responsabilité pour l'usage impropre de cet outil. En utilisant ce logiciel, vous acceptez de le faire à VOS PROPRES RISQUES.**

## 👥 Auteur

- [@mrl91](https://github.com/mrl91)

## 🛠️ Installation

Prérequis :

- **Rust** *(la dernière version stable recommandée)*
- **Cargo**

Clonez le dépôt sur votre système local :

```sh
git clone https://github.com/mrl91/loadbalancer.git
cd loadbalancer
```

## ⚙️ Configuration

Assurez-vous de configurer correctement les adresses IP et les ports des serveurs HTTP dans votre configuration du Load Balancer. Ceci peut être fait en modifiant directement dans le code source avant la compilation.

**bin/simple_server().rs :**

```rust
let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
```

## 📚 Utilisation

Une fois lancé, le Load Balancer commencera à écouter sur le port spécifié pour les requêtes entrantes et les répartira selon la configuration établie et la disponibilité des serveurs.

   1. **Compilation et lancement des serveurs HTTP:**
      Compilez et lancez chaque serveur en amont dans des terminaux séparés en exécutant les commandes suivantes :
      ```sh
      cargo run --bin simple_server1
      cargo run --bin simple_server2
      ```

   2. **Lancement du Load Balancer:**
      Lors du premier lancement du load balancer, lancer le load balancer en exécutant la commande suivante : 
      ```sh
      RUST_LOG=info cargo run -- --port 8080
      ```

      Sinon, lancer le load balancer en exécutant la commande suivante :
      ```sh
      cargo run -- --port 8080
      ```

   3. **Envoi de requêtes HTTP:**
      Utilisez un outil de requête HTTP tel que curl pour envoyer des requêtes à votre load balancer. Exemple :
      ```sh
      curl http://127.0.0.1:8080/
      ```

   4. **Envoi de 110 requêtes Curl:**
      Pour envoyer 110 requêtes curl, vous pouvez utiliser la boucle suivante :
      ```sh
      for i in {1..110}; do
      curl http://127.0.0.1:8080/
      done
      ```
   Le load balancer bloque à un moment donné les requêtes car il considère ça comme une attaque.

## ✨ Fonctionnalités

- **Répartition de charge** : Distribue les requêtes entrantes de manière équilibrée entre les serveurs en amont.
- **Limitation du débit** : Prévient la surcharge des serveurs avec une politique de limitation de débit configurable.
- **Vérification de l'état de santé** : Vérifie régulièrement la disponibilité des serveurs en amont pour assurer une distribution fiable.
 
