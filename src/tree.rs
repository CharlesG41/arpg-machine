//! The character's skill tree, generated from a seed like God Tower's ability
//! trees: an Origin with six short exits, a cluster of spells behind each exit,
//! an outer ring of clusters, and bridges between neighboring clusters. Every spell appears exactly once;
//! only the layout changes from one seed to the next.
//!
//! Spending works like God Tower: clicking a node buys the shortest route to
//! it from what you already own, one Skill Point per new node. Respec is free.
use crate::content::SpellId;
use crate::math::V2;
use crate::rng::Rng;
use std::collections::VecDeque;
use std::f32::consts::TAU;

/// Layout is stretched horizontally, like God Tower's, to fit wide cards.
const STRETCH: f32 = 1.7;
/// Minimum spacing between card centers (either axis is enough).
/// Cards are as wide as their spell's size: size 2 is twice size 1.
pub const CARD_UNIT: f32 = 170.0;
pub const CARD_GAP: f32 = 14.0;
pub const CARD_H: f32 = 100.0;
pub const ORIGIN_SIZE: f32 = 90.0;
/// Space kept between two cards side by side, or one above the other.
const GAP_X: f32 = 40.0;
const MIN_DY: f32 = CARD_H + 30.0;

pub fn card_width(size: u8) -> f32 {
    size as f32 * CARD_UNIT - CARD_GAP
}

/// Whether two cards (centers and widths) leave enough room between them.
fn apart(p: V2, pw: f32, q: V2, qw: f32) -> bool {
    (p.x - q.x).abs() >= (pw + qw) / 2.0 + GAP_X || (p.y - q.y).abs() >= MIN_DY
}

