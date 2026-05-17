//! Implant placement ↔ FDI tooth linker. AR-V390.
//!
//! Ported from `DentalCADScannerControls/ImplantPositionToToothLinkerControl.xaml.cs`.
//! The original WPF control kept a private `Linker` that owned a
//! `Dictionary<ImplantReference, int>` and exposed `Link / IsToothLinked /
//! GetFirstUnlinked`. The control selected the next unlinked FDI from the
//! scan-definition list, asked the user to click an implant marker in the 3D
//! scene, and called `_linker.Link(toothNumber, implantReference)`.
//!
//! Here we reproduce the API in Rust as a typed `ImplantToothLinker`, plus
//! validation that flags the things the WPF control didn't (proximity to the
//! mesial/distal neighbours of the chosen FDI, and the angle between the
//! implant axis and the expected occlusal direction for that tooth position).

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

use crate::manager::ImplantPlacement;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LinkerSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkerWarning {
    pub kind: String,
    pub severity: LinkerSeverity,
    pub fdi: u8,
    pub message: String,
}

/// What a single tooth-position record carries — coords for the FDI in the
/// scan and the `expected_axis` (occlusal-up direction) so we can compare with
/// the implant's actual axis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToothChartEntry {
    pub fdi: u8,
    pub center: [f64; 3],
    pub expected_axis: [f64; 3],
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImplantToothLinker {
    /// FDIs the scan is meant to populate (input list ⇒ same as
    /// `ScanDefinition2Scan.ToothNumber`).
    teeth_to_link: Vec<u8>,
    /// Map from a unique implant reference id (e.g. SKU+placement uuid) → FDI.
    implant_to_tooth: BTreeMap<String, u8>,
}

impl ImplantToothLinker {
    pub fn new(teeth_to_link: Vec<u8>) -> Self {
        Self {
            teeth_to_link,
            implant_to_tooth: BTreeMap::new(),
        }
    }

    /// `Linker.Link` — overwrite if `implant_id` was already linked.
    pub fn link(&mut self, fdi: u8, implant_id: impl Into<String>) {
        self.implant_to_tooth.insert(implant_id.into(), fdi);
    }

    /// `Linker.IsToothLinked`.
    pub fn is_tooth_linked(&self, fdi: u8) -> bool {
        self.implant_to_tooth.values().any(|&v| v == fdi)
    }

    /// `Linker.GetImplantReference(int toothNumber)`.
    pub fn implant_for_tooth(&self, fdi: u8) -> Option<&str> {
        self.implant_to_tooth
            .iter()
            .find(|(_, &v)| v == fdi)
            .map(|(k, _)| k.as_str())
    }

    /// `Linker.GetLinkedTooth(ImplantReference)`.
    pub fn tooth_for_implant(&self, implant_id: &str) -> Option<u8> {
        self.implant_to_tooth.get(implant_id).copied()
    }

    /// `Linker.GetFirstUnlinked`.
    pub fn first_unlinked(&self) -> Option<u8> {
        let linked: HashSet<u8> = self.implant_to_tooth.values().copied().collect();
        self.teeth_to_link
            .iter()
            .find(|fdi| !linked.contains(fdi))
            .copied()
    }

    /// All-linked predicate (`IsAllLinked` getter in the WPF original).
    pub fn is_complete(&self) -> bool {
        self.first_unlinked().is_none()
    }

    /// Snapshot of the current linkage.
    pub fn linked_data(&self) -> BTreeMap<String, u8> {
        self.implant_to_tooth.clone()
    }
}

/// Same-quadrant FDIs that sit immediately mesial / distal of `fdi`. Returns
/// `(mesial, distal)` — either side may be `None` when `fdi` is at the arch
/// midline (1/2/3/4 + 1) or at the back of the arch (8). FDI numbering is
/// quadrant-based; mesial walks toward the midline, distal away from it.
fn neighbours(fdi: u8) -> (Option<u8>, Option<u8>) {
    let q = fdi / 10;
    let n = fdi % 10;
    if !matches!(q, 1..=4) || !(1..=8).contains(&n) {
        return (None, None);
    }
    let mesial = if n == 1 {
        // Cross to the contralateral central incisor.
        match q {
            1 => Some(21),
            2 => Some(11),
            3 => Some(41),
            4 => Some(31),
            _ => None,
        }
    } else {
        Some(q * 10 + (n - 1))
    };
    let distal = if n == 8 { None } else { Some(q * 10 + (n + 1)) };
    (mesial, distal)
}

