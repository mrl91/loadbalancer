/// Module principal pour le Load Balancer
/// 
/// Ce module configure et exécute un load balancer basé sur une interface CLI
/// Il répartit les requêtes HTTP entrantes entre plusieurs serveurs selon leur état de santé
/// et applique une politique de limitation de débit. Des vérifications périodiques de l'état de santé
/// des serveurs sont également effectuées

use env_logger::Env;
use std::net::SocketAddr;
use clap::{App, Arg};

mod load_balancer;
mod upstream;
mod rate_limiter;
mod proxy_handler;

use crate::load_balancer::LoadBalancer;

/// Point d'entrée du programme
///
/// Initialise le système de logging, configure l'interface CLI pour le port d'écoute,
/// crée une instance du LoadBalancer, ajoute des serveurs, et lance le processus de vérification
/// périodique de l'état de santé des serveurs. Ensuite, démarre le load balancer pour écouter
/// et traiter les requêtes entrantes
#[tokio::main]
async fn main() {
    // Initialise le système de logging basé sur la variable d'environnement "RUST_LOG"
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Configure et analyse les arguments CLI pour le port d'écoute du load balancer
    let matches = App::new("Load Balancer")
        .version("1.0")
        .author("Loic LEAL")
        .about("Load Balancer en Rust.")
        .arg(Arg::new("port")
             .short('p')
             .long("port")
             .takes_value(true)
             .help("Définit le port d'écoute pour le load balancer."))
        .get_matches();

    // Détermine le port d'écoute à partir des arguments CLI, avec une valeur par défaut si non spécifié
    let port = matches.value_of("port").unwrap_or("8080");
    let addr = format!("127.0.0.1:{}", port).parse::<SocketAddr>().unwrap();

    // Crée une instance du LoadBalancer avec des paramètres de limitation de débit spécifiques
    let load_balancer = LoadBalancer::new(60, 100); // Utilise une fenêtre de 60 secondes et un max de 100 requêtes

    // Ajoute des serveurs au load balancer de manière asynchrone.
    load_balancer.add_server("http://127.0.0.1:3000".to_string()).await;
    load_balancer.add_server("http://127.0.0.1:3001".to_string()).await;

    // Lance une routine asynchrone pour effectuer des vérifications périodiques de l'état de santé des serveurs
    let servers = load_balancer.servers.clone();
    tokio::spawn(async move {
        upstream::health_check_loop(servers).await;
    });

    // Démarre le load balancer pour écouter sur l'adresse spécifiée et traiter les requêtes entrantes
    load_balancer.run(addr).await;
}
