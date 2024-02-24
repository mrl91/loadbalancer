/// Gère les requêtes HTTP entrantes du load balancer
///
/// Redirige les requêtes vers les serveurs en fonction de leur disponibilité et de la politique de limitation de débit,
/// assurant ainsi une répartition équilibrée du trafic et prévenant la surcharge des serveurs

use hyper::{Body, Client, Request, Response, StatusCode};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use crate::upstream::UpstreamServer; // Représente un serveur
use crate::rate_limiter::RateLimiter; // Gère la limitation du débit des requêtes
use log::{info, warn};
use once_cell::sync::Lazy; // Pour l'initialisation de l'index global

/// Mutex protégeant un index global pour le round-robin
static NEXT_SERVER_INDEX: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

/// Traite une requête entrante et la redirige vers un serveur
///
/// # Arguments
/// * "req" - La requête HTTP entrante
/// * "rate_limiter" - Le gestionnaire de limitation de débit partagé
/// * "servers" - La liste partagée des serveurs 
/// * "client" - Client HTTP pour effectuer les requêtes vers les serveurs
///
/// # Retour
/// Renvoie une réponse HTTP résultant de la redirection vers un serveur ou un message d'erreur
/// si aucune redirection n'est possible
pub async fn proxy_request(
    req: Request<Body>, 
    rate_limiter: Arc<Mutex<RateLimiter>>,
    servers: Arc<RwLock<Vec<UpstreamServer>>>, 
    client: Client<hyper::client::HttpConnector>,
) -> Result<Response<Body>, hyper::Error> {
    // Extraction de l'adresse IP du client à partir de l'en-tête de la requête
    let ip = req.headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_else(|| "unknown");
    
    // Vérifie si la requête dépasse la limite de débit autorisée
    let is_allowed = {
        let rate_limiter = rate_limiter.lock().await;
        rate_limiter.check(ip) 
    };

    if !is_allowed {
        info!("Limitation du débit pour l'IP: {}", ip);
        return Ok(Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(Body::from("Too Many Requests\n"))
            .unwrap());
    }

    // Sélectionne le serveur suivant en mode round-robin parmi les serveurs sains
    let selected_server = select_next_server(servers).await;

    // Envoie la requête au serveur sélectionné et renvoie la réponse obtenue
    if let Some(server) = selected_server {
        let uri_string = format!("{}{}", server.url, req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("/"));
        let new_req = Request::builder()
            .method(req.method())
            .uri(uri_string)
            .body(req.into_body())
            .expect("Failed to create the request");

        info!("Transfert de la requête vers le serveur : {}", server.url);
        let response = client.request(new_req).await?;
        info!("Réponse reçue du serveur : {}", server.url);
        info!("Statut de la réponse: {}", response.status());
        Ok(response)
    } else {
        warn!("Aucun serveur sain disponible.");
        Ok(Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from("Service Unavailable"))
            .unwrap())
    }
}

/// Sélectionne le prochain serveur sain à utiliser pour la requête entrante en mode round-robin
///
/// # Arguments
/// * "servers" - La liste partagée des serveurs
///
/// # Retour
/// Renvoie une option contenant un serveur sain ou "None" si aucun serveur sain n'est disponible
async fn select_next_server(servers: Arc<RwLock<Vec<UpstreamServer>>>) -> Option<UpstreamServer> {
    let servers_read = servers.read().await; // Accès sécurisé et asynchrone à la liste des serveurs
    let mut healthy_servers = Vec::new();

    // Collecte les serveurs sains
    for server in servers_read.iter() {
        if *server.is_healthy.read().await {
            healthy_servers.push(server);
        }
    }

    if healthy_servers.is_empty() {
        None
    } else {
        let mut next_index = NEXT_SERVER_INDEX.lock().await;
        *next_index = *next_index % healthy_servers.len();
        let server = healthy_servers[*next_index].clone();
        *next_index = (*next_index + 1) % healthy_servers.len();
        Some(server)
    }
}
