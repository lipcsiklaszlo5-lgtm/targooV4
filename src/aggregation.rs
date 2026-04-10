use std::collections::HashMap;
use crate::ledger::LedgerRow;
use crate::scope3_aggregation::{Scope3Aggregation, CategorySummary};
use crate::scope3_types::CalcPath;

pub struct ScopeAggregation {
    pub scope1_total_tco2e: f64,
    pub scope2_total_tco2e: f64,
    pub scope3_data: Scope3Aggregation,
    pub total_inventory_tco2e: f64,
}

pub fn aggregate_ledger(ledger: &[LedgerRow]) -> ScopeAggregation {
    let mut s1_total = 0.0;
    let mut s2_total = 0.0;
    let mut s3_total = 0.0;
    let mut s3_breakdown: HashMap<u8, CategorySummary> = HashMap::new();
    let mut dq_sum = 0.0;
    let mut dq_count = 0;

    for row in ledger {
        match row.ghg_scope.as_str() {
            "Scope1" => s1_total += row.tco2e,
            "Scope2" => s2_total += row.tco2e,
            "Scope3" => {
                s3_total += row.tco2e;
                if let Some(ext) = &row.scope3_ext {
                    let summary = s3_breakdown.entry(ext.category_id).or_insert(CategorySummary {
                        category_id: ext.category_id,
                        total_tco2e: 0.0,
                        row_count: 0,
                        calc_path_mix: (0, 0),
                    });

                    summary.total_tco2e += row.tco2e;
                    summary.row_count += 1;
                    match ext.calc_path {
                        CalcPath::ActivityBased => summary.calc_path_mix.0 += 1,
                        CalcPath::SpendBased => summary.calc_path_mix.1 += 1,
                    }

                    dq_sum += ext.ghg_protocol_dq_score as f32;
                    dq_count += 1;
                }
            }
            _ => {}
        }
    }

    let avg_dq = if dq_count > 0 { dq_sum / dq_count as f32 } else { 0.0 };

    let s3_data = Scope3Aggregation {
        grand_total_tco2e: s3_total,
        category_breakdown: s3_breakdown,
        data_quality_avg_score: avg_dq,
    };

    ScopeAggregation {
        scope1_total_tco2e: s1_total,
        scope2_total_tco2e: s2_total,
        scope3_data: s3_data,
        total_inventory_tco2e: s1_total + s2_total + s3_total,
    }
}
