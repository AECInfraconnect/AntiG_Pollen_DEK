// SPDX-License-Identifier: Apache-2.0
use crate::model::*;
use regex::Regex;
use tracing::warn;

pub struct PrecompiledVendor {
    pub ns: Vec<String>,
    pub vendor: String,
    pub license_class: Option<String>,
    pub flags: Vec<String>,
}

pub struct PrecompiledFamilyRule {
    pub id: String,
    pub re: Regex,
    pub family: String,
    pub vendor: Option<String>,
    pub tags: Vec<String>,
    pub risk_base: f64,
}

pub struct PrecompiledPopularModel {
    pub re: Regex,
    pub def: PopularModelDef,
}

pub struct PrecompiledRiskFlag {
    pub re: Regex,
    pub flag: String,
    pub risk_add: f64,
    pub tags: Vec<String>,
    pub note: Option<String>,
}

pub struct ModelClassifier {
    vendors: Vec<PrecompiledVendor>,
    family_rules: Vec<PrecompiledFamilyRule>,
    popular_models: Vec<PrecompiledPopularModel>,
    risk_flags: Vec<PrecompiledRiskFlag>,

    // Attribute parsers
    params_b_re: Option<Regex>,
    params_m_re: Option<Regex>,
    moe_active_b_re: Option<Regex>,
    moe_experts_re: Option<Regex>,
    context_hint_re: Option<Regex>,
    quant_re: Option<Regex>,
    variant_res: Vec<(String, Regex)>,
}

impl ModelClassifier {
    pub fn new(def: &ModelClassifierDef) -> Self {
        // Compile vendors
        let vendors = def
            .vendors
            .iter()
            .map(|v| PrecompiledVendor {
                ns: v.ns.clone(),
                vendor: v.vendor.clone(),
                license_class: v.license_class.clone(),
                flags: v.flags.clone(),
            })
            .collect();

        // Compile family rules
        let family_rules = def
            .family_rules
            .iter()
            .filter_map(|r| match Regex::new(&r.pattern) {
                Ok(re) => Some(PrecompiledFamilyRule {
                    id: r.id.clone(),
                    re,
                    family: r.family.clone(),
                    vendor: r.vendor.clone(),
                    tags: r.tags.clone(),
                    risk_base: r.risk_base,
                }),
                Err(e) => {
                    warn!("Failed to compile family_rule regex for '{}': {}", r.id, e);
                    None
                }
            })
            .collect();

        // Compile popular models
        let popular_models = def
            .popular_models
            .iter()
            .filter_map(|p| {
                let pattern = p.match_.as_ref().unwrap_or(&p.match_pattern);
                match Regex::new(pattern) {
                    Ok(re) => Some(PrecompiledPopularModel { re, def: p.clone() }),
                    Err(e) => {
                        warn!(
                            "Failed to compile popular_model regex for '{}': {}",
                            p.display, e
                        );
                        None
                    }
                }
            })
            .collect();

        // Compile risk flags
        let risk_flags = def
            .risk_flags
            .iter()
            .filter_map(|r| match Regex::new(&r.pattern) {
                Ok(re) => Some(PrecompiledRiskFlag {
                    re,
                    flag: r.flag.clone(),
                    risk_add: r.risk_add,
                    tags: r.tags.clone(),
                    note: r.note.clone(),
                }),
                Err(e) => {
                    warn!("Failed to compile risk_flag regex for '{}': {}", r.flag, e);
                    None
                }
            })
            .collect();

        // Compile attribute parsers
        let mut params_b_re = None;
        let mut params_m_re = None;
        let mut moe_active_b_re = None;
        let mut moe_experts_re = None;
        let mut context_hint_re = None;
        let mut quant_re = None;
        let mut variant_res = Vec::new();

        if let Some(AttributeParserDef::String(s)) = def.attribute_parsers.get("params_b") {
            params_b_re = Regex::new(s).ok();
        }
        if let Some(AttributeParserDef::String(s)) = def.attribute_parsers.get("params_m") {
            params_m_re = Regex::new(s).ok();
        }
        if let Some(AttributeParserDef::String(s)) = def.attribute_parsers.get("moe_active_b") {
            moe_active_b_re = Regex::new(s).ok();
        }
        if let Some(AttributeParserDef::String(s)) = def.attribute_parsers.get("moe_experts") {
            moe_experts_re = Regex::new(s).ok();
        }
        if let Some(AttributeParserDef::String(s)) = def.attribute_parsers.get("context_hint") {
            context_hint_re = Regex::new(s).ok();
        }
        if let Some(AttributeParserDef::String(s)) = def.attribute_parsers.get("quant") {
            quant_re = Regex::new(s).ok();
        }
        if let Some(AttributeParserDef::Map(variants)) = def.attribute_parsers.get("variant") {
            for (k, v) in variants {
                if let Ok(re) = Regex::new(v) {
                    variant_res.push((k.clone(), re));
                }
            }
        }

        Self {
            vendors,
            family_rules,
            popular_models,
            risk_flags,
            params_b_re,
            params_m_re,
            moe_active_b_re,
            moe_experts_re,
            context_hint_re,
            quant_re,
            variant_res,
        }
    }

    pub fn classify(&self, raw_id: &str) -> ModelClass {
        let id = raw_id.trim();

        let (vendor, vendor_flags) = self.lookup_vendor(id);

        if let Some(p) = self.match_popular(id) {
            return self.finalize(id, p, vendor, vendor_flags);
        }

        let base = match self.match_family(id) {
            Some(f) => f,
            None => ClassBase::unknown(vendor.clone()),
        };

        self.finalize(id, base, vendor, vendor_flags)
    }

