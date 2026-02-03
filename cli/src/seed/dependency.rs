use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet, VecDeque};

/// Resolves table dependencies and determines seeding order
pub struct DependencyResolver {
    /// Map of table -> list of tables it depends on (foreign keys point to these)
    dependencies: HashMap<String, HashSet<String>>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    /// Add a dependency: `table` depends on `depends_on`
    pub fn add_dependency(&mut self, table: String, depends_on: String) {
        // Don't add self-referential dependencies to the graph
        // (they'll be handled specially during seeding)
        if table != depends_on {
            self.dependencies
                .entry(table)
                .or_insert_with(HashSet::new)
                .insert(depends_on);
        }
    }

    /// Ensure all tables are in the dependency map (even if they have no dependencies)
    pub fn add_table(&mut self, table: String) {
        self.dependencies.entry(table).or_insert_with(HashSet::new);
    }

    /// Resolve the seeding order using topological sort
    /// Returns a Vec of "levels" where tables in the same level can be seeded in parallel
    pub fn resolve_order(&self, tables: &[String]) -> Result<Vec<Vec<String>>> {
        // Check for cycles first
        if self.has_cycle(tables)? {
            return Err(anyhow!("Circular dependency detected in table relationships"));
        }

        // Build the graph with only requested tables
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        for table in tables {
            graph.insert(table.clone(), HashSet::new());
            in_degree.insert(table.clone(), 0);
        }

        // Add edges (only for tables in our set)
        for table in tables {
            if let Some(deps) = self.dependencies.get(table) {
                for dep in deps {
                    if tables.contains(dep) {
                        graph.get_mut(dep).unwrap().insert(table.clone());
                        *in_degree.get_mut(table).unwrap() += 1;
                    }
                }
            }
        }

        // Kahn's algorithm for topological sort
        let mut levels = Vec::new();
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(table, _)| table.clone())
            .collect();