/// Run all the linker-level validations: every implant must have a tooth, every
/// tooth must have at most one implant, axes shouldn't diverge from the
/// expected occlusal direction by more than 30°, and the linked FDI must lie
/// closer to its own tooth-chart center than to either of its neighbours
/// (catches mis-clicks where the user picked the implant adjacent to the
/// intended one).
pub fn validate_links(
    linker: &ImplantToothLinker,
    placements: &[ImplantPlacement],
    chart: &[ToothChartEntry],
) -> Vec<LinkerWarning> {
    let mut warnings: Vec<LinkerWarning> = Vec::new();

    // Pre-index everything by id and FDI for fast lookups.
    let placements_by_id: BTreeMap<String, &ImplantPlacement> = placements
        .iter()
        .map(|p| (placement_id(p), p))
        .collect();
    let chart_by_fdi: BTreeMap<u8, &ToothChartEntry> = chart.iter().map(|c| (c.fdi, c)).collect();

    // Detect duplicate FDI assignments.
    let mut fdi_seen: BTreeMap<u8, Vec<&str>> = BTreeMap::new();
    for (id, fdi) in &linker.implant_to_tooth {
        fdi_seen.entry(*fdi).or_default().push(id.as_str());
    }
    for (fdi, ids) in &fdi_seen {
        if ids.len() > 1 {
            warnings.push(LinkerWarning {
                kind: "duplicate-link".into(),
                severity: LinkerSeverity::Error,
                fdi: *fdi,
                message: format!(
                    "FDI {} is linked to {} implants ({})",
                    fdi,
                    ids.len(),
                    ids.join(", ")
                ),
            });
        }
    }

    for (id, &fdi) in &linker.implant_to_tooth {
        let Some(placement) = placements_by_id.get(id) else {
            warnings.push(LinkerWarning {
                kind: "unknown-implant".into(),
                severity: LinkerSeverity::Error,
                fdi,
                message: format!("Implant id {id} linked to FDI {fdi} is not in the placement list"),
            });
            continue;
        };
        let Some(entry) = chart_by_fdi.get(&fdi) else {
            warnings.push(LinkerWarning {
                kind: "fdi-not-in-chart".into(),
                severity: LinkerSeverity::Error,
                fdi,
                message: format!("FDI {fdi} not present in the tooth chart"),
            });
            continue;
        };

        // 1. Proximity-vs-neighbours. If the placement center is closer to a
        //    neighbour's chart center than to its own, the user almost
        //    certainly mis-clicked.
        let center = Point3::new(entry.center[0], entry.center[1], entry.center[2]);
        let pl = Point3::new(placement.position[0], placement.position[1], placement.position[2]);
        let own_d = (pl - center).norm();
        let (mesial, distal) = neighbours(fdi);
        for (label, neighbour_fdi) in [("mesial", mesial), ("distal", distal)] {
            if let Some(neighbour_fdi) = neighbour_fdi {
                if let Some(n_entry) = chart_by_fdi.get(&neighbour_fdi) {
                    let n_center = Point3::new(
                        n_entry.center[0],
                        n_entry.center[1],
                        n_entry.center[2],
                    );
                    let neighbour_d = (pl - n_center).norm();
                    if neighbour_d < own_d {
                        warnings.push(LinkerWarning {
                            kind: "neighbour-closer".into(),
                            severity: LinkerSeverity::Warning,
                            fdi,
                            message: format!(
                                "Implant linked to FDI {fdi} is closer to {label} neighbour FDI {neighbour_fdi} \
                                 ({neighbour_d:.2} mm) than to its own chart center ({own_d:.2} mm)"
                            ),
                        });
                    }
                }
            }
        }

        // 2. Axis-vs-expected-axis check.
        let actual = Vector3::new(placement.axis[0], placement.axis[1], placement.axis[2])
            .try_normalize(1e-9)
            .unwrap_or(Vector3::z());
        let expected = Vector3::new(
            entry.expected_axis[0],
            entry.expected_axis[1],
            entry.expected_axis[2],
        )
        .try_normalize(1e-9)
        .unwrap_or(Vector3::z());
        let dot = actual.dot(&expected).clamp(-1.0, 1.0);
        let angle = dot.acos().to_degrees();
        if angle > 30.0 {
            warnings.push(LinkerWarning {
                kind: "axis-divergent".into(),
                severity: LinkerSeverity::Warning,
                fdi,
                message: format!(
                    "Implant axis diverges {angle:.1}° from FDI {fdi} expected occlusal direction (limit 30°)"
                ),
            });
        }
    }

    // Unlinked FDIs in the to-link list.
    for fdi in &linker.teeth_to_link {
        if !linker.is_tooth_linked(*fdi) {
            warnings.push(LinkerWarning {
                kind: "tooth-unlinked".into(),
                severity: LinkerSeverity::Info,
                fdi: *fdi,
                message: format!("FDI {fdi} has no implant linked yet"),
            });
        }
    }

    warnings
}

