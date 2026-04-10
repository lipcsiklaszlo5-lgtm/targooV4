use crate::models::Language;

pub struct I18nDict {
    pub report_title: String,
    pub scope1_label: String,
    pub scope2_label: String,
    pub scope3_label: String,
    pub quarantine_warning: String,
}

pub fn get_dictionary(lang: &Language) -> I18nDict {
    match lang {
        Language::German => I18nDict {
            report_title: "TREIBHAUSGAS-INVENTAR 2024".to_string(),
            scope1_label: "SCOPE 1 — DIREKTE EMISSIONEN".to_string(),
            scope2_label: "SCOPE 2 — INDIREKTE EMISSIONEN".to_string(),
            scope3_label: "SCOPE 3 — WERTSCHÖPFUNGSKETTE".to_string(),
            quarantine_warning: "WARNUNG: Daten in Quarantäne gefunden".to_string(),
        },
        Language::English => I18nDict {
            report_title: "GREENHOUSE GAS INVENTORY 2024".to_string(),
            scope1_label: "SCOPE 1 — DIRECT EMISSIONS".to_string(),
            scope2_label: "SCOPE 2 — INDIRECT EMISSIONS".to_string(),
            scope3_label: "SCOPE 3 — VALUE CHAIN".to_string(),
            quarantine_warning: "WARNING: Data found in quarantine".to_string(),
        },
        Language::Hungarian => I18nDict {
            report_title: "ÜVEGHÁZHATÁSÚ GÁZ LELTÁR 2024".to_string(),
            scope1_label: "SCOPE 1 — KÖZVETLEN KIBOCSÁTÁSOK".to_string(),
            scope2_label: "SCOPE 2 — KÖZVETETT KIBOCSÁTÁSOK".to_string(),
            scope3_label: "SCOPE 3 — ÉRTÉKLÁNC KIBOCSÁTÁSOK".to_string(),
            quarantine_warning: "FIGYELEM: Karanténba zárt adatok találhatók".to_string(),
        },
    }
}
