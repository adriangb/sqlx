use std::time::Duration;

/// TCP keepalive parameters for a connection's socket.
///
/// Keepalive is what lets a client notice that its server is gone when the
/// server disappeared without closing the socket: a failover, a killed
/// container, a NAT table that dropped the mapping. Without it, a connection
/// blocked reading a response it will never receive waits forever, because
/// there is nothing left to retransmit and so nothing to time out.
///
/// The parameters mirror libpq's `keepalives_idle`, `keepalives_interval` and
/// `keepalives_count`, and are applied with `setsockopt` after connecting.
///
/// Support varies by platform: `interval` is ignored on OpenBSD and Solaris,
/// and `retries` is ignored on OpenBSD, Solaris, watchOS and tvOS. `idle` is
/// supported everywhere keepalive itself is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TcpKeepalive {
    /// Idle time after which the first keepalive probe is sent.
    pub idle: Option<Duration>,
    /// Time between probes once the first one has been sent.
    pub interval: Option<Duration>,
    /// Number of unacknowledged probes before the connection is dropped.
    pub retries: Option<u32>,
}

impl TcpKeepalive {
    /// Keepalive with no parameters overridden, leaving the system defaults in
    /// place. On Linux those are 7200s idle, 75s interval, 9 retries.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the idle time after which the first keepalive probe is sent.
    pub fn with_idle(mut self, idle: Duration) -> Self {
        self.idle = Some(idle);
        self
    }

    /// Sets the time between keepalive probes.
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }

    /// Sets the number of unacknowledged probes before the connection is dropped.
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = Some(retries);
        self
    }

    #[cfg(any(feature = "_rt-tokio", feature = "_rt-async-io"))]
    pub(crate) fn to_socket2(self) -> socket2::TcpKeepalive {
        let mut keepalive = socket2::TcpKeepalive::new();

        if let Some(idle) = self.idle {
            keepalive = keepalive.with_time(idle);
        }

        #[cfg(not(any(target_os = "openbsd", target_os = "solaris")))]
        if let Some(interval) = self.interval {
            keepalive = keepalive.with_interval(interval);
        }

        #[cfg(not(any(
            target_os = "openbsd",
            target_os = "solaris",
            target_os = "watchos",
            target_os = "tvos",
        )))]
        if let Some(retries) = self.retries {
            keepalive = keepalive.with_retries(retries);
        }

        keepalive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keepalive_is_disabled_by_default() {
        assert_eq!(TcpKeepalive::new(), TcpKeepalive::default());
        assert!(TcpKeepalive::new().idle.is_none());
    }

    #[test]
    fn builders_set_each_parameter() {
        let keepalive = TcpKeepalive::new()
            .with_idle(Duration::from_secs(30))
            .with_interval(Duration::from_secs(10))
            .with_retries(3);

        assert_eq!(keepalive.idle, Some(Duration::from_secs(30)));
        assert_eq!(keepalive.interval, Some(Duration::from_secs(10)));
        assert_eq!(keepalive.retries, Some(3));
    }
}
