//! Dependency Graph & Topological Resolution for Ind OS Init
//!
//! Constitutional References:
//! - REQ-USE-201: Dependency graph resolution
//! - INV-INIT-001: Deterministic Dependency Ordering (zero cycles, strict prerequisites)
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code

use crate::manifest::ServiceManifest;

pub const MAX_SERVICES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitError {
    DependencyCycleDetected,
    MissingDependency,
    MaxServicesExceeded,
    InvalidManifest,
    ServiceNotFound,
}

pub struct DependencyGraph {
    services: [Option<ServiceManifest>; MAX_SERVICES],
    count: usize,
}

impl DependencyGraph {
    pub const fn new() -> Self {
        Self {
            services: [None; MAX_SERVICES],
            count: 0,
        }
    }

    pub fn add_service(&mut self, manifest: ServiceManifest) -> Result<(), InitError> {
        if !manifest.validate() {
            return Err(InitError::InvalidManifest);
        }
        if self.count >= MAX_SERVICES {
            return Err(InitError::MaxServicesExceeded);
        }
        self.services[self.count] = Some(manifest);
        self.count += 1;
        Ok(())
    }

    pub fn get_service(&self, name: &str) -> Option<&ServiceManifest> {
        for slot in self.services.iter().take(self.count) {
            if let Some(m) = slot {
                if m.name == name {
                    return Some(m);
                }
            }
        }
        None
    }

    pub fn service_count(&self) -> usize {
        self.count
    }

    /// Resolves topological execution order using Kahn's algorithm.
    /// Detects cycles and verifies that all dependencies exist (`INV-INIT-001`).
    pub fn resolve_order(&self, order: &mut [&'static str; MAX_SERVICES]) -> Result<usize, InitError> {
        // 1. Verify all declared dependencies exist in the graph
        for i in 0..self.count {
            if let Some(manifest) = self.services[i] {
                for &dep in manifest.dependencies {
                    if self.get_service(dep).is_none() {
                        return Err(InitError::MissingDependency);
                    }
                }
            }
        }

        // 2. Compute in-degrees (number of unmet prerequisites)
        let mut in_degree = [0usize; MAX_SERVICES];
        for i in 0..self.count {
            if let Some(manifest) = self.services[i] {
                in_degree[i] = manifest.dependencies.len();
            }
        }

        // 3. Queue nodes with 0 in-degrees
        let mut queue = [0usize; MAX_SERVICES];
        let mut q_head = 0;
        let mut q_tail = 0;

        for i in 0..self.count {
            if in_degree[i] == 0 {
                queue[q_tail] = i;
                q_tail += 1;
            }
        }

        let mut sorted_count = 0;

        while q_head < q_tail {
            let u_idx = queue[q_head];
            q_head += 1;

            let u_name = self.services[u_idx].unwrap().name;
            order[sorted_count] = u_name;
            sorted_count += 1;

            // For every service v that depends on u_name, decrement in_degree[v]
            for v_idx in 0..self.count {
                if let Some(v_manifest) = self.services[v_idx] {
                    for &dep in v_manifest.dependencies {
                        if dep == u_name {
                            in_degree[v_idx] -= 1;
                            if in_degree[v_idx] == 0 {
                                queue[q_tail] = v_idx;
                                q_tail += 1;
                            }
                        }
                    }
                }
            }
        }

        // 4. Cycle detection: if sorted_count != self.count, a cycle exists
        if sorted_count != self.count {
            return Err(InitError::DependencyCycleDetected);
        }

        Ok(sorted_count)
    }
}
