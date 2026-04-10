use crate::models::Jurisdiction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictResolution {
    Clear(Jurisdiction),
    Ambiguous {
        winner: Jurisdiction,
        loser: Option<Jurisdiction>,
        note: String,
    },
}

#[derive(Debug, Clone)]
pub struct JurisdictionSignal {
    pub weight: u8,
    pub jurisdiction: Jurisdiction,
}

pub fn handle_jurisdiction_conflict(mut signals: Vec<JurisdictionSignal>) -> (Jurisdiction, ConflictResolution) {
    if signals.is_empty() {
        return (Jurisdiction::Global, ConflictResolution::Clear(Jurisdiction::Global));
    }

    // Sort by weight descending
    signals.sort_by(|a, b| b.weight.cmp(&a.weight));

    let winner = signals[0].clone();
    let second = signals.get(1).cloned();

    match second {
        Some(s) if (winner.weight as i16 - s.weight as i16).abs() < 20 => {
            (
                winner.jurisdiction.clone(),
                ConflictResolution::Ambiguous {
                    winner: winner.jurisdiction,
                    loser: Some(s.jurisdiction),
                    note: "JURISDICTION_CONFLICT_FLAGGED".to_string(),
                },
            )
        }
        _ => (
            winner.jurisdiction.clone(),
            ConflictResolution::Clear(winner.jurisdiction),
        ),
    }
}
