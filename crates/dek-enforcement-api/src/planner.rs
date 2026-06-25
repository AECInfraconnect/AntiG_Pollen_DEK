use std::cmp::Ord;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ControlLevel {
    Observe,
    Warn,
    Ask,
    Enforce,
}

impl ControlLevel {
    pub fn min_observe(&self) -> ControlLevel {
        ControlLevel::Observe
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlDomain {
    Network,
    FileSystem,
    McpTool,
    Process,
    Dns,
}

impl std::fmt::Display for ControlDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlDomain::Network => write!(f, "network"),
            ControlDomain::FileSystem => write!(f, "file_system"),
            ControlDomain::McpTool => write!(f, "mcp_tool"),
            ControlDomain::Process => write!(f, "process"),
            ControlDomain::Dns => write!(f, "dns"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MethodStatus {
    Available,
    NeedsInstall,
    NeedsPermission,
    Unsupported,
}

#[derive(Clone, Debug)]
pub struct ControlMethodCap {
    pub id: String,
    pub domains: Vec<ControlDomain>,
    pub max_level: ControlLevel,
    pub status: MethodStatus,
}

#[derive(Clone, Debug)]
pub struct CapabilityUpgrade {
    pub unlocks: String,
}

#[derive(Clone, Debug)]
pub struct LocalCapabilitySnapshot {
    pub control_methods: Vec<ControlMethodCap>,
}

impl LocalCapabilitySnapshot {
    pub fn observe_capable(&self, _domain: ControlDomain) -> Option<&ControlMethodCap> {
        None
    }
    pub fn upgrade_for(&self, _domain: ControlDomain) -> Option<&CapabilityUpgrade> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct Policy {
    pub id: String,
    pub requested_level: ControlLevel,
}

impl Policy {
    pub fn required_domains(&self) -> Vec<ControlDomain> {
        vec![]
    }
}

#[derive(Clone, Debug)]
pub struct DomainFeasibility {
    pub domain: ControlDomain,
    pub chosen_method: Option<String>,
    pub level: ControlLevel,
}

impl DomainFeasibility {
    pub fn ok(domain: ControlDomain, method: &ControlMethodCap, level: ControlLevel) -> Self {
        Self {
            domain,
            chosen_method: Some(method.id.clone()),
            level,
        }
    }
    pub fn observe_fallback(domain: ControlDomain) -> Self {
        Self {
            domain,
            chosen_method: None,
            level: ControlLevel::Observe,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PolicyFeasibilityResult {
    pub policy_id: String,
    pub per_domain: Vec<DomainFeasibility>,
}

impl PolicyFeasibilityResult {
    pub fn build(
        policy: &Policy,
        _achievable: ControlLevel,
        per_domain: Vec<DomainFeasibility>,
        _gaps: Vec<CapabilityUpgrade>,
    ) -> Self {
        Self {
            policy_id: policy.id.clone(),
            per_domain,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MethodBinding {
    pub domain: ControlDomain,
    pub method_id: String,
    pub effective_level: ControlLevel,
    pub maturity: String,
}

#[derive(Clone, Debug)]
pub struct ControlMethodPlan {
    pub policy_id: String,
    pub bindings: Vec<MethodBinding>,
    pub fallbacks: Vec<String>,
    pub auto_selected: bool,
}

/// 1) Feasibility planner — policy นี้ทำได้จริงแค่ไหนบนเครื่องนี้
pub fn assess_feasibility(policy: &Policy, snap: &LocalCapabilitySnapshot) -> PolicyFeasibilityResult {
    let mut per_domain = vec![]; let mut gaps = vec![]; let mut achievable = ControlLevel::Enforce;
    for domain in policy.required_domains() {
        match select_method(domain.clone(), policy.requested_level.clone(), snap) {
            Some(m) => {
                let lvl = m.max_level.clone().min(policy.requested_level.clone());  // negotiate ลงตามจริง
                achievable = achievable.min(lvl.clone());
                per_domain.push(DomainFeasibility::ok(domain, &m, lvl));
            }
            None => {
                achievable = ControlLevel::Observe.min(achievable);
                if let Some(u) = snap.upgrade_for(domain.clone()) { gaps.push(u.clone()); }
                per_domain.push(DomainFeasibility::observe_fallback(domain));
            }
        }
    }
    PolicyFeasibilityResult::build(policy, achievable, per_domain, gaps)
}

/// 2) Control method selector — เลือกวิธีคุมที่ "ดีที่สุดที่ทำได้จริง" ต่อ domain (capability-aware)
fn select_method<'a>(domain: ControlDomain, want: ControlLevel, snap: &'a LocalCapabilitySnapshot)
    -> Option<&'a ControlMethodCap> {
    let pref: &[&str] = match domain {
        ControlDomain::Network    => &["linux_ebpf","macos_netext","windows_wfp_um","mcp_http"],
        ControlDomain::FileSystem => &["linux_landlock","macos_es","mcp_stdio"],
        ControlDomain::McpTool    => &["mcp_stdio","mcp_http"],
        ControlDomain::Process    => &["linux_ebpf","macos_es","windows_etw"],
        ControlDomain::Dns        => &["macos_netext","linux_ebpf","mcp_http"],
    };
    pref.iter().find_map(|id| snap.control_methods.iter().find(|m|
        &m.id == id && m.status == MethodStatus::Available
        && m.domains.contains(&domain) && m.max_level >= want.min_observe()))
      .or_else(|| snap.observe_capable(domain)) // ไม่มี enforce → observe
}

/// 3) Control level negotiation — สรุประดับที่ได้จริง; ห้าม downgrade เงียบ
pub fn negotiate(r: &PolicyFeasibilityResult) -> ControlMethodPlan {
    ControlMethodPlan {
        policy_id: r.policy_id.clone(),
        bindings: r.per_domain.iter().filter_map(|d| d.chosen_method.clone()
            .map(|mid| MethodBinding { domain: d.domain.clone(), method_id: mid,
                effective_level: d.level.clone(), maturity: String::new() })).collect(),
        fallbacks: r.per_domain.iter().filter(|d| d.chosen_method.is_none())
            .map(|d| format!("{}: observe (no enforce method)", d.domain)).collect(),
        auto_selected: true,
    }
}
