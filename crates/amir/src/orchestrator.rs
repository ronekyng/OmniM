use crate::types::*;
use std::collections::HashMap;

/// Orchestrator — 多智能体编排器
///
/// 将用户意图经过三步处理：
/// 1. decompose — 跨领域任务自动分解
/// 2. detect_conflicts — 冲突检测
/// 3. plan — 多步执行规划
pub struct Orchestrator {
    domain_templates: HashMap<Domain, String>,
}

impl Orchestrator {
    pub fn new() -> Self {
        let mut domain_templates = HashMap::new();

        domain_templates.insert(
            Domain::Building,
            "You are a building engineering expert. Focus on walls, beams, columns, slabs, doors, windows, stairs, and roofs.".into(),
        );
        domain_templates.insert(
            Domain::Civil,
            "You are a civil engineering expert. Focus on roads, bridges, tunnels, dams, culverts, and retaining walls.".into(),
        );
        domain_templates.insert(
            Domain::Plant,
            "You are a plant design expert. Focus on tanks, pumps, valves, heat exchangers, reactors, pipes, and structural steel.".into(),
        );
        domain_templates.insert(
            Domain::Mechanical,
            "You are a mechanical engineering expert. Focus on parts, assemblies, fasteners, gears, shafts, bearings, and motors.".into(),
        );
        domain_templates.insert(
            Domain::General,
            "You are a general engineering expert. Handle all engineering domains.".into(),
        );

        Self { domain_templates }
    }

    pub fn get_domain_prompt(&self, domain: &Domain) -> &str {
        self.domain_templates
            .get(domain)
            .unwrap_or(&self.domain_templates[&Domain::General])
    }

    pub fn route(&self, intent: &ParsedIntent) -> DomainRouting {
        DomainRouting::Specific(intent.domain.clone())
    }

    /// 分解：检测单个意图是否涉及多领域，必要时拆分为子意图
    pub fn decompose(&self, intent: &ParsedIntent) -> DecomposedIntent {
        let text = intent.original_text.to_lowercase();
        let mut domains = Vec::new();
        let keywords: Vec<(&str, Domain)> = vec![
            ("wall", Domain::Building),
            ("beam", Domain::Building),
            ("column", Domain::Building),
            ("floor", Domain::Building),
            ("building", Domain::Building),
            ("door", Domain::Building),
            ("window", Domain::Building),
            ("road", Domain::Civil),
            ("bridge", Domain::Civil),
            ("tunnel", Domain::Civil),
            ("dam", Domain::Civil),
            ("pipe", Domain::Plant),
            ("tank", Domain::Plant),
            ("pump", Domain::Plant),
            ("valve", Domain::Plant),
            ("gear", Domain::Mechanical),
            ("shaft", Domain::Mechanical),
            ("bearing", Domain::Mechanical),
            ("motor", Domain::Mechanical),
        ];

        for (kw, d) in &keywords {
            if text.contains(kw) && !domains.contains(d) {
                domains.push(d.clone());
            }
        }

        if domains.is_empty() || domains.len() == 1 {
            return DecomposedIntent {
                sub_intents: vec![intent.clone()],
                domains: vec![intent.domain.clone()],
            };
        }

        let sub_intents = domains
            .iter()
            .map(|d| {
                let mut sub = intent.clone();
                sub.domain = d.clone();
                sub
            })
            .collect();

        DecomposedIntent {
            sub_intents,
            domains,
        }
    }

    /// 冲突检测：检查一组意图之间的冲突
    pub fn detect_conflicts(&self, intents: &[ParsedIntent]) -> Vec<Conflict> {
        let mut conflicts = Vec::new();
        let mut create_targets: HashMap<String, Vec<usize>> = HashMap::new();
        let mut delete_targets: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, intent) in intents.iter().enumerate() {
            match &intent.intent {
                Intent::CreateObject { name: Some(n), .. } => {
                    create_targets.entry(n.clone()).or_default().push(i);
                }
                Intent::DeleteObject { target_ref } => {
                    delete_targets.entry(target_ref.clone()).or_default().push(i);
                }
                _ => {}
            }
        }

        for (name, create_indices) in &create_targets {
            if let Some(delete_indices) = delete_targets.get(name) {
                let mut involved = create_indices.clone();
                involved.extend(delete_indices.clone());
                conflicts.push(Conflict {
                    severity: ConflictSeverity::Warning,
                    message: format!(
                        "Create and delete both reference '{}' — possible ambiguity",
                        name
                    ),
                    involved_intents: involved,
                });
            }
        }

        conflicts
    }

    /// 规划：确定意图的执行顺序
    pub fn plan(&self, intents: &[ParsedIntent]) -> Plan {
        let mut steps = Vec::new();
        let mut ordering_notes = Vec::new();

        let mut deferred: Vec<usize> = Vec::new();

        for (i, intent) in intents.iter().enumerate() {
            match &intent.intent {
                Intent::CreateObject { .. } => {
                    steps.push(PlanStepKind::Execute { intent_index: i });
                }
                Intent::SetProperty { .. } => {
                    steps.push(PlanStepKind::Execute { intent_index: i });
                }
                Intent::AddContainment { .. } => {
                    deferred.push(i);
                    ordering_notes.push(format!(
                        "Intent {} (containment) deferred — parent/child must exist first",
                        i
                    ));
                }
                Intent::DeleteObject { .. } => {
                    steps.push(PlanStepKind::Execute { intent_index: i });
                    ordering_notes.push(format!(
                        "Intent {} (delete) placed first to avoid resource conflicts",
                        i
                    ));
                }
                _ => {
                    steps.push(PlanStepKind::Execute { intent_index: i });
                }
            }
        }

        for i in deferred {
            steps.push(PlanStepKind::Execute { intent_index: i });
        }

        Plan {
            steps,
            ordering_notes,
        }
    }

    /// 完整编排：分解 → 冲突检测 → 规划
    pub fn orchestrate(
        &self,
        intents: &[ParsedIntent],
    ) -> Result<(Vec<ParsedIntent>, Vec<Conflict>, Plan), AmirError> {
        let mut all_intents = Vec::new();
        for intent in intents {
            let decomposed = self.decompose(intent);
            all_intents.extend(decomposed.sub_intents);
        }

        let conflicts = self.detect_conflicts(&all_intents);
        for c in &conflicts {
            if c.severity == ConflictSeverity::Error {
                return Err(AmirError::ConflictError(c.message.clone()));
            }
        }

        let plan = self.plan(&all_intents);

        let mut ordered = Vec::new();
        for step in &plan.steps {
            if let PlanStepKind::Execute { intent_index } = step {
                if *intent_index < all_intents.len() {
                    ordered.push(all_intents[*intent_index].clone());
                }
            }
        }

        Ok((ordered, conflicts, plan))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DomainRouting {
    Specific(Domain),
    All,
    None,
}