#[derive(Clone, Debug)]
pub struct TreeNode {
    /// None for the Origin.
    pub spell: Option<SpellId>,
    pub pos: V2,
    pub neighbors: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct SkillTree {
    pub seed: u64,
    pub nodes: Vec<TreeNode>,
    pub acquired: Vec<bool>,
    pub points: u32,
}

pub const ORIGIN: usize = 0;

impl SkillTree {
    pub fn generate(seed: u64, points: u32) -> Self {
        let mut rng = Rng::new(seed ^ 0x7EE5_7EE5);
        let mut pool: Vec<SpellId> = SpellId::all().collect();
        shuffle(&mut rng, &mut pool);
        // The six exits come first in the pool. A new character must be able to
        // kill things, so at least four of them deal damage.
        let damaging = |s: &SpellId| s.hits_enemies();
        while pool[..6].iter().filter(|s| damaging(s)).count() < 4 {
            let weak = pool[..6].iter().position(|s| !damaging(s)).unwrap();
            let strong = 6 + pool[6..].iter().position(damaging).unwrap();
            pool.swap(weak, strong);
        }

        let mut positions = vec![V2::ZERO];
        // Card width of each position: position k holds pool[k - 1].
        let width_of = |k: usize| if k == 0 { ORIGIN_SIZE } else { card_width(pool[k - 1].size()) };
        let rotation = rng.range(0.0, TAU);
        // Six short exits keep the first choices close to the Origin.
        for i in 0..6 {
            let a = rotation + i as f32 * TAU / 6.0;
            positions.push(V2::new(a.cos() * 330.0 * STRETCH, a.sin() * 330.0));
        }
        // The rest in clusters of about eight: six behind the exits, then an outer ring.
        let rest = pool.len().saturating_sub(6);
        let cluster_count = ((rest as f32 / 8.0).round() as usize).max(6);
        let sizes: Vec<usize> = (0..cluster_count).map(|c| rest / cluster_count + usize::from(c < rest % cluster_count)).collect();
        let mut groups: Vec<Vec<usize>> = Vec::new();
        let mut centers: Vec<V2> = Vec::new();
        for (c, &count) in sizes.iter().enumerate() {
            let center = if c < 6 {
                let a = rotation + c as f32 * TAU / 6.0 + rng.range(-0.12, 0.12);
                let radius = rng.range(780.0, 860.0) + count as f32 * 10.0;
                V2::new(a.cos() * radius, a.sin() * radius)
            } else {
                let mut attempt = 0;
                loop {
                    let a = rng.range(0.0, TAU);
                    let radius = rng.range(1500.0, 2000.0) + (attempt / 100) as f32 * 120.0;
                    let candidate = V2::new(a.cos() * radius, a.sin() * radius);
                    if centers.iter().all(|c| c.dist(candidate) > 900.0) {
                        break candidate;
                    }
                    attempt += 1;
                }
            };
            centers.push(center);
            let phase = rng.range(0.0, TAU);
            let spread = 200.0 + count as f32 * 12.0;
            let mut group = Vec::new();
            for j in 0..count {
                let mut attempt = 0;
                loop {
                    let angle = phase + j as f32 * TAU / count as f32 + rng.range(-0.1, 0.1);
                    let r = if j == 0 && attempt == 0 && rng.chance(0.5) {
                        0.0
                    } else {
                        rng.range(spread, spread + 60.0) + attempt as f32 * 20.0
                    };
                    let p = V2::new((center.x + angle.cos() * r) * STRETCH, center.y + angle.sin() * r);
                    let w = width_of(positions.len());
                    if positions.iter().enumerate().all(|(k, &q)| apart(p, w, q, width_of(k))) {
                        group.push(positions.len());
                        positions.push(p);
                        break;
                    }
                    attempt += 1;
                }
            }
            groups.push(group);
        }

        let mut nodes: Vec<TreeNode> = positions
            .iter()
            .enumerate()
            .map(|(id, &pos)| TreeNode { spell: if id == 0 { None } else { Some(pool[id - 1]) }, pos, neighbors: Vec::new() })
            .collect();
        let distance = |a: usize, b: usize| {
            let (p, q) = (positions[a], positions[b]);
            ((p.x - q.x) / STRETCH).hypot(p.y - q.y)
        };
        let closest = |xs: &[usize], ys: &[usize]| {
            xs.iter()
                .flat_map(|&x| ys.iter().map(move |&y| (x, y)))
                .min_by(|&(a, b), &(c, d)| distance(a, b).total_cmp(&distance(c, d)))
                .unwrap()
        };
        let mut links = Vec::new();
        for exit in 1..=6 {
            links.push((ORIGIN, exit));
        }
        for (index, group) in groups.iter().enumerate() {
            // Inner clusters hang off their exit; outer ones off the nearest earlier cluster.
            let entry = if index < 6 {
                let pair = closest(&[index + 1], group);
                links.push(pair);
                pair.1
            } else {
                let parent = (0..index)
                    .min_by(|&a, &b| centers[a].dist(centers[index]).total_cmp(&centers[b].dist(centers[index])))
                    .unwrap();
                let pair = closest(group, &groups[parent]);
                links.push(pair);
                pair.0
            };
            let mut reached = vec![entry];
            let mut remaining: Vec<usize> = group.iter().copied().filter(|&n| n != entry).collect();
            let mut local = Vec::new();
            while !remaining.is_empty() {
                let pair = closest(&reached, &remaining);
                local.push(pair);
                reached.push(pair.1);
                remaining.retain(|&n| n != pair.1);
            }
            let mut extra = Vec::new();
            for (i, &a) in group.iter().enumerate() {
                for &b in &group[i + 1..] {
                    if distance(a, b) < 430.0 && !local.contains(&(a, b)) && !local.contains(&(b, a)) {
                        extra.push((a, b));
                    }
                }
            }
            shuffle(&mut rng, &mut extra);
            let loops = rng.range_usize(1, 2);
            local.extend(extra.into_iter().take(loops));
            links.extend(local);
            // Short bridges between neighboring clusters, so routes can cross branches.
            if index < 6 && rng.chance(0.67) {
                links.push(closest(group, &groups[(index + 1) % 6]));
            }
            for previous in 6..index {
                if centers[previous].dist(centers[index]) < 1500.0 && rng.chance(0.5) {
                    links.push(closest(group, &groups[previous]));
                }
            }
        }
        for (a, b) in links {
            if a != b && !nodes[a].neighbors.contains(&b) {
                nodes[a].neighbors.push(b);
                nodes[b].neighbors.push(a);
            }
        }
        let mut acquired = vec![false; nodes.len()];
        acquired[ORIGIN] = true;
        SkillTree { seed, nodes, acquired, points }
    }

    /// Discovery: you see what you own and the nodes right next to it.
    pub fn visible(&self, id: usize) -> bool {
        self.acquired[id] || self.nodes[id].neighbors.iter().any(|&n| self.acquired[n])
    }

    /// Rebuilds a saved tree: same seed, owned nodes restored. Nodes that no
    /// longer connect to the Origin (or exceed the points) are dropped.
    pub fn restore(seed: u64, points: u32, owned: &[usize]) -> Self {
        let mut tree = SkillTree::generate(seed, points);
        for &id in owned {
            if id < tree.nodes.len() {
                tree.acquired[id] = true;
            }
        }
        let mut connected = vec![false; tree.nodes.len()];
        connected[ORIGIN] = true;
        let mut stack = vec![ORIGIN];
        while let Some(n) = stack.pop() {
            for &m in &tree.nodes[n].neighbors {
                if tree.acquired[m] && !connected[m] {
                    connected[m] = true;
                    stack.push(m);
                }
            }
        }
        tree.acquired = connected;
        tree.set_points(points);
        tree
    }

