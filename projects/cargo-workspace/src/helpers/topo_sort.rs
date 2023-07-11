use crate::{
    errors::{CargoError, Result},
    helpers::workspace::{CargoPackage, CargoWorkspace},
};
use petgraph::{Directed, Graph, algo::toposort};
use std::collections::{HashMap, HashSet};

/// Performs topological sort on workspace packages based on their dependencies
pub fn topological_sort(workspace: &CargoWorkspace) -> Result<Vec<CargoPackage>> {
    let mut graph: Graph<String, (), Directed> = Graph::new();
    let mut node_indices: HashMap<String, petgraph::prelude::NodeIndex> = HashMap::new();

    // Add all packages as nodes
    for (name, _package) in &workspace.packages {
        let index = graph.add_node(name.clone());
        node_indices.insert(name.clone(), index);
    }

    // Add edges based on dependencies
    // Edge direction: dependency -> dependent (so dependencies come first in topological order)
    for (name, package) in &workspace.packages {
        let to_index = node_indices.get(name).unwrap();

        for dep in &package.dependencies {
            // Only add edges for dependencies that are also in the workspace
            if let Some(from_index) = node_indices.get(dep) {
                graph.add_edge(*from_index, *to_index, ());
            }
        }
    }

    // Perform topological sort
    match toposort(&graph, None) {
        Ok(sorted_indices) => {
            let mut sorted_packages = Vec::new();
            for index in sorted_indices {
                let package_name = &graph[index];
                if let Some(package) = workspace.packages.get(package_name) {
                    sorted_packages.push(package.clone());
                }
            }
            Ok(sorted_packages)
        }
        Err(cycle_error) => {
            // Use petgraph's cycle detection to get the actual cycle
            use petgraph::algo::is_cyclic_directed;
            if is_cyclic_directed(&graph) {
                // Find the actual cycle using strongly connected components
                use petgraph::algo::tarjan_scc;
                let sccs = tarjan_scc(&graph);
                let mut cycles = Vec::new();
                
                for scc in sccs {
                    if scc.len() > 1 {
                        let cycle_names: Vec<String> = scc.iter()
                            .map(|&idx| graph[idx].clone())
                            .collect();
                        cycles.push(cycle_names.join(" -> "));
                    }
                }
                
                Err(CargoError::CircularDependency(format!("Circular dependencies detected: {:?}", cycles)))
            } else {
                Err(CargoError::CircularDependency("Topological sort failed but no cycles detected".to_string()))
            }
        }
    }
}

/// Helper function to find cycles in the dependency graph
fn find_cycles(
    graph: &Graph<String, (), Directed>,
    node_indices: &HashMap<String, petgraph::prelude::NodeIndex>,
) -> Vec<String> {
    use petgraph::visit::Dfs;

    let mut cycles = Vec::new();
    let mut visited = HashSet::new();

    for (name, index) in node_indices {
        if !visited.contains(name) {
            let mut dfs = Dfs::new(graph, *index);
            let mut path = Vec::new();
            let mut path_set = HashSet::new();

            while let Some(nx) = dfs.next(graph) {
                let node_name = &graph[nx];

                if path_set.contains(node_name) {
                    // Found a cycle
                    if let Some(pos) = path.iter().position(|n| n == node_name) {
                        let cycle = path[pos..].join(" -> ");
                        cycles.push(format!("{} -> {}", cycle, node_name));
                    }
                    break;
                }

                path.push(node_name.clone());
                path_set.insert(node_name.clone());
                visited.insert(node_name.clone());
            }
        }
    }

    cycles
}

/// Filters packages based on whether they should be published
pub fn filter_publishable_packages(packages: Vec<CargoPackage>) -> Vec<CargoPackage> {
    packages.into_iter().filter(|p| p.publish).collect()
}
