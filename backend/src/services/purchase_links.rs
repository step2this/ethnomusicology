use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PurchaseLink {
    pub store: String,
    pub name: String,
    pub url: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PurchaseLinkResponse {
    pub links: Vec<PurchaseLink>,
}

#[derive(Debug, Clone)]
pub struct AffiliateConfig {
    pub beatport_affiliate_id: Option<String>,
    pub juno_affiliate_id: Option<String>,
}

impl AffiliateConfig {
    pub fn from_env() -> Self {
        Self {
            beatport_affiliate_id: std::env::var("BEATPORT_AFFILIATE_ID").ok(),
            juno_affiliate_id: std::env::var("JUNO_AFFILIATE_ID").ok(),
        }
    }
}

pub fn build_purchase_links(
    title: &str,
    artist: &str,
    affiliate_config: &AffiliateConfig,
) -> PurchaseLinkResponse {
    let title = title.trim();
    let artist = artist.trim();

    if title.is_empty() && artist.is_empty() {
        return PurchaseLinkResponse { links: vec![] };
    }

    let query = match (title.is_empty(), artist.is_empty()) {
        (false, false) => format!("{artist} {title}"),
        (true, false) => artist.to_string(),
        (false, true) => title.to_string(),
        (true, true) => unreachable!(),
    };

    let encoded = urlencoding::encode(&query);

    let mut links = Vec::with_capacity(4);

    // Beatport
    let mut beatport_url = format!("https://www.beatport.com/search?q={encoded}");
    if let Some(ref id) = affiliate_config.beatport_affiliate_id {
        beatport_url.push_str(&format!("&a_aid={}", urlencoding::encode(id)));
    }
    links.push(PurchaseLink {
        store: "beatport".to_string(),
        name: "Beatport".to_string(),
        url: beatport_url,
        icon: "beatport".to_string(),
    });

    // Bandcamp
    links.push(PurchaseLink {
        store: "bandcamp".to_string(),
        name: "Bandcamp".to_string(),
        url: format!("https://bandcamp.com/search?q={encoded}"),
        icon: "bandcamp".to_string(),
    });

    // Juno Download
    let mut juno_url = format!("https://www.junodownload.com/search/?q%5Ball%5D%5B0%5D={encoded}");
    if let Some(ref id) = affiliate_config.juno_affiliate_id {
        juno_url.push_str(&format!("&aff={}", urlencoding::encode(id)));
    }
    links.push(PurchaseLink {
        store: "juno".to_string(),
        name: "Juno Download".to_string(),
        url: juno_url,
        icon: "juno".to_string(),
    });

    // Traxsource
    links.push(PurchaseLink {
        store: "traxsource".to_string(),
        name: "Traxsource".to_string(),
        url: format!("https://www.traxsource.com/search?term={encoded}"),
        icon: "traxsource".to_string(),
    });

    PurchaseLinkResponse { links }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_affiliate() -> AffiliateConfig {
        AffiliateConfig {
            beatport_affiliate_id: None,
            juno_affiliate_id: None,
        }
    }

    fn with_affiliates() -> AffiliateConfig {
        AffiliateConfig {
            beatport_affiliate_id: Some("bp123".to_string()),
            juno_affiliate_id: Some("jn456".to_string()),
        }
    }

    #[test]
    fn full_query_generates_all_four_stores() {
        let resp = build_purchase_links("Strobe", "Deadmau5", &no_affiliate());
        assert_eq!(resp.links.len(), 4);
        assert!(resp.links[0].url.contains("Deadmau5%20Strobe"));
        assert!(resp.links[1].url.contains("Deadmau5%20Strobe"));
        assert!(resp.links[2].url.contains("Deadmau5%20Strobe"));
        assert!(resp.links[3].url.contains("Deadmau5%20Strobe"));
    }

    #[test]
    fn store_order_is_beatport_bandcamp_juno_traxsource() {
        let resp = build_purchase_links("Test", "Artist", &no_affiliate());
        assert_eq!(resp.links[0].store, "beatport");
        assert_eq!(resp.links[1].store, "bandcamp");
        assert_eq!(resp.links[2].store, "juno");
        assert_eq!(resp.links[3].store, "traxsource");
    }

    #[test]
    fn title_only() {
        let resp = build_purchase_links("Strobe", "", &no_affiliate());
        assert_eq!(resp.links.len(), 4);
        assert!(resp.links[0].url.contains("q=Strobe"));
    }

    #[test]
    fn artist_only() {
        let resp = build_purchase_links("", "Deadmau5", &no_affiliate());
        assert_eq!(resp.links.len(), 4);
        assert!(resp.links[0].url.contains("q=Deadmau5"));
    }

    #[test]
    fn both_empty_returns_no_links() {
        let resp = build_purchase_links("", "", &no_affiliate());
        assert!(resp.links.is_empty());
    }

    #[test]
    fn whitespace_only_returns_no_links() {
        let resp = build_purchase_links("  ", "  ", &no_affiliate());
        assert!(resp.links.is_empty());
    }

    #[test]
    fn special_chars_are_encoded() {
        let resp =
            build_purchase_links("Track (Remix)", "Artist & Friends + More", &no_affiliate());
        // Parentheses, ampersand, plus should be encoded
        let url = &resp.links[0].url;
        assert!(url.contains("%28Remix%29")); // parentheses
        assert!(url.contains("%26")); // &
        assert!(url.contains("%2B")); // +
    }

    #[test]
    fn affiliate_tags_appended_when_present() {
        let resp = build_purchase_links("Track", "Artist", &with_affiliates());
        assert!(resp.links[0].url.contains("a_aid=bp123")); // beatport
        assert!(resp.links[2].url.contains("aff=jn456")); // juno
                                                          // bandcamp and traxsource have no affiliate params
        assert!(!resp.links[1].url.contains("aff"));
        assert!(!resp.links[3].url.contains("aff"));
    }

    #[test]
    fn no_affiliate_tags_when_absent() {
        let resp = build_purchase_links("Track", "Artist", &no_affiliate());
        assert!(!resp.links[0].url.contains("a_aid"));
        assert!(!resp.links[2].url.contains("aff"));
    }

    #[test]
    fn beatport_url_format() {
        let resp = build_purchase_links("Track", "Artist", &no_affiliate());
        assert!(resp.links[0]
            .url
            .starts_with("https://www.beatport.com/search?q="));
    }

    #[test]
    fn bandcamp_url_format() {
        let resp = build_purchase_links("Track", "Artist", &no_affiliate());
        assert!(resp.links[1]
            .url
            .starts_with("https://bandcamp.com/search?q="));
    }

    #[test]
    fn juno_url_format() {
        let resp = build_purchase_links("Track", "Artist", &no_affiliate());
        assert!(resp.links[2]
            .url
            .starts_with("https://www.junodownload.com/search/?q%5Ball%5D%5B0%5D="));
    }

    #[test]
    fn traxsource_url_format() {
        let resp = build_purchase_links("Track", "Artist", &no_affiliate());
        assert!(resp.links[3]
            .url
            .starts_with("https://www.traxsource.com/search?term="));
    }
}
