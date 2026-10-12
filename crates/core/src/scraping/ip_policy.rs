//! Politique d'adresses IP : seules les adresses publiques peuvent être
//! contactées lors de la récupération d'une page d'offre (protection contre
//! les requêtes vers la machine locale ou le réseau privé).

// Importation des modules standard
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Indique si une adresse IP est publique, donc autorisée comme cible.
///
/// Refuse loopback, adresses privées, link-local, non spécifiées, broadcast,
/// multicast, documentation, plage partagée CGNAT (100.64.0.0/10), plages
/// réservées, ULA IPv6 (fc00::/7) et adresses IPv4 encapsulées dans IPv6.
///
/// # Arguments
///
/// * `ip` - Adresse à vérifier.
///
/// # Returns
///
/// `true` si l'adresse est routable publiquement.
pub fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_public_ipv4(v4),
        IpAddr::V6(v6) => is_public_ipv6(v6),
    }
}

fn is_public_ipv4(ip: Ipv4Addr) -> bool {
    let [a, b, ..] = ip.octets();
    let shared_cgnat = a == 100 && (64..=127).contains(&b);
    let this_network = a == 0;
    let ietf_protocol = a == 192 && b == 0 && ip.octets()[2] == 0;
    let benchmarking = a == 198 && (18..=19).contains(&b);
    let reserved = a >= 240;
    !(ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.is_multicast()
        || ip.is_documentation()
        || shared_cgnat
        || this_network
        || ietf_protocol
        || benchmarking
        || reserved)
}

fn is_public_ipv6(ip: Ipv6Addr) -> bool {
    // Une IPv4 encapsulée (::ffff:a.b.c.d) ou compatible suit la règle IPv4 :
    // sinon ::ffff:127.0.0.1 contournerait le refus du loopback.
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_public_ipv4(v4);
    }
    let first = ip.segments()[0];
    let unique_local = (first & 0xfe00) == 0xfc00;
    let link_local = (first & 0xffc0) == 0xfe80;
    let documentation = first == 0x2001 && ip.segments()[1] == 0x0db8;
    let ipv4_compatible = ip.segments()[..6].iter().all(|s| *s == 0);
    !(ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || unique_local
        || link_local
        || documentation
        || ipv4_compatible)
}
