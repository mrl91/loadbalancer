// Importations nécessaires depuis la crate `hyper` pour la création du serveur HTTP, la gestion des requêtes et des réponses.
use hyper::{Body, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::Request; // Importation spécifique pour le type de requête HTTP.
use std::net::SocketAddr; // Utilisé pour spécifier l'adresse et le port d'écoute du serveur.

// Fonction asynchrone pour gérer les requêtes HTTP reçues.
// Elle renvoie une réponse HTTP avec un corps de type `Body`.
async fn handle_request(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    // Création et renvoi d'une réponse HTTP simple avec le corps "Hello from upstream server!"
    // Note : `_req` est préfixé par `_` car il n'est pas utilisé dans cette fonction simple.
    Ok(Response::new(Body::from("Hello from upstream server!\n")))
}

// Point d'entrée principal du serveur.
#[tokio::main] 
async fn main() {
    // Définition de l'adresse et du port d'écoute du serveur.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    
    // Création d'un `service` utilisant `make_service_fn` et `service_fn` pour lier `handle_request`
    // comme la fonction de gestion des requêtes HTTP.
    let service = make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(handle_request))
    });

    // Initialisation et démarrage du serveur avec l'adresse et le `service` configurés.
    let server = Server::bind(&addr).serve(service);
    println!("Serveur lancé : {:?}", addr); // Affiche l'adresse d'écoute dans la console.

    // Gestion des erreurs potentielles lors de l'exécution du serveur.
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
