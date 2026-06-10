use anyhow::{anyhow, Context, Result};
use reqwest::ClientBuilder;
use std::net::{IpAddr, SocketAddr};

/// Outbound IP protocol-version restriction for a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpFamily {
    V4,
    V6,
}

impl IpFamily {
    pub fn matches(self, ip: IpAddr) -> bool {
        match self {
            IpFamily::V4 => ip.is_ipv4(),
            IpFamily::V6 => ip.is_ipv6(),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            IpFamily::V4 => "IPv4",
            IpFamily::V6 => "IPv6",
        }
    }

    pub fn flag(self) -> &'static str {
        match self {
            IpFamily::V4 => "--ipv4-only",
            IpFamily::V6 => "--ipv6-only",
        }
    }

    fn of(ip: IpAddr) -> IpFamily {
        if ip.is_ipv4() {
            IpFamily::V4
        } else {
            IpFamily::V6
        }
    }
}

pub fn resolve_ip_family(
    ipv4_only: bool,
    ipv6_only: bool,
    bind_ip: Option<IpAddr>,
) -> Result<Option<IpFamily>> {
    let explicit = match (ipv4_only, ipv6_only) {
        (true, true) => {
            return Err(anyhow!(
                "--ipv4-only and --ipv6-only cannot be used together"
            ))
        }
        (true, false) => Some(IpFamily::V4),
        (false, true) => Some(IpFamily::V6),
        (false, false) => None,
    };

    let implied = bind_ip.map(IpFamily::of);

    match (explicit, implied) {
        (Some(e), Some(i)) if e != i => Err(anyhow!(
            "{} was requested but the bound source address {} is {}",
            e.flag(),
            bind_ip.expect("implied family requires a bind IP"),
            i.label(),
        )),
        (Some(e), _) => Ok(Some(e)),
        (None, i) => Ok(i),
    }
}

pub async fn resolve_addrs_for_family(
    host: &str,
    port: u16,
    family: IpFamily,
) -> Result<Vec<SocketAddr>> {
    let target = format!("{}:{}", host, port);
    let addrs: Vec<SocketAddr> = tokio::net::lookup_host(&target)
        .await
        .with_context(|| format!("DNS lookup failed for {}", host))?
        .filter(|a| family.matches(a.ip()))
        .collect();

    if addrs.is_empty() {
        return Err(anyhow!(
            "no {} address resolved for {} ({} in effect)",
            family.label(),
            host,
            family.flag(),
        ));
    }

    Ok(addrs)
}

/// Get the IP address of a network interface using the `if-addrs` crate
pub fn get_interface_ip(interface: &str) -> Result<IpAddr> {
    use if_addrs::get_if_addrs;

    let addrs = get_if_addrs().context("Failed to enumerate network interfaces")?;

    // Prefer IPv4 addresses
    for addr in &addrs {
        if addr.name == interface {
            if let if_addrs::IfAddr::V4(v4) = &addr.addr {
                return Ok(IpAddr::V4(v4.ip));
            }
        }
    }

    // Fallback to IPv6 if no IPv4 found
    for addr in &addrs {
        if addr.name == interface {
            if let if_addrs::IfAddr::V6(v6) = &addr.addr {
                return Ok(IpAddr::V6(v6.ip));
            }
        }
    }

    Err(anyhow::anyhow!(
        "Interface {} not found or has no IP address assigned",
        interface
    ))
}

/// Resolve binding address from interface name or source IP
pub fn resolve_bind_address(
    interface: Option<&String>,
    source_ip: Option<&String>,
) -> Result<Option<SocketAddr>> {
    if let Some(ip_str) = source_ip {
        let ip: IpAddr = ip_str.parse().context("Invalid source IP address format")?;
        return Ok(Some(SocketAddr::new(ip, 0)));
    }

    if let Some(iface) = interface {
        let ip = get_interface_ip(iface)
            .with_context(|| format!("Failed to get IP for interface {}", iface))?;
        return Ok(Some(SocketAddr::new(ip, 0)));
    }

    Ok(None)
}

/// Apply local address binding to a reqwest client builder.
/// If `bind_ip` is Some, binds the client to that local address.
pub fn apply_local_address(builder: ClientBuilder, bind_ip: Option<IpAddr>) -> ClientBuilder {
    match bind_ip {
        Some(ip) => builder.local_address(ip),
        None => builder,
    }
}

