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
    for (name, package) in &workspace.packages {
        let from_index = node_indices.get(name).unwrap();

        for dep in &package.dependencies {
            // Only add edges for dependencies that are also in the workspace
            if let Some(to_index) = node_indices.get(dep) {
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
        Err(_) => {
            // Find circular dependencies for better error reporting
            let cycles = find_cycles(&graph, &node_indices);
            Err(CargoError::CircularDependency(format!("Circular dependencies detected: {:?}", cycles)))
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
