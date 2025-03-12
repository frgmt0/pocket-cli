//! Graph visualization for Pocket VCS
//! 
//! This module provides functionality to generate a visual representation
//! of the repository's timeline history.

use std::path::Path;
use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow};
use owo_colors::OwoColorize;

use crate::vcs::{Repository, Timeline, Shove, ShoveId};

/// Represents a node in the timeline graph
#[derive(Debug)]
struct GraphNode {
    /// The ID of the shove (commit)
    id: ShoveId,
    /// The commit message
    message: String,
    /// IDs of parent nodes
    parents: Vec<ShoveId>,
    /// IDs of child nodes
    children: Vec<ShoveId>,
    /// Timelines that include this node
    timelines: Vec<String>,
}

/// Represents the entire graph structure
#[derive(Debug)]
struct Graph {
    /// All nodes in the graph, indexed by ID
    nodes: HashMap<ShoveId, GraphNode>,
    /// Root nodes (nodes with no parents)
    roots: Vec<ShoveId>,
    /// Leaf nodes (nodes with no children)
    leaves: Vec<ShoveId>,
    /// All timelines in the repository
    timelines: HashMap<String, ShoveId>,
}

impl Graph {
    /// Creates a new graph from the repository
    /// 
    /// # Arguments
    /// 
    /// * `repo` - The repository to generate the graph from
    /// 
    /// # Returns
    /// 
    /// A new Graph instance with all nodes and relationships loaded
    fn new(repo: &Repository) -> anyhow::Result<Self> {
        let mut nodes = HashMap::new();
        let mut roots = Vec::new();
        let mut leaves = Vec::new();
        let mut timelines = HashMap::new();
        
        // Load all timelines
        for timeline_entry in repo.list_timelines()? {
            let timeline = repo.load_timeline(&timeline_entry.name)?;
            timelines.insert(timeline_entry.name.clone(), timeline.head.clone());
        }
        
        // Load all shoves and create nodes
        for timeline_name in timelines.keys() {
            let mut current_id = timelines.get(timeline_name).unwrap().clone();
            let mut visited = HashSet::new();
            
            while !visited.contains(&current_id) {
                visited.insert(current_id.clone());
                
                if !nodes.contains_key(&current_id) {
                    let shove = repo.load_shove(&current_id)?;
                    
                    let node = GraphNode {
                        id: current_id.clone(),
                        message: shove.message.clone(),
                        parents: shove.parents.clone(),
                        children: Vec::new(),
                        timelines: vec![timeline_name.clone()],
                    };
                    
                    nodes.insert(current_id.clone(), node);
                    
                    if shove.parents.is_empty() {
                        roots.push(current_id.clone());
                    }
                } else {
                    // Add this timeline to the node's timelines
                    if let Some(node) = nodes.get_mut(&current_id) {
                        if !node.timelines.contains(timeline_name) {
                            node.timelines.push(timeline_name.clone());
                        }
                    }
                }
                
                // Move to parent
                let node = nodes.get(&current_id).unwrap();
                if node.parents.is_empty() {
                    break;
                }
                
                current_id = node.parents[0].clone();
            }
        }
        
        // Build child relationships
        let mut child_parent_relations = Vec::new();
        for (id, node) in &nodes {
            for parent_id in &node.parents {
                child_parent_relations.push((parent_id.clone(), id.clone()));
            }
        }
        
        // Update children for each parent
        for (parent_id, child_id) in child_parent_relations {
            if let Some(parent) = nodes.get_mut(&parent_id) {
                parent.children.push(child_id);
            }
        }
        
        // Find leaf nodes (nodes with no children)
        for (id, node) in &nodes {
            if node.children.is_empty() {
                leaves.push(id.clone());
            }
        }
        
        Ok(Self {
            nodes,
            roots,
            leaves,
            timelines,
        })
    }
    
    /// Generates a visual representation of the graph
    /// 
    /// # Returns
    /// 
    /// A vector of strings representing the graph visualization
    fn visualize(&self) -> Vec<String> {
        let mut lines = Vec::new();
        
        if self.nodes.is_empty() {
            return vec!["No commits found in repository.".to_string()];
        }
        
        // Display timeline heads
        lines.push("Timeline Heads:".to_string());
        for (timeline_name, head_id) in &self.timelines {
            if let Some(node) = self.nodes.get(head_id) {
                lines.push(format!("  🌿 {}", timeline_name.bright_green()));
                lines.push(format!("     └── {} {}", 
                        head_id.as_str()[0..8].bright_yellow(), 
                        node.message.bright_white()));
            }
        }
        
        lines.push("\nCommit Graph:".to_string());
        
        // Start from roots and traverse
        let mut visited = HashSet::new();
        for root_id in &self.roots {
            self.traverse_node(root_id, "", &mut lines, &mut visited, true);
        }
        
        lines
    }
    
    /// Recursively traverses the graph to generate visualization
    /// 
    /// # Arguments
    /// 
    /// * `node_id` - The ID of the current node
    /// * `prefix` - The prefix to use for this line (for indentation)
    /// * `lines` - The vector to add lines to
    /// * `visited` - Set of already visited nodes to prevent cycles
    /// * `is_last` - Whether this is the last child of its parent
    fn traverse_node(&self, node_id: &ShoveId, prefix: &str, lines: &mut Vec<String>, visited: &mut HashSet<ShoveId>, is_last: bool) {
        if visited.contains(node_id) {
            return;
        }
        
        visited.insert(node_id.clone());
        
        let node = match self.nodes.get(node_id) {
            Some(n) => n,
            None => return,
        };
        
        // Generate the line for this node
        let branch_symbol = if is_last { "└── " } else { "├── " };
        let timeline_indicators = if !node.timelines.is_empty() {
            format!("[{}] ", node.timelines.join(", "))
        } else {
            "".to_string()
        };
        
        lines.push(format!("{}{}{}{} {}", 
            prefix, 
            branch_symbol, 
            timeline_indicators,
            node_id.as_str()[0..8].bright_yellow(), 
            node.message.bright_white()));
        
        // Generate lines for children
        let child_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };
        
        let children = &node.children;
        for (i, child_id) in children.iter().enumerate() {
            let is_last_child = i == children.len() - 1;
            self.traverse_node(child_id, &child_prefix, lines, visited, is_last_child);
        }
        
        // If this node has parents that haven't been visited, show them too
        for (i, parent_id) in node.parents.iter().enumerate() {
            if !visited.contains(parent_id) {
                let parent = match self.nodes.get(parent_id) {
                    Some(p) => p,
                    None => continue,
                };
                
                let is_last_parent = i == node.parents.len() - 1;
                lines.push(format!("{}    ↑ {} {}", 
                    prefix,
                    parent_id.as_str()[0..8].bright_yellow(), 
                    parent.message.bright_white()));
            }
        }
    }
}

/// Generates a graph visualization for the repository
/// 
/// # Arguments
/// 
/// * `repo_path` - Path to the repository
/// 
/// # Returns
/// 
/// A vector of strings representing the graph visualization
pub fn generate_graph(repo_path: &Path) -> anyhow::Result<Vec<String>> {
    let repo = Repository::open(repo_path)?;
    let graph = Graph::new(&repo)?;
    Ok(graph.visualize())
} 