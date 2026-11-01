//! The plan as a Graphviz graph (picture 58): who waits on whom, for Graphviz, documentation and agents.

use pchart_core::{Plan, Schedule};

fn quoted(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"))
}

/// A DOT graph of the plan: a cluster for each lane with its team and city, a box for each piece of work
/// with its length, a diamond for each milestone, and an arrow for each item it waits on; the chain
/// that decides the date in bold.
pub fn dot(plan: &Plan, schedule: &Schedule) -> String {
    let mut out = format!("digraph {} {{\n  rankdir=TB;\n  graph [fontname=\"Noto Sans\", fontsize=12];\n  node [shape=box, style=rounded, fontname=\"Noto Sans\", fontsize=11];\n  edge [color=\"#7d827b\"];\n", quoted(&plan.name));
    for lane in &plan.lanes {
        let label = [Some(lane.name.as_str()), lane.city.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ");
        out.push_str(&format!("  subgraph {} {{\n    label={};\n    color=\"#c9cdc4\";\n", quoted(&format!("cluster_{}", lane.id)), quoted(&label)));
        for (index, item) in plan.items.iter().enumerate().filter(|(_, item)| item.lane == lane.id) {
            let bold = if schedule.chain.contains(&index) { ", penwidth=2" } else { "" };
            if item.milestone {
                out.push_str(&format!("    {} [shape=diamond, style=filled, color=\"#e33d84\", label=\"\", width=0.25, height=0.25, xlabel={}];\n", quoted(&item.id), quoted(&item.name)));
            } else {
                out.push_str(&format!("    {} [label={}{bold}];\n", quoted(&item.id), quoted(&format!("{}\n{} wd", item.name, item.duration))));
            }
        }
        out.push_str("  }\n");
    }
    for (index, item) in plan.items.iter().enumerate() {
        for pred in item.after.iter().filter_map(|id| plan.item(id)) {
            let bold = if schedule.chain.contains(&index) && schedule.chain.contains(&pred) { " [color=\"#1f2421\", penwidth=2]" } else { "" };
            out.push_str(&format!("  {} -> {}{bold};\n", quoted(&plan.items[pred].id), quoted(&item.id)));
        }
    }
    out.push_str("}\n");
    out
}