    pub fn spent(&self) -> u32 {
        self.acquired.iter().filter(|&&a| a).count() as u32 - 1
    }

    pub fn remaining(&self) -> u32 {
        self.points.saturating_sub(self.spent())
    }

    /// Nodes that would be bought to reach `dest`, in order (empty if owned).
    /// None when unreachable.
    pub fn route_to(&self, dest: usize) -> Option<Vec<usize>> {
        if dest >= self.nodes.len() {
            return None;
        }
        if self.acquired[dest] {
            return Some(Vec::new());
        }
        // Every owned node is a free starting point (owned nodes stay connected).
        let mut previous: Vec<Option<usize>> = vec![None; self.nodes.len()];
        let mut seen = self.acquired.clone();
        let mut queue: VecDeque<usize> = (0..self.nodes.len()).filter(|&i| self.acquired[i]).collect();
        while let Some(node) = queue.pop_front() {
            for &next in &self.nodes[node].neighbors {
                if seen[next] {
                    continue;
                }
                seen[next] = true;
                previous[next] = Some(node);
                if next == dest {
                    let mut route = vec![next];
                    let mut cursor = next;
                    while let Some(parent) = previous[cursor] {
                        if self.acquired[parent] {
                            break;
                        }
                        route.push(parent);
                        cursor = parent;
                    }
                    route.reverse();
                    return Some(route);
                }
                queue.push_back(next);
            }
        }
        None
    }

    /// Buys the route to `dest` if the points allow it.
    pub fn acquire(&mut self, dest: usize) -> bool {
        match self.route_to(dest) {
            Some(route) if !route.is_empty() && route.len() as u32 <= self.remaining() => {
                for id in route {
                    self.acquired[id] = true;
                }
                true
            }
            _ => false,
        }
    }

    /// Refunds a node and everything that only connected through it. Free.
    pub fn refund(&mut self, id: usize) -> u32 {
        if id == ORIGIN || !self.acquired.get(id).copied().unwrap_or(false) {
            return 0;
        }
        self.acquired[id] = false;
        let mut connected = vec![false; self.nodes.len()];
        connected[ORIGIN] = true;
        let mut stack = vec![ORIGIN];
        while let Some(node) = stack.pop() {
            for &next in &self.nodes[node].neighbors {
                if self.acquired[next] && !connected[next] {
                    connected[next] = true;
                    stack.push(next);
                }
            }
        }
        let mut refunded = 1;
        for i in 0..self.nodes.len() {
            if self.acquired[i] && !connected[i] {
                self.acquired[i] = false;
                refunded += 1;
            }
        }
        refunded
    }

    pub fn reset(&mut self) {
        self.acquired = vec![false; self.nodes.len()];
        self.acquired[ORIGIN] = true;
    }

    /// Lowers the point budget, refunding the most recent branches if needed.
    pub fn set_points(&mut self, points: u32) {
        self.points = points;
        while self.spent() > self.points {
            // Refund the owned node farthest from the Origin (a leaf).
            let far = (0..self.nodes.len()).filter(|&i| self.acquired[i] && i != ORIGIN).max_by(|&a, &b| {
                self.nodes[a].pos.len_sq().total_cmp(&self.nodes[b].pos.len_sq())
            });
            match far {
                Some(id) => {
                    self.refund(id);
                }
                None => break,
            }
        }
    }

    pub fn acquired_spells(&self) -> Vec<SpellId> {
        self.nodes.iter().zip(&self.acquired).filter(|(_, &a)| a).filter_map(|(n, _)| n.spell).collect()
    }

