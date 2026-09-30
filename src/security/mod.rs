#[derive(Clone, Debug, Default)]
pub struct AuditReport {
    pub passed: bool,
    pub issues: Vec<String>,
}

impl AuditReport {
    pub fn new() -> Self {
        Self {
            passed: true,
            issues: Vec::new(),
        }
    }

    pub fn add_issue(&mut self, issue: String) {
        self.issues.push(issue);
        self.passed = false;
    }
}

#[derive(Clone, Debug, Default)]
pub struct SecurityAudit {
    pub static_checks: bool,
    pub fuzzed: bool,
    pub formal_verified: bool,
}

impl SecurityAudit {
    pub fn new() -> Self {
        Self {
            static_checks: true,
            fuzzed: false,
            formal_verified: false,
        }
    }
}
