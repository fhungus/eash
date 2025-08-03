use crate::elements::element_trait::Element;
use std::sync::MutexGuard;

pub type Chain = Vec<ChainLink>;

// [TODO] configuration
const SPRING_CONSTANT: f32 = 1.0;
const SPRING_DAMPING: f32 = 0.05;

pub fn calculate_force(chain: &Chain, link_index: usize) -> f32 {
    let link = &chain[link_index];
    let mut force: f32 = 0.0;

    // left spring
    // we could DRY this out, right?
    if link_index != 0 {
        if let Some(left_neighbour) = chain.get(link_index - 1) {
            let natural_distance = calculate_spring_distance(left_neighbour, link);
            let displacement = link.mass.position - left_neighbour.mass.position;
            force += -SPRING_CONSTANT * (displacement - (natural_distance as f32));
        }
    } else {
        // nudge the starting element to zero, so we can anchor to something
        // will be removed when i figure out how to do this more cleanly
        let natural_distance = 2;
        let displacement = link.mass.position; // goal position is ZERO!
        force += -SPRING_CONSTANT * (displacement - (natural_distance as f32))
    }

    // right spring
    if let Some(right_neighbour) = chain.get(link_index + 1) {
        let natural_distance = calculate_spring_distance(link, right_neighbour);
        let displacement = right_neighbour.mass.position - link.mass.position;
        force += SPRING_CONSTANT * (displacement - (natural_distance as f32));
    }

    force -= SPRING_DAMPING * link.mass.velocity;

    return force;
}

pub fn step_links(chain: &mut MutexGuard<Chain>, dt: f32) {
    // I DONT KNOW WHAT VERLET INTEGRATION IS
    let n = chain.len();

    // calculate extra forces and such
    let mut extra_forces = vec![0.0; n];
    for i in 0..n {
        extra_forces[i] = calculate_force(chain, i) / chain.get(i).unwrap().mass.mass;
    }

    for i in 0..n {
        let link = chain.get_mut(i).unwrap();

        link.mass.position =
            link.mass.position + (link.mass.velocity * dt + 0.5 * extra_forces[i] * dt * dt);

        link.mass.velocity += extra_forces[i];
    }
}

pub struct ChainMass {
    pub mass: f32,
    pub position: f32,
    pub velocity: f32,
}

pub struct ChainLink {
    pub mass: ChainMass,
    pub element: Box<dyn Element + Send + Sync>,
}

// ignoring the r until we have to anchor shit to the right
fn calculate_spring_distance(l: &ChainLink, _r: &ChainLink /* uhm */) -> u32 {
    // spring distance should be size of both of its neighbours / 2 + the spacing between them
    const SPACING: u32 = 2;
    return l.element.get_width() + SPACING;
}
