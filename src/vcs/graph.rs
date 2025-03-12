//! Timeline graph visualization for Pocket VCS
//!
//! Provides functionality to generate a visual representation of the timeline history.

use std::path::Path;
use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow};
use colored::Colorize;

use crate::vcs::{Repository, Timeline, Shove, ShoveId};

/// Represents a node in the timeline graph
struct GraphNode {
    /// The shove ID
    id: ShoveId,
    /// The shove message (first line)
    message: String,
    /// Parent shove IDs
    parents: Vec<ShoveId>,
    /// Child shove IDs
    children: Vec<ShoveId>,
    /// The timeline(s) this node belongs to
    timelines: Vec<String>,
}

/// Represents the timeline graph
pub struct Graph {
    /// All nodes in the graph
    nodes: HashMap<ShoveId, GraphNode>,
    /// Root nodes (shoves with no parents)
    roots: Vec<ShoveId>,
    /// Leaf nodes (shoves with no children)
    leaves: Vec<ShoveId>,
    /// All timelines in the graph
    timelines: HashMap<String, ShoveId>,
}

impl Graph {
    /// Create a new graph from a repository
    pub fn new(repo: &Repository) -> Result<Self> {
        let mut nodes = HashMap::new();
        let mut roots = Vec::new();
        let mut leaves = HashSet::new();
        let mut timelines = HashMap::new();
        
        // Load all timelines
        let timelines_dir = repo.path.join(".pocket").join("timelines");
        for entry in std::fs::read_dir(timelines_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let filename = entry.file_name();
                let filename_str = filename.to_string_lossy();
                if filename_str.ends_with(".toml") {
                    let timeline_name = filename_str.trim_end_matches(".toml").to_string();
                    let timeline_path = entry.path();
                    let timeline = Timeline::load(&timeline_path)?;
                    
                    if let Some(head) = timeline.head {
                        timelines.insert(timeline_name, head);
                    }
                }
            }
        }
        
        // Load all shoves
        let shoves_dir = repo.path.join(".pocket").join("shoves");
        for entry in std::fs::read_dir(shoves_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let filename = entry.file_name();
                let filename_str = filename.to_string_lossy();
                if filename_str.ends_with(".toml") {
                    let shove_path = entry.path();
                    let shove = Shove::load(&shove_path)?;
                    
                    // Create a graph node
                    let message = shove.message.lines().next().unwrap_or("").to_string();
                    let node = GraphNode {
                        id: shove.id.clone(),
                        message,
                        parents: shove.parent_ids.clone(),
                        children: Vec::new(),
                        timelines: Vec::new(),
                    };
                    
                    // Add to nodes
                    nodes.insert(shove.id.clone(), node);
                    
                    // If no parents, it's a root
                    if shove.parent_ids.is_empty() {
                        roots.push(shove.id.clone());
                    }
                    
                    // Initially, consider all nodes as leaves
                    leaves.insert(shove.id.clone());
                }
            }
        }
        
        // Build parent-child relationships
        // First, collect all the child-parent relationships
        let mut child_parent_relations = Vec::new();
        for (id, node) in &nodes {
            for parent_id in &node.parents {
                child_parent_relations.push((id.clone(), parent_id.clone()));
                // Remove parent from leaves
                leaves.remove(parent_id);
            }
        }
        
        // Now update the children field for each parent
        for (child_id, parent_id) in child_parent_relations {
            if let Some(parent) = nodes.get_mut(&parent_id) {
                parent.children.push(child_id);
            }
        }
        
        // Assign timelines to nodes
        for (timeline_name, head_id) in &timelines {
            if let Some(node) = nodes.get_mut(head_id) {
                node.timelines.push(timeline_name.clone());
            }
            
            // Traverse up the graph to mark all ancestors
            let mut current = head_id.clone();
            let mut visited = HashSet::new();
            visited.insert(current.clone());
            
            while let Some(node) = nodes.get(&current) {
                if node.parents.is_empty() {
                    break;
                }
                
                // Move to the first parent
                if let Some(parent_id) = node.parents.first() {
                    current = parent_id.clone();
                    
                    // Avoid cycles
                    if visited.contains(&current) {
                        break;
                    }
                    visited.insert(current.clone());
                    
                    // Mark this node as part of the timeline
                    if let Some(parent_node) = nodes.get_mut(&current) {
                        if !parent_node.timelines.contains(timeline_name) {
                            parent_node.timelines.push(timeline_name.clone());
                        }
                    }
                } else {
                    break;
                }
            }
        }
        
        Ok(Self {
            nodes,
            roots,
            leaves: leaves.into_iter().collect(),
            timelines,
        })
    }
    
    /// Generate a visual representation of the graph
    pub fn visualize(&self) -> Vec<String> {
        let mut lines = Vec::new();
        
        // Sort timelines alphabetically
        let mut timeline_names: Vec<String> = self.timelines.keys().cloned().collect();
        timeline_names.sort();
        
        // Display timeline heads
        for timeline_name in &timeline_names {
            if let Some(head_id) = self.timelines.get(timeline_name) {
                if let Some(node) = self.nodes.get(head_id) {
                    lines.push(format!("  🌿 {}", timeline_name.bright_green()));
                    lines.push(format!("  ├── 📌 {} {}", 
                        head_id.as_str()[0..8].bright_yellow(), 
                        node.message.bright_white()));
                    
                    // Traverse the graph
                    self.visualize_node(head_id, &mut lines, "  │   ", &HashSet::new());
                }
            }
        }
        
        lines
    }
    
    /// Recursively visualize a node and its ancestors
    fn visualize_node(&self, id: &ShoveId, lines: &mut Vec<String>, prefix: &str, visited: &HashSet<ShoveId>) -> HashSet<ShoveId> {
        let mut new_visited = visited.clone();
        new_visited.insert(id.clone());
        
        if let Some(node) = self.nodes.get(id) {
            // Process parents
            for (i, parent_id) in node.parents.iter().enumerate() {
                if visited.contains(parent_id) {
                    continue;
                }
                
                if let Some(parent) = self.nodes.get(parent_id) {
                    let is_last = i == node.parents.len() - 1;
                    let new_prefix = if is_last {
                        prefix.replace("├", "└")
                    } else {
                        prefix.to_string()
                    };
                    
                    lines.push(format!("{} 📌 {} {}", 
                        new_prefix,
                        parent_id.as_str()[0..8].bright_yellow(), 
                        parent.message.bright_white()));
                    
                    // Continue with parent's parents
                    let next_prefix = if is_last {
                        prefix.replace("├", " ")
                    } else {
                        prefix.to_string()
                    };
                    
                    let parent_visited = self.visualize_node(parent_id, lines, &next_prefix, &new_visited);
                    for id in parent_visited {
                        new_visited.insert(id);
                    }
                }
            }
        }
        
        new_visited
    }
}

/// Generate a visual graph of the timeline history
pub fn generate_graph(path: &Path) -> Result<Vec<String>> {
    let repo = Repository::open(path)?;
    let graph = Graph::new(&repo)?;
    Ok(graph.visualize())
} 