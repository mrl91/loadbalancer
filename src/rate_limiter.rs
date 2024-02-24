/// Gestionnaire de limitation de débit basé sur une fenêtre de temps
///
/// Ce module implémente un mécanisme de limitation de débit pour contrôler le nombre de requêtes
/// qu'une adresse IP peut effectuer dans un intervalle de temps donné, afin de prévenir
/// la surcharge du serveur

use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Structure représentant le gestionnaire de limitation de débit
pub struct RateLimiter {
    /// Compteur de requêtes par IP, protégé par un Mutex pour une synchronisation entre threads
    requests: Mutex<HashMap<String, (u32, Instant)>>,
    
    /// Durée de la fenêtre de limitation de débit pendant laquelle le comptage des requêtes est effectué
    window: Duration,
    
    /// Nombre maximal de requêtes autorisées par adresse IP pendant la fenêtre de temps
    max_requests: u32,
}

impl RateLimiter {
    /// Crée une nouvelle instance de "RateLimiter"
    ///
    /// # Arguments
    ///
    /// * "window_secs" - La durée de la fenêtre de limitation en secondes
    /// * "max_requests" - Le nombre maximal de requêtes autorisées par fenêtre de temps par adresse IP
    pub fn new(window_secs: u64, max_requests: u32) -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            window: Duration::from_secs(window_secs),
            max_requests,
        }
    }

    /// Vérifie si une requête provenant d'une adresse IP spécifique est autorisée
    /// en fonction de la politique de limitation de débit définie
    ///
    /// # Arguments
    ///
    /// * "ip" - L'adresse IP pour laquelle vérifier la limitation de débit
    ///
    /// # Retour
    ///
    /// Renvoie "true" si la requête est autorisée, "false" sinon
    pub fn check(&self, ip: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let current_time = Instant::now();

        let entry = requests.entry(ip.to_string()).or_insert((0, current_time));

        if current_time.duration_since(entry.1) > self.window {
            *entry = (1, current_time);
            true
        } else if entry.0 < self.max_requests {
                entry.0 += 1;
                true
            } else {
                false
            }
    }
} 