    fn lookup_vendor(&self, id: &str) -> (Option<String>, Vec<String>) {
        let prefix = id.split('/').next().unwrap_or("").to_lowercase();
        for v in &self.vendors {
            if v.ns.iter().any(|ns| ns.to_lowercase() == prefix) {
                return (Some(v.vendor.clone()), v.flags.clone());
            }
        }
        (None, vec![])
    }

    fn match_popular(&self, id: &str) -> Option<ClassBase> {
        for p in &self.popular_models {
            if p.re.is_match(id) {
                return Some(ClassBase {
                    display: p.def.display.clone(),
                    family: p.def.family.clone(),
                    vendor: p.def.vendor.clone(),
                    license: p.def.license.clone(),
                    arch: p.def.arch.clone(),
                    params_total_b: p.def.params_total_b,
                    params_active_b: p.def.params_active_b,
                    context: p.def.context,
                    modality: p.def.modality.clone(),
                    quant: None,
                    variant: vec![],
                    capability_tags: p.def.tags.clone(),
                    risk_base: p.def.risk_base,
                    flags: p.def.flags.clone(),
                    matched_tier: "popular",
                });
            }
        }
        None
    }

    fn match_family(&self, id: &str) -> Option<ClassBase> {
        for f in &self.family_rules {
            if f.re.is_match(id) {
                return Some(ClassBase {
                    display: format!("{} (Unknown)", f.family),
                    family: f.family.clone(),
                    vendor: f.vendor.clone(),
                    license: None,
                    arch: None,
                    params_total_b: None,
                    params_active_b: None,
                    context: None,
                    modality: vec!["text".into()],
                    quant: None,
                    variant: vec![],
                    capability_tags: f.tags.clone(),
                    risk_base: f.risk_base,
                    flags: vec![],
                    matched_tier: "family",
                });
            }
        }
        None
    }

    fn parse_params_b(&self, id: &str) -> Option<f64> {
        if let Some(re) = &self.params_b_re {
            if let Some(cap) = re.captures(id) {
                if let Some(m) = cap.get(1) {
                    return m.as_str().parse().ok();
                }
            }
        }
        if let Some(re) = &self.params_m_re {
            if let Some(cap) = re.captures(id) {
                if let Some(m) = cap.get(1) {
                    if let Ok(val) = m.as_str().parse::<f64>() {
                        return Some(val / 1000.0);
                    }
                }
            }
        }
        None
    }

    fn parse_active_b(&self, id: &str) -> Option<f64> {
        if let Some(re) = &self.moe_active_b_re {
            if let Some(cap) = re.captures(id) {
                if let Some(m) = cap.get(1) {
                    return m.as_str().parse().ok();
                }
            }
        }
        None
    }

    fn parse_quant(&self, id: &str) -> Option<String> {
        if let Some(re) = &self.quant_re {
            if let Some(cap) = re.captures(id) {
                if let Some(m) = cap.get(1) {
                    return Some(m.as_str().to_lowercase());
                }
            }
        }
        None
    }

    fn apply_variant_tags(&self, base: &mut ClassBase, variant: &str) {
        let tag = match variant {
            "instruct" => "tool.use",
            "reasoning" => "reasoning.cot",
            "coder" => "code.generation",
            "vision" => "vision.input",
            "audio" => "audio.input",
            _ => return,
        };
        if !base.capability_tags.contains(&tag.to_string()) {
            base.capability_tags.push(tag.to_string());
        }
    }

    fn finalize(
        &self,
        id: &str,
        mut base: ClassBase,
        vendor: Option<String>,
        vflags: Vec<String>,
    ) -> ModelClass {
        if base.params_total_b.is_none() {
            base.params_total_b = self.parse_params_b(id);
        }
        if base.params_active_b.is_none() {
            base.params_active_b = self.parse_active_b(id);
        }
        if base.params_active_b.is_some() && base.arch.is_none() {
            base.arch = Some("moe".into());
        }
        if base.quant.is_none() {
            base.quant = self.parse_quant(id);
        }

        for (name, re) in &self.variant_res {
            if re.is_match(id) {
                base.variant.push(name.clone());
                self.apply_variant_tags(&mut base, name);
            }
        }

        let mut risk = base.risk_base;
        for rf in &self.risk_flags {
            if rf.re.is_match(id) {
                risk = (risk + rf.risk_add).clamp(0.0, 1.0);
                for tag in &rf.tags {
                    if !base.capability_tags.contains(tag) {
                        base.capability_tags.push(tag.clone());
                    }
                }
                if !base.flags.contains(&rf.flag) {
                    base.flags.push(rf.flag.clone());
                }
            }
        }

        for flag in vflags {
            if !base.flags.contains(&flag) {
                base.flags.push(flag);
            }
        }

        let needs_human = base.matched_tier == "unknown" || risk >= 0.7;

        ModelClass {
            raw_id: id.into(),
            display: base.display,
            vendor: base.vendor.or(vendor),
            family: base.family,
            license: base.license,
            arch: base.arch,
            params_total_b: base.params_total_b,
            params_active_b: base.params_active_b,
            context: base.context,
            modality: base.modality,
            quant: base.quant,
            variant: base.variant,
            capability_tags: base.capability_tags,
            risk_score: risk,
            flags: base.flags,
            matched_tier: base.matched_tier,
            needs_human,
        }
    }
}