/// Reverse-lookup: find the interface name that owns a given IP address.
pub fn get_interface_for_ip(ip_str: &str) -> Option<String> {
    let target_ip: IpAddr = ip_str.parse().ok()?;
    let addrs = if_addrs::get_if_addrs().ok()?;

    for addr in &addrs {
        let iface_ip = match &addr.addr {
            if_addrs::IfAddr::V4(v4) => IpAddr::V4(v4.ip),
            if_addrs::IfAddr::V6(v6) => IpAddr::V6(v6.ip),
        };
        if iface_ip == target_ip {
            return Some(addr.name.clone());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Name of the loopback interface on the current platform.
    /// Linux/Android call it "lo"; macOS and the BSDs call it "lo0".
    #[cfg(any(target_os = "linux", target_os = "android"))]
    const LOOPBACK_IFACE: &str = "lo";
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    const LOOPBACK_IFACE: &str = "lo0";

    #[test]
    fn test_get_interface_for_ip_loopback() {
        // 127.0.0.1 is bound to the loopback interface ("lo" on Linux, "lo0" on macOS/BSD)
        let iface = get_interface_for_ip("127.0.0.1");
        assert_eq!(iface, Some(LOOPBACK_IFACE.to_string()));
    }

    #[test]
    fn test_get_interface_for_ip_not_found() {
        // No interface should own this arbitrary IP
        let iface = get_interface_for_ip("198.51.100.99");
        assert_eq!(iface, None);
    }

    #[test]
    fn test_get_interface_for_ip_invalid() {
        let iface = get_interface_for_ip("not-an-ip");
        assert_eq!(iface, None);
    }

    #[test]
    fn test_get_interface_ip_loopback() {
        let ip = get_interface_ip(LOOPBACK_IFACE).unwrap();
        assert_eq!(ip, "127.0.0.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_get_interface_ip_nonexistent() {
        let result = get_interface_ip("nonexistent_iface_xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_interface_to_ip_and_back() {
        // Get the IP for loopback, then reverse-lookup should return the loopback name
        let ip = get_interface_ip(LOOPBACK_IFACE).unwrap();
        let iface = get_interface_for_ip(&ip.to_string());
        assert_eq!(iface, Some(LOOPBACK_IFACE.to_string()));
    }

    #[test]
    fn test_resolve_bind_address_none() {
        let result = resolve_bind_address(None, None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_bind_address_source_ip() {
        let source = "127.0.0.1".to_string();
        let result = resolve_bind_address(None, Some(&source)).unwrap();
        let addr = result.unwrap();
        assert_eq!(addr.ip(), "127.0.0.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_resolve_bind_address_invalid_source() {
        let source = "not-an-ip".to_string();
        let result = resolve_bind_address(None, Some(&source));
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_bind_address_interface() {
        let iface = LOOPBACK_IFACE.to_string();
        let result = resolve_bind_address(Some(&iface), None).unwrap();
        let addr = result.unwrap();
        assert_eq!(addr.ip(), "127.0.0.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_resolve_bind_address_source_takes_priority() {
        // When both are provided, source_ip wins
        let iface = LOOPBACK_IFACE.to_string();
        let source = "192.168.1.1".to_string();
        let result = resolve_bind_address(Some(&iface), Some(&source)).unwrap();
        let addr = result.unwrap();
        assert_eq!(addr.ip(), "192.168.1.1".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_resolve_ip_family_no_restriction() {
        assert_eq!(resolve_ip_family(false, false, None).unwrap(), None);
    }

    #[test]
    fn test_resolve_ip_family_explicit_flags() {
        assert_eq!(
            resolve_ip_family(true, false, None).unwrap(),
            Some(IpFamily::V4)
        );
        assert_eq!(
            resolve_ip_family(false, true, None).unwrap(),
            Some(IpFamily::V6)
        );
    }

    #[test]
    fn test_resolve_ip_family_both_flags_conflict() {
        assert!(resolve_ip_family(true, true, None).is_err());
    }

    #[test]
    fn test_resolve_ip_family_implied_by_bind_ip() {
        let v4: IpAddr = "192.168.1.1".parse().unwrap();
        let v6: IpAddr = "::1".parse().unwrap();
        assert_eq!(
            resolve_ip_family(false, false, Some(v4)).unwrap(),
            Some(IpFamily::V4)
        );
        assert_eq!(
            resolve_ip_family(false, false, Some(v6)).unwrap(),
            Some(IpFamily::V6)
        );
    }

    #[test]
    fn test_resolve_ip_family_flag_agrees_with_bind_ip() {
        let v4: IpAddr = "192.168.1.1".parse().unwrap();
        assert_eq!(
            resolve_ip_family(true, false, Some(v4)).unwrap(),
            Some(IpFamily::V4)
        );
    }

    #[test]
    fn test_resolve_ip_family_flag_conflicts_with_bind_ip() {
        // --ipv6-only with a v4 source IP is contradictory.
        let v4: IpAddr = "192.168.1.1".parse().unwrap();
        assert!(resolve_ip_family(false, true, Some(v4)).is_err());
        // --ipv4-only with a v6 source IP is contradictory.
        let v6: IpAddr = "::1".parse().unwrap();
        assert!(resolve_ip_family(true, false, Some(v6)).is_err());
    }

    #[test]
    fn test_ip_family_matches() {
        let v4: IpAddr = "8.8.8.8".parse().unwrap();
        let v6: IpAddr = "2001:4860:4860::8888".parse().unwrap();
        assert!(IpFamily::V4.matches(v4));
        assert!(!IpFamily::V4.matches(v6));
        assert!(IpFamily::V6.matches(v6));
        assert!(!IpFamily::V6.matches(v4));
    }

    #[test]
    fn test_apply_local_address_none() {
        // Should build successfully without binding
        let builder = reqwest::Client::builder();
        let client = apply_local_address(builder, None).build();
        assert!(client.is_ok());
    }

    #[test]
    fn test_apply_local_address_some() {
        // Should build successfully with binding
        let builder = reqwest::Client::builder();
        let ip: IpAddr = "127.0.0.1".parse().unwrap();
        let client = apply_local_address(builder, Some(ip)).build();
        assert!(client.is_ok());
    }
}
