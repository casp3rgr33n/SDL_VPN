use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum SubscriptionTier {
    Free,
    Pro,
    Elite,
}

impl Default for SubscriptionTier {
    fn default() -> Self {
        Self::Free
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSubscription {
    pub user_id: Uuid,
    pub tier: SubscriptionTier,
    pub active_until: Option<DateTime<Utc>>,
    pub bandwidth_used_bytes: u64,
    pub bandwidth_limit_bytes: Option<u64>,
}

impl UserSubscription {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            tier: SubscriptionTier::Free,
            active_until: None,
            bandwidth_used_bytes: 0,
            bandwidth_limit_bytes: Some(5_000_000_000), // 5GB default for free
        }
    }

    pub fn is_active(&self) -> bool {
        match self.active_until {
            Some(expiration) => Utc::now() < expiration,
            None => self.tier == SubscriptionTier::Free, // Free tier doesn't expire, it's just bandwidth limited
        }
    }

    pub fn has_bandwidth_remaining(&self) -> bool {
        match self.bandwidth_limit_bytes {
            Some(limit) => self.bandwidth_used_bytes < limit,
            None => true, // Unlimited bandwidth
        }
    }

    pub fn apply_fallback(&mut self) {
        if !self.is_active() && self.tier != SubscriptionTier::Free {
            self.tier = SubscriptionTier::Free;
            self.active_until = None;
            self.bandwidth_limit_bytes = Some(5_000_000_000);
        }
    }
}