        while !queue.is_empty() {
            let mut current_level = Vec::new();

            // Process all nodes at current level
            for _ in 0..queue.len() {
                let table = queue.pop_front().unwrap();
                current_level.push(table.clone());

                // Reduce in-degree for dependent tables
                if let Some(dependents) = graph.get(&table) {
                    for dependent in dependents {
                        let degree = in_degree.get_mut(dependent).unwrap();
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }

            if !current_level.is_empty() {
                levels.push(current_level);
            }
        }

        // Verify all tables were processed
        let total_processed: usize = levels.iter().map(|level| level.len()).sum();
        if total_processed != tables.len() {
            return Err(anyhow!("Circular dependency detected in table relationships"));
        }

        Ok(levels)
    }

    /// Check if there's a cycle in the dependency graph
    pub fn has_cycle(&self, tables: &[String]) -> Result<bool> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for table in tables {
            if !visited.contains(table) {
                if self.has_cycle_util(table, tables, &mut visited, &mut rec_stack) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn has_cycle_util(
        &self,
        table: &str,
        tables: &[String],
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(table.to_string());
        rec_stack.insert(table.to_string());

        if let Some(deps) = self.dependencies.get(table) {
            for dep in deps {
                // Only consider dependencies within our table set
                if !tables.contains(dep) {
                    continue;
                }

                if !visited.contains(dep) {
                    if self.has_cycle_util(dep, tables, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(dep) {
                    return true;
                }
            }
        }

        rec_stack.remove(table);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_linear_dependency() {
        let mut resolver = DependencyResolver::new();

        // users -> orders -> order_items (linear chain)
        resolver.add_table("users".to_string());
        resolver.add_dependency("orders".to_string(), "users".to_string());
        resolver.add_dependency("order_items".to_string(), "orders".to_string());

        let tables = vec![
            "users".to_string(),
            "orders".to_string(),
            "order_items".to_string(),
        ];

        let order = resolver.resolve_order(&tables).unwrap();

        assert_eq!(order.len(), 3);
        assert_eq!(order[0], vec!["users"]);
        assert_eq!(order[1], vec!["orders"]);
        assert_eq!(order[2], vec!["order_items"]);
    }

    #[test]
    fn test_parallel_dependencies() {
        let mut resolver = DependencyResolver::new();

        // users -> orders
        // users -> reviews
        // products (independent)
        resolver.add_table("users".to_string());
        resolver.add_table("products".to_string());
        resolver.add_dependency("orders".to_string(), "users".to_string());
        resolver.add_dependency("reviews".to_string(), "users".to_string());

        let tables = vec![
            "users".to_string(),
            "products".to_string(),
            "orders".to_string(),
            "reviews".to_string(),
        ];

        let order = resolver.resolve_order(&tables).unwrap();

        assert_eq!(order.len(), 2);

        // Level 0: users and products (independent)
        assert_eq!(order[0].len(), 2);
        assert!(order[0].contains(&"users".to_string()));
        assert!(order[0].contains(&"products".to_string()));

        // Level 1: orders and reviews (both depend on users)
        assert_eq!(order[1].len(), 2);
        assert!(order[1].contains(&"orders".to_string()));
        assert!(order[1].contains(&"reviews".to_string()));
    }

    #[test]
    fn test_cycle_detection() {
        let mut resolver = DependencyResolver::new();

        // Create a cycle: a -> b -> c -> a
        resolver.add_dependency("a".to_string(), "b".to_string());
        resolver.add_dependency("b".to_string(), "c".to_string());
        resolver.add_dependency("c".to_string(), "a".to_string());

        let tables = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        assert!(resolver.has_cycle(&tables).unwrap());

        let result = resolver.resolve_order(&tables);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency"));
    }

    #[test]
    fn test_self_referential_table() {
        let mut resolver = DependencyResolver::new();

        // Employee table with self-referential manager_id
        resolver.add_dependency("employees".to_string(), "employees".to_string());

        let tables = vec!["employees".to_string()];

        // Self-references are ignored in the graph
        let order = resolver.resolve_order(&tables).unwrap();

        assert_eq!(order.len(), 1);
        assert_eq!(order[0], vec!["employees"]);
    }

    #[test]
    fn test_complex_dependencies() {
        let mut resolver = DependencyResolver::new();

        // users
        // categories (independent)
        // products -> categories
        // orders -> users
        // order_items -> orders, products

        resolver.add_table("users".to_string());
        resolver.add_table("categories".to_string());
        resolver.add_dependency("products".to_string(), "categories".to_string());
        resolver.add_dependency("orders".to_string(), "users".to_string());
        resolver.add_dependency("order_items".to_string(), "orders".to_string());
        resolver.add_dependency("order_items".to_string(), "products".to_string());

        let tables = vec![
            "users".to_string(),
            "categories".to_string(),
            "products".to_string(),
            "orders".to_string(),
            "order_items".to_string(),
        ];

        let order = resolver.resolve_order(&tables).unwrap();

        // Verify correct levels
        assert!(order.len() >= 3);

        // Level 0: users and categories
        assert!(order[0].contains(&"users".to_string()));
        assert!(order[0].contains(&"categories".to_string()));

        // Level 1: products and orders
        assert!(order[1].contains(&"products".to_string()));
        assert!(order[1].contains(&"orders".to_string()));

        // Level 2: order_items (depends on both orders and products)
        assert!(order[2].contains(&"order_items".to_string()));
    }

    #[test]
    fn test_no_dependencies() {
        let mut resolver = DependencyResolver::new();

        resolver.add_table("table1".to_string());
        resolver.add_table("table2".to_string());
        resolver.add_table("table3".to_string());

        let tables = vec![
            "table1".to_string(),
            "table2".to_string(),
            "table3".to_string(),
        ];

        let order = resolver.resolve_order(&tables).unwrap();

        // All tables can be seeded in parallel
        assert_eq!(order.len(), 1);
        assert_eq!(order[0].len(), 3);
    }
}
