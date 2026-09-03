//! MaxMind GeoLite2 datacenter detection for the rate limiter.
//!
//! Replaces the hand-rolled `IP_TAG_SOURCES` flat-file HashSet (which was
//! loaded but never consulted — `is_datacenter_ip` hardcoded `false`) with a
//! proper GeoLite2 ASN lookup: an IP whose autonomous-system owner matches a
//! known hosting/datacenter keyword is treated as datacenter traffic and
//! blocked by the tiered limiter.
//!
//! Fail-open: when the `.mmdb` file is missing/unreadable, `GeoIp` is `None`
//! and the limiter simply skips the datacenter check (no request is ever
//! blocked because geo lookup failed).

use std::net::IpAddr;
use std::path::Path;

/// ASN organizations treated as datacenter/hosting egress.
///
/// Matched case-insensitively against the GeoLite2-ASN `autonomous_system_organization`
/// field. This is intentionally a broad set — the goal is to catch the common
/// cloud providers and hosting ranges that scrapers/bots originate from.
const DATACENTER_ORGS: &[&str] = &[
    "amazon",
    "amazon.com",
    "aws",
    "google",
    "microsoft",
    "azure",
    "ovh",
    "digitalocean",
    "hetzner",
    "linode",
    "akamai",
    "cloudflare",
    "leaseweb",
    "contabo",
    "vultr",
    "choopa",
    "voxel",
    "psychz",
    "qualtrics", // heavy hosting/abuse carrier
    "hostinger",
    "namecheap",
    "godaddy",
    "bluehost",
    "dreamhost",
    "ionos",
    "1&1",
    "scaleway",
    "upcloud",
    "vps",
    "dedicated",
    "colo",
    "datacenter",
    "data center",
    "hosting",
    "server",
    "cloud",
];

/// MaxMind GeoLite2 reader wrapper. Cheap to construct once at startup.
pub struct GeoIp {
    reader: maxminddb::Reader<Vec<u8>>,
}

/// True when an ASN organization string looks like a datacenter/hosting
/// provider. Pure function so it is trivially unit-testable without an mmdb.
fn org_matches_datacenter(org: &str) -> bool {
    let lower = org.to_ascii_lowercase();
    DATACENTER_ORGS.iter().any(|kw| lower.contains(kw))
}

impl GeoIp {
    /// Open a GeoLite2-ASN `.mmdb` file. Returns `None` (fail-open) when the
    /// file cannot be read — callers treat that as "no datacenter blocking".
    pub fn open(path: &Path) -> Option<Self> {
        let bytes = std::fs::read(path).ok()?;
        let reader = maxminddb::Reader::from_source(bytes).ok()?;
        Some(GeoIp { reader })
    }

    /// True when the IP's ASN owner looks like a datacenter/hosting provider.
    /// Unknown IPs / lookup errors return `false` (fail-open).
    pub fn asn_org_is_datacenter(&self, ip: IpAddr) -> bool {
        let org: Option<String> = self
            .reader
            .lookup(ip)
            .ok()
            .and_then(|res| res.decode::<maxminddb::geoip2::Asn>().ok())
            .flatten()
            .and_then(|asn| asn.autonomous_system_organization.map(str::to_owned));
        match org {
            Some(org) => org_matches_datacenter(&org),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datacenter_org_keywords_match() {
        // Hosting / cloud providers → datacenter.
        for org in [
            "Google LLC",
            "Amazon.com, Inc.",
            "Microsoft Corporation",
            "OVH SAS",
            "DigitalOcean, LLC",
            "Hetzner Online GmbH",
            "Akamai Technologies, Inc.",
            "Cloudflare, Inc.",
            "Contabo GmbH",
            "Vultr Holdings, LLC",
            "Linode, LLC",
        ] {
            assert!(
                org_matches_datacenter(org),
                "{org} should be flagged as datacenter"
            );
        }
    }

    #[test]
    fn residential_org_keywords_do_not_match() {
        // Real residential ISPs must NOT be flagged.
        for org in [
            "Comcast Cable Communications, LLC",
            "Charter Communications",
            "Verizon Fios",
            "AT&T Services, Inc.",
            "Deutsche Telekom AG",
            "Sky UK Limited",
            "Vodafone GmbH",
            "Rogers Communications Canada Inc.",
            "NTT Communications Corporation",
            "Telefonica de Espana",
        ] {
            assert!(
                !org_matches_datacenter(org),
                "{org} should NOT be flagged as datacenter"
            );
        }
    }

    #[test]
    fn org_matching_is_case_insensitive() {
        assert!(org_matches_datacenter("AMAZON.COM, INC."));
        assert!(org_matches_datacenter("amazon.com"));
        assert!(org_matches_datacenter("DIGITALOCEAN, LLC"));
    }

    #[test]
    fn open_missing_file_fails_open() {
        let geo = GeoIp::open(Path::new("/nonexistent/GeoLite2-ASN.mmdb"));
        assert!(geo.is_none(), "missing file → None (fail-open)");
    }
}

