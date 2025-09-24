use crate::{element::ElementType, misc_types::Spring};
use std::sync::MutexGuard;

// maybe i should stop empubbinating everything
pub struct Chain {
    pub spring: Spring,
    pub links: Vec<ChainLink>,
}

pub fn calculate_force(chain: &Chain, link_index: usize) -> f32 {
    let link = &chain.links[link_index];
    let mut force: f32 = 0.0;

    // left spring
    // we could DRY this out, right?
    if link_index != 0 {
        if let Some(left_neighbour) = chain.links.get(link_index - 1) {
            let natural_distance = calculate_spring_distance(chain.spring.spacing, left_neighbour);
            let displacement = link.mass.position - left_neighbour.mass.position;
            force += -chain.spring.constant * (displacement - (natural_distance as f32));
        }
    } else {
        // nudge the starting element to zero, so we can anchor to something
        // will be removed when i figure out how to do this more cleanly
        let natural_distance = 2;
        let displacement = link.mass.position; // goal position is ZERO!
        force += -chain.spring.constant * (displacement - (natural_distance as f32))
    }

    // right spring
    if let Some(right_neighbour) = chain.links.get(link_index + 1) {
        let natural_distance = calculate_spring_distance(chain.spring.spacing, link); // ERM!!!
        let displacement = right_neighbour.mass.position - link.mass.position;
        force += chain.spring.constant * (displacement - (natural_distance as f32));
    }

    force -= chain.spring.dampening * link.mass.velocity;

    force
}

pub fn step_links(chain: &mut MutexGuard<Chain>, dt: f32) {
    // I DONT KNOW WHAT VERLET INTEGRATION IS
    let n = chain.links.len();

    // calculate extra forces and such
    let mut extra_forces = vec![0.0; n];
    for (i, force) in extra_forces.iter_mut().enumerate() {
        *force = calculate_force(chain, i) / chain.links.get(i).unwrap().mass.mass;
    }

    for (i, link) in chain.links.iter_mut().enumerate() {
        link.mass.position += link.mass.velocity * dt + 0.5 * extra_forces[i] * dt * dt;

        link.mass.velocity += extra_forces[i];
    }
}

pub struct ChainMass {
    pub mass: f32,
    pub position: f32,
    pub velocity: f32,
    pub width: u16,
}

pub struct ChainLink {
    pub mass: ChainMass,
    pub element: ElementType,
}

// ignoring the right element until we have to anchor shit to the right
// also its kind of annoying that we have to define the generic despite not touching it at all
fn calculate_spring_distance(spacing: u16, l: &ChainLink) -> u16 {
    l.mass.width + spacing // good enough
}
