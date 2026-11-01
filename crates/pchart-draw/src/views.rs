//! The 16:9 pages pChart exports, and who they are for.

use jiff::civil::Date;
use pchart_core::analysis::Check;
use pchart_core::{Plan, Schedule};

use crate::scene::Scene;

/// Who an export is for. The team sees everything; management sees no warnings; the client sees no
/// warnings and not what single items have to spare. Agents get what the team gets, and in Markdown
/// the ids of the items, so they can report progress back through the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Audience {
    #[default]
    Team,
    Management,
    Client,
    Agents,
}

impl Audience {
    pub fn warnings(self) -> bool {
        matches!(self, Audience::Team | Audience::Agents)
    }

    /// Whether the page says what single items have to spare.
    /// Notes under the milestones: the client does not see them.
    pub fn notes(self) -> bool {
        self != Audience::Client
    }

    pub fn spare(self) -> bool {
        self != Audience::Client
    }

    pub fn from_id(id: &str) -> Option<Audience> {
        Some(match id {
            "team" => Audience::Team,
            "management" => Audience::Management,
            "client" => Audience::Client,
            "agents" => Audience::Agents,
            _ => return None,
        })
    }
}

/// The light pages, in the order the Export screen lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    Gantt,
    #[default]
    Milestones,
    Roadmap,
    Phases,
    NowNextLater,
    Structure,
    Network,
}

impl View {
    pub const ALL: [View; 7] = [View::Gantt, View::Milestones, View::Roadmap, View::Phases, View::NowNextLater, View::Structure, View::Network];

    pub fn name(self) -> &'static str {
        match self {
            View::Gantt => "Gantt chart",
            View::Milestones => "Milestone chart",
            View::Roadmap => "Swimlane roadmap",
            View::Phases => "Phase plan",
            View::NowNextLater => "Now / Next / Later",
            View::Structure => "Project structure",
            View::Network => "Network diagram",
        }
    }

    /// The name on the command line (`--view`) and in file names.
    pub fn id(self) -> &'static str {
        match self {
            View::Gantt => "gantt",
            View::Milestones => "milestones",
            View::Roadmap => "roadmap",
            View::Phases => "phases",
            View::NowNextLater => "now-next-later",
            View::Structure => "structure",
            View::Network => "network",
        }
    }

    pub fn from_id(id: &str) -> Option<View> {
        View::ALL.into_iter().find(|view| view.id() == id)
    }

    /// The page for `audience`; `checks` are the plan's, the page leaves out what the audience does not see.
    pub fn page(self, plan: &Plan, schedule: &Schedule, today: Date, checks: &[Check], audience: Audience) -> Scene {
        match self {
            View::Gantt => crate::gantt(plan, schedule, today, checks, audience),
            View::Milestones => crate::milestones(plan, schedule, today, audience),
            View::Roadmap => crate::swimlanes(plan, schedule, today, checks, audience),
            View::Phases => crate::phase_plan(plan, schedule, today),
            View::NowNextLater => crate::now_next_later(plan, schedule, today),
            View::Structure => crate::structure(plan, schedule, today),
            View::Network => crate::network(plan, schedule, today),
        }
    }
}
