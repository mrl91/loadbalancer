/// Module principal du Load Balancer.
///
/// Ce module implémente le coeur du Load Balancer, gérant la répartition des requêtes HTTP entrantes
/// entre plusieurs serveurs en fonction de leur disponibilité, tout en appliquant une politique
/// de limitation de débit pour prévenir la surcharge.

use hyper::{Client, Server, service::{make_service_fn, service_fn}, Body, Request};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use crate::upstream::UpstreamServer;
use crate::rate_limiter::RateLimiter;
use log::info;
use crate::proxy_handler::proxy_request; // Importe la fonction de gestion des requêtes proxy

/// Structure principale du Load Balancer
/// Gère la répartition des requêtes HTTP entrantes entre plusieurs serveurs
/// en fonction de leur disponibilité et applique une politique de limitation de débit
pub struct LoadBalancer {
    /// Liste des serveurs disponibles pour la répartition des requêtes
    /// Utilise "Arc" et "RwLock" pour un accès concurrent sécurisé en lecture et écriture
    pub servers: Arc<RwLock<Vec<UpstreamServer>>>,
    
    /// Gestionnaire de la limitation du débit pour les requêtes
    /// Encapsulé dans un "Arc" et un "Mutex" pour assurer la synchronisation entre les threads
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl LoadBalancer {
    /// Constructeur pour initialiser un nouveau Load Balancer avec des paramètres spécifiques
    /// de limitation de débit
    ///
    /// # Arguments
    /// * "window_secs" - Durée de la fenêtre de limitation en secondes
    /// * "max_requests" - Nombre maximal de requêtes autorisées par fenêtre de temps
    pub fn new(window_secs: u64, max_requests: u32) -> Self {
        Self {
            servers: Arc::new(RwLock::new(Vec::new())), // Initialise une liste vide pour les serveurs
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(window_secs, max_requests))),
        }
    }

    /// Ajoute un serveur à la liste des serveurs gérés par le load balancer
    /// Cette opération est asynchrone pour éviter de bloquer l'exécution pendant la modification
    ///
    /// # Arguments
    /// * "url" - URL du serveur à ajouter
    pub async fn add_server(&self, url: String) {
        let mut servers = self.servers.write().await; // Obtient un verrou en écriture
        servers.push(UpstreamServer::new(url)); // Ajoute le nouveau serveur
    }

    /// Démarre l'exécution asynchrone du serveur de load balancing
    /// Écoute sur l'adresse spécifiée et traite les requêtes entrantes en les répartissant
    /// entre les serveurs disponibles
    ///
    /// # Arguments
    /// * "addr" - Adresse socket sur laquelle écouter
    pub async fn run(&self, addr: SocketAddr) {
        let rate_limiter = self.rate_limiter.clone();
        let servers = self.servers.clone();
        let client = Client::new(); // Client HTTP pour envoyer des requêtes aux serveurs

        // Préparation de la logique de service pour traiter les requêtes entrantes
        let make_svc = make_service_fn(move |_| {
            let rate_limiter = rate_limiter.clone();
            let servers = servers.clone();
            let client = client.clone();

            // Utilise "proxy_request" pour répondre aux requêtes
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                    proxy_request(req, rate_limiter.clone(), servers.clone(), client.clone())
                }))
            }
        });

        // Configuration et démarrage du serveur HTTP
        let server = Server::bind(&addr).serve(make_svc);
        info!("En écoute sur http://{}", addr);

        // Gestion des erreurs potentielles lors de l'exécution du serveur
        if let Err(e) = server.await {
            eprintln!("Erreur sur le serveur : {}", e);
        }
    }
}
 