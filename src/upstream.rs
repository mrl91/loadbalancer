/// Gère la vérification de l'état de santé des serveurs
/// 
/// Implémente une routine de vérification de santé utilisant des requêtes HTTP GET
/// pour déterminer la disponibilité des serveurs. Une boucle asynchrone répète ces vérifications
/// à intervalles réguliers

use hyper::{Client, Uri, StatusCode};
use log::{info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{self, Duration};

/// Représente un serveur avec son URL et son état de santé
#[derive(Clone)]
pub struct UpstreamServer {
    /// URL du serveur
    pub url: String,
    /// Indicateur de l'état de santé du serveur, partagé et modifiable de manière asynchrone
    pub is_healthy: Arc<RwLock<bool>>,
}

impl UpstreamServer {
    /// Crée une nouvelle instance d' "UpstreamServer"
    ///
    /// # Arguments
    ///
    /// * "url" - URL du serveur
    pub fn new(url: String) -> Self {
        Self {
            url,
            is_healthy: Arc::new(RwLock::new(true)), // Initialise comme sain par défaut
        }
    }

    /// Vérifie l'état de santé du serveur en envoyant une requête HTTP GET
    ///
    /// Met à jour l'état de santé du serveur en fonction de la réponse à cette requête
    pub async fn check_health(&self) {
        let client = Client::new();
        let uri = Uri::try_from(&*self.url).expect("Failed to parse URI");

        match client.get(uri).await {
            Ok(response) => {
                let mut is_healthy = self.is_healthy.write().await;
                *is_healthy = response.status() == StatusCode::OK;
                if *is_healthy {
                    info!("{} est UP.", &self.url);
                } else {
                    warn!("{} a répondu avec le statut: {}", &self.url, response.status());
                }
            },
            Err(e) => {
                let mut is_healthy = self.is_healthy.write().await;
                *is_healthy = false;
                warn!("Échec de la vérification de santé pour {}: {}", &self.url, e);
            },
        }
    }
}

/// Exécute une boucle de vérification de santé pour tous les serveurs enregistrés,
/// vérifiant leur état de santé à intervalles réguliers
///
/// # Arguments
///
/// * "servers" - Liste partagée des serveurs à vérifier
///
/// Cette routine lance une vérification immédiate au démarrage, puis continue à vérifier
/// l'état de santé des serveurs toutes les 20 secondes
pub async fn health_check_loop(servers: Arc<RwLock<Vec<UpstreamServer>>>) {
    let mut interval = time::interval(Duration::from_secs(20));
    loop {
        interval.tick().await;
        let servers_read = servers.read().await;
        for server in servers_read.iter() {
            server.check_health().await;
        }
    }
}