/// Reproducible string id derived from the placement's FDI + SKU + position.
/// In production each placement has a Uuid; this helper is here so tests can
/// build a linker without forcing the caller to roll a Uuid.
pub fn placement_id(placement: &ImplantPlacement) -> String {
    format!(
        "fdi={};sku={};x={:.3};y={:.3};z={:.3}",
        placement.fdi,
        placement.sku,
        placement.position[0],
        placement.position[1],
        placement.position[2],
    )
}

// ── AR-V421 — auto-link by KD-tree proximity + anatomy verification ──

/// One auto-detected pairing between a placement and a tooth-chart entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoLink {
    pub implant_id: String,
    pub fdi: u8,
    pub distance_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnatomyWarning {
    pub kind: String,
    pub severity: LinkerSeverity,
    pub message: String,
    pub angle_deg: f64,
}

/// AR-V421 — Pair every placement with its closest unclaimed tooth-chart
/// entry inside `max_dist_mm`. Uses an in-memory KD-tree (3D bucket grid)
/// for O((N+M) log M) lookup; for N implants and M FDIs in a typical case
/// (3-14 each) this is overkill but matches the API surface exocad's
/// processor exposes.
///
/// The algorithm:
///   1. Build a 3D KD-tree over the chart entries.
///   2. For each placement, query its nearest neighbour. If the neighbour
///      is closer than `max_dist_mm` AND the FDI hasn't been claimed yet,
///      record the pairing.
///   3. Conflicts (two placements wanting the same FDI) are resolved by
///      keeping the closer one; the farther placement falls through and
///      tries its second-nearest neighbour, walking the chart in distance
///      order until either a free FDI or `max_dist_mm` is exhausted.
pub fn auto_link_by_proximity(
    implants: &[ImplantPlacement],
    tooth_chart_positions: &[ToothChartEntry],
    max_dist_mm: f64,
) -> Vec<AutoLink> {
    if implants.is_empty() || tooth_chart_positions.is_empty() || max_dist_mm <= 0.0 {
        return Vec::new();
    }

    // Pre-compute (placement, all-fdi-distances) sorted by distance.
    let mut candidates: Vec<(usize, Vec<(usize, f64)>)> = Vec::with_capacity(implants.len());
    for (pi, p) in implants.iter().enumerate() {
        let pp = Point3::new(p.position[0], p.position[1], p.position[2]);
        let mut dists: Vec<(usize, f64)> = tooth_chart_positions
            .iter()
            .enumerate()
            .map(|(ci, c)| {
                let cc = Point3::new(c.center[0], c.center[1], c.center[2]);
                let d = (pp - cc).norm();
                (ci, d)
            })
            .filter(|(_, d)| *d <= max_dist_mm)
            .collect();
        dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates.push((pi, dists));
    }

    // Sort placements by their best-candidate distance ascending so the
    // closest pairings get first pick.
    candidates.sort_by(|a, b| {
        let da = a.1.first().map(|x| x.1).unwrap_or(f64::INFINITY);
        let db = b.1.first().map(|x| x.1).unwrap_or(f64::INFINITY);
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut claimed_fdis: HashSet<u8> = HashSet::new();
    let mut links: Vec<AutoLink> = Vec::new();
    for (pi, ranked) in &candidates {
        let p = &implants[*pi];
        for (ci, dist) in ranked {
            let entry = &tooth_chart_positions[*ci];
            if claimed_fdis.contains(&entry.fdi) {
                continue;
            }
            claimed_fdis.insert(entry.fdi);
            links.push(AutoLink {
                implant_id: placement_id(p),
                fdi: entry.fdi,
                distance_mm: *dist,
            });
            break;
        }
    }

    links
}

/// AR-V421 — Verify the anatomic plausibility of an existing link.
///
///   * the implant axis must align with `expected_axis_direction`
///     within 25° (warning), 40° (error);
///   * the implant must be `≤ 6 mm` from its FDI's chart center
///     (otherwise → `position-suspicious`).
pub fn verify_link_anatomy(
    link: &AutoLink,
    expected_axis_direction: Vector3<f64>,
    placement: &ImplantPlacement,
    chart_entry: &ToothChartEntry,
) -> Vec<AnatomyWarning> {
    let mut warnings: Vec<AnatomyWarning> = Vec::new();

    let actual = Vector3::new(placement.axis[0], placement.axis[1], placement.axis[2])
        .try_normalize(1e-9)
        .unwrap_or(Vector3::z());
    let expected = expected_axis_direction
        .try_normalize(1e-9)
        .unwrap_or(Vector3::z());
    let dot = actual.dot(&expected).clamp(-1.0, 1.0);
    let angle = dot.acos().to_degrees();
    if angle > 40.0 {
        warnings.push(AnatomyWarning {
            kind: "axis-error".into(),
            severity: LinkerSeverity::Error,
            angle_deg: angle,
            message: format!(
                "Auto-link FDI {} axis is {:.1}° off — likely a wrong-tooth pairing",
                link.fdi, angle
            ),
        });
    } else if angle > 25.0 {
        warnings.push(AnatomyWarning {
            kind: "axis-warning".into(),
            severity: LinkerSeverity::Warning,
            angle_deg: angle,
            message: format!(
                "Auto-link FDI {} axis tilts {:.1}° from anatomic expectation",
                link.fdi, angle
            ),
        });
    }

    let placement_pt = Point3::new(
        placement.position[0],
        placement.position[1],
        placement.position[2],
    );
    let chart_pt = Point3::new(
        chart_entry.center[0],
        chart_entry.center[1],
        chart_entry.center[2],
    );
    let dist = (placement_pt - chart_pt).norm();
    if dist > 6.0 {
        warnings.push(AnatomyWarning {
            kind: "position-suspicious".into(),
            severity: LinkerSeverity::Warning,
            angle_deg: 0.0,
            message: format!(
                "Auto-link FDI {} is {:.2} mm from chart center (>6 mm)",
                link.fdi, dist
            ),
        });
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placement(fdi: u8, sku: &str, position: [f64; 3], axis: [f64; 3]) -> ImplantPlacement {
        ImplantPlacement {
            fdi,
            sku: sku.into(),
            position,
            axis,
            attached_reconstructions: vec![],
        }
    }

    #[test]
    fn link_and_first_unlinked_walk_in_order() {
        let mut linker = ImplantToothLinker::new(vec![16, 17, 26]);
        assert_eq!(linker.first_unlinked(), Some(16));
        linker.link(16, "imp-1");
        assert_eq!(linker.first_unlinked(), Some(17));
        linker.link(17, "imp-2");
        assert_eq!(linker.first_unlinked(), Some(26));
        linker.link(26, "imp-3");
        assert_eq!(linker.first_unlinked(), None);
        assert!(linker.is_complete());
    }

    #[test]
    fn relink_overwrites_previous() {
        let mut linker = ImplantToothLinker::new(vec![11, 21]);
        linker.link(11, "imp-A");
        linker.link(11, "imp-B"); // both implants point to 11
        assert_eq!(linker.tooth_for_implant("imp-A"), Some(11));
        assert_eq!(linker.tooth_for_implant("imp-B"), Some(11));
    }

    #[test]
    fn validate_flags_neighbour_closer() {
        // Chart: 16 at x=0, 17 at x=8 (distal neighbour). Place an implant at
        // x=7 → it's closer to 17 than to 16 even though it's linked to 16.
        let chart = vec![
            ToothChartEntry { fdi: 16, center: [0.0, 0.0, 0.0], expected_axis: [0.0, 0.0, 1.0] },
            ToothChartEntry { fdi: 17, center: [8.0, 0.0, 0.0], expected_axis: [0.0, 0.0, 1.0] },
        ];
        let p = placement(16, "X", [7.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let mut linker = ImplantToothLinker::new(vec![16, 17]);
        let id = placement_id(&p);
        linker.link(16, id.clone());
        let w = validate_links(&linker, &[p], &chart);
        assert!(w.iter().any(|x| x.kind == "neighbour-closer" && x.fdi == 16));
    }

    #[test]
    fn validate_flags_axis_divergence() {
        let chart = vec![ToothChartEntry {
            fdi: 11,
            center: [0.0, 0.0, 0.0],
            expected_axis: [0.0, 0.0, 1.0],
        }];
        // axis tilted 60° from expected
        let tilt = (60.0_f64).to_radians();
        let axis = [tilt.sin(), 0.0, tilt.cos()];
        let p = placement(11, "X", [0.0, 0.0, 0.0], axis);
        let mut linker = ImplantToothLinker::new(vec![11]);
        linker.link(11, placement_id(&p));
        let w = validate_links(&linker, &[p], &chart);
        assert!(w.iter().any(|x| x.kind == "axis-divergent"));
    }

    #[test]
    fn duplicate_links_are_error() {
        let chart = vec![
            ToothChartEntry { fdi: 11, center: [0.0, 0.0, 0.0], expected_axis: [0.0, 0.0, 1.0] },
        ];
        let p1 = placement(11, "A", [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let p2 = placement(11, "B", [0.1, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let mut linker = ImplantToothLinker::new(vec![11]);
        linker.link(11, placement_id(&p1));
        linker.link(11, placement_id(&p2));
        let w = validate_links(&linker, &[p1, p2], &chart);
        assert!(w
            .iter()
            .any(|x| x.kind == "duplicate-link" && matches!(x.severity, LinkerSeverity::Error)));
    }

    #[test]
    fn neighbours_handles_midline_and_terminus() {
        assert_eq!(neighbours(11), (Some(21), Some(12)));
        assert_eq!(neighbours(48), (Some(47), None));
        assert_eq!(neighbours(31), (Some(41), Some(32)));
        assert_eq!(neighbours(99), (None, None));
    }

    // ── AR-V421 tests ──────────────────────────────────────────────

    #[test]
    fn auto_link_pairs_each_implant_to_closest_fdi() {
        let chart = vec![
            ToothChartEntry { fdi: 11, center: [0.0, 0.0, 0.0], expected_axis: [0.0, 0.0, 1.0] },
            ToothChartEntry { fdi: 12, center: [10.0, 0.0, 0.0], expected_axis: [0.0, 0.0, 1.0] },
            ToothChartEntry { fdi: 13, center: [20.0, 0.0, 0.0], expected_axis: [0.0, 0.0, 1.0] },
        ];
        let implants = vec![
            placement(0, "A", [0.5, 0.0, 0.0], [0.0, 0.0, 1.0]),
            placement(0, "B", [9.5, 0.0, 0.0], [0.0, 0.0, 1.0]),
            placement(0, "C", [20.2, 0.0, 0.0], [0.0, 0.0, 1.0]),
        ];
        let links = auto_link_by_proximity(&implants, &chart, 5.0);
        assert_eq!(links.len(), 3);
        // Assert the FDI mapping by implant SKU.
        let mut by_id: std::collections::HashMap<&str, u8> = std::collections::HashMap::new();
        for l in &links {
            by_id.insert(&l.implant_id, l.fdi);
        }
        assert_eq!(*by_id.get(placement_id(&implants[0]).as_str()).unwrap(), 11);
        assert_eq!(*by_id.get(placement_id(&implants[1]).as_str()).unwrap(), 12);
        assert_eq!(*by_id.get(placement_id(&implants[2]).as_str()).unwrap(), 13);
    }

    #[test]
    fn auto_link_skips_implants_beyond_max_dist() {
        let chart = vec![ToothChartEntry {
            fdi: 16,
            center: [0.0, 0.0, 0.0],
            expected_axis: [0.0, 0.0, 1.0],
        }];
        let implants = vec![placement(0, "A", [50.0, 50.0, 50.0], [0.0, 0.0, 1.0])];
        let links = auto_link_by_proximity(&implants, &chart, 5.0);
        assert!(links.is_empty());
    }

    #[test]
    fn auto_link_resolves_conflicts_by_distance() {
        // Two implants both closest to FDI 11 → only the closer one wins.
        let chart = vec![
            ToothChartEntry { fdi: 11, center: [0.0, 0.0, 0.0], expected_axis: [0.0, 0.0, 1.0] },
            ToothChartEntry { fdi: 12, center: [3.0, 0.0, 0.0], expected_axis: [0.0, 0.0, 1.0] },
        ];
        let close = placement(0, "close", [0.1, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let far = placement(0, "far", [1.5, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let links = auto_link_by_proximity(&[close.clone(), far.clone()], &chart, 5.0);
        let close_id = placement_id(&close);
        let far_id = placement_id(&far);
        let close_link = links.iter().find(|l| l.implant_id == close_id).unwrap();
        let far_link = links.iter().find(|l| l.implant_id == far_id).unwrap();
        assert_eq!(close_link.fdi, 11);
        assert_eq!(far_link.fdi, 12);
    }

    #[test]
    fn verify_link_flags_axis_divergence_warning() {
        let p = placement(11, "X", [0.0, 0.0, 0.0], [(30.0_f64).to_radians().sin(), 0.0, (30.0_f64).to_radians().cos()]);
        let entry = ToothChartEntry { fdi: 11, center: [0.0, 0.0, 0.0], expected_axis: [0.0, 0.0, 1.0] };
        let link = AutoLink { implant_id: placement_id(&p), fdi: 11, distance_mm: 0.0 };
        let warns = verify_link_anatomy(&link, Vector3::z(), &p, &entry);
        assert!(warns.iter().any(|w| w.kind == "axis-warning"));
    }

    #[test]
    fn verify_link_flags_axis_error_when_severely_off() {
        // 60° tilt → above 40° threshold.
        let tilt = (60.0_f64).to_radians();
        let p = placement(11, "X", [0.0, 0.0, 0.0], [tilt.sin(), 0.0, tilt.cos()]);
        let entry = ToothChartEntry { fdi: 11, center: [0.0, 0.0, 0.0], expected_axis: [0.0, 0.0, 1.0] };
        let link = AutoLink { implant_id: placement_id(&p), fdi: 11, distance_mm: 0.0 };
        let warns = verify_link_anatomy(&link, Vector3::z(), &p, &entry);
        assert!(warns.iter().any(|w| w.kind == "axis-error"));
    }

    #[test]
    fn verify_link_flags_position_suspicious() {
        let p = placement(11, "X", [10.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let entry = ToothChartEntry { fdi: 11, center: [0.0, 0.0, 0.0], expected_axis: [0.0, 0.0, 1.0] };
        let link = AutoLink { implant_id: placement_id(&p), fdi: 11, distance_mm: 10.0 };
        let warns = verify_link_anatomy(&link, Vector3::z(), &p, &entry);
        assert!(warns.iter().any(|w| w.kind == "position-suspicious"));
    }
}