    pub fn node_of(&self, spell: SpellId) -> Option<usize> {
        self.nodes.iter().position(|n| n.spell == Some(spell))
    }
}

fn shuffle<T>(rng: &mut Rng, items: &mut [T]) {
    for i in (1..items.len()).rev() {
        let j = rng.range_usize(0, i);
        items.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connected_from_origin(tree: &SkillTree) -> usize {
        let mut seen = vec![false; tree.nodes.len()];
        seen[ORIGIN] = true;
        let mut stack = vec![ORIGIN];
        while let Some(n) = stack.pop() {
            for &m in &tree.nodes[n].neighbors {
                if !seen[m] {
                    seen[m] = true;
                    stack.push(m);
                }
            }
        }
        seen.iter().filter(|&&s| s).count()
    }

    #[test]
    fn every_spell_once_and_all_connected() {
        for seed in 0..50 {
            let tree = SkillTree::generate(seed, 14);
            assert_eq!(tree.nodes.len(), SpellId::count() + 1);
            let spells: std::collections::HashSet<_> = tree.nodes.iter().filter_map(|n| n.spell).collect();
            assert_eq!(spells.len(), SpellId::count());
            assert_eq!(connected_from_origin(&tree), tree.nodes.len(), "seed {seed}");
        }
    }

    #[test]
    fn cards_do_not_overlap() {
        for seed in 0..50 {
            let tree = SkillTree::generate(seed, 14);
            for (i, a) in tree.nodes.iter().enumerate() {
                for b in &tree.nodes[i + 1..] {
                    let w = |n: &TreeNode| n.spell.map_or(ORIGIN_SIZE, |s| card_width(s.size()));
                    // Exits sit on a fixed ring; everything placed later keeps its distance.
                    if a.spell.is_some() && b.spell.is_some() {
                        assert!(
                            (a.pos.x - b.pos.x).abs() >= (w(a) + w(b)) / 2.0 || (a.pos.y - b.pos.y).abs() >= CARD_H,
                            "seed {seed}: {} and {} overlap",
                            a.spell.unwrap().name(),
                            b.spell.unwrap().name()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn exits_mostly_deal_damage() {
        for seed in 0..50 {
            let tree = SkillTree::generate(seed, 7);
            let damaging = tree.nodes[ORIGIN].neighbors.iter().filter(|&&n| tree.nodes[n].spell.unwrap().hits_enemies()).count();
            assert!(damaging >= 4, "seed {seed}: {damaging} damaging exits");
        }
    }

    #[test]
    fn discovery_reveals_neighbors_of_owned_nodes() {
        let mut tree = SkillTree::generate(5, 7);
        let exits = tree.nodes[ORIGIN].neighbors.clone();
        let hidden = (1..tree.nodes.len()).filter(|&i| !tree.visible(i)).count();
        assert!(exits.iter().all(|&e| tree.visible(e)));
        assert!(hidden > tree.nodes.len() / 2);
        assert!(tree.acquire(exits[0]));
        let after = (1..tree.nodes.len()).filter(|&i| !tree.visible(i)).count();
        assert!(after < hidden);
    }

    #[test]
    fn restore_keeps_connected_owned_nodes_only() {
        let mut tree = SkillTree::generate(9, 20);
        let far = (1..tree.nodes.len()).find(|&i| tree.route_to(i).unwrap().len() == 3).unwrap();
        tree.acquire(far);
        let owned: Vec<usize> = (1..tree.nodes.len()).filter(|&i| tree.acquired[i]).collect();
        let restored = SkillTree::restore(9, 20, &owned);
        assert_eq!(restored.acquired, tree.acquired);
        // A node with no owned path to the Origin is dropped.
        let lonely = (1..tree.nodes.len()).find(|&i| tree.route_to(i).is_some_and(|r| r.len() >= 2)).unwrap();
        let broken = SkillTree::restore(9, 20, &[lonely]);
        assert!(!broken.acquired[lonely]);
    }

    #[test]
    fn seeds_change_the_layout() {
        let a = SkillTree::generate(1, 14);
        let b = SkillTree::generate(2, 14);
        assert!(a.nodes.iter().zip(&b.nodes).any(|(x, y)| x.spell != y.spell));
    }

    #[test]
    fn buying_follows_the_shortest_route_and_refund_cascades() {
        let mut tree = SkillTree::generate(3, 40);
        // A cluster node is at least two steps from the Origin.
        let far = (1..tree.nodes.len()).find(|&i| tree.route_to(i).unwrap().len() >= 2).unwrap();
        let route = tree.route_to(far).unwrap();
        assert!(tree.acquire(far));
        assert_eq!(tree.spent(), route.len() as u32);
        // Refunding the first step strands the rest of the route.
        let refunded = tree.refund(route[0]);
        assert!(refunded >= 2);
        assert!(!tree.acquired[far]);
    }

    #[test]
    fn points_limit_purchases() {
        let mut tree = SkillTree::generate(4, 1);
        let far = (1..tree.nodes.len()).find(|&i| tree.route_to(i).unwrap().len() >= 2).unwrap();
        assert!(!tree.acquire(far));
        assert!(tree.acquire(tree.nodes[ORIGIN].neighbors[0]));
        assert_eq!(tree.remaining(), 0);
        tree.set_points(0);
        assert_eq!(tree.spent(), 0);
    }
}
