use serde::{Deserialize, Serialize};

/// Información de ejércitos que puede desplegar cada facción/marca.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Army {
    pub name: String,
    pub composition: String,
    pub specialty: String,
    pub motto: String,
}

/// Marca/facción registrada en el códice, alineada con las entradas 27–32 del YAML.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactionMark {
    pub id_marca: u8,
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub territories: Vec<String>,
    pub armies: Vec<Army>,
}

impl FactionMark {
    fn new(id_marca: u8, code: &str, name: &str, tags: &[&str], territories: &[&str], armies: Vec<Army>) -> Self {
        Self {
            id_marca,
            code: code.to_string(),
            name: name.to_string(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            territories: territories.iter().map(|t| t.to_string()).collect(),
            armies,
        }
    }
}

/// Devuelve el catálogo base de Marcas/Facciones con sus ejércitos disponibles.
pub fn core_faction_marks() -> Vec<FactionMark> {
    vec![
        FactionMark::new(
            27,
            "FACCION.ELEMENTAL",
            "Tormentas Elementales",
            &["ELEMENTAL", "CLIMA", "DESASTRE"],
            &["HUMANO", "EXTERIOR"],
            vec![
                Army {
                    name: "Guardia del Trueno".to_string(),
                    composition: "Infantería pesada con martillos aislados y estandartes para canalizar rayos".to_string(),
                    specialty: "Romper líneas bajo tormenta y disipar efectos climáticos".to_string(),
                    motto: "La tempestad obedece a los que no tiemblan".to_string(),
                },
                Army {
                    name: "Cohorte de Relámpagos".to_string(),
                    composition: "Zapadores rúnicos y artillería de bobinas".to_string(),
                    specialty: "Ataques rápidos, sabotaje de nodos y dispersión de nubes tóxicas".to_string(),
                    motto: "Golpeamos antes de que la luz se apague".to_string(),
                },
            ],
        ),
        FactionMark::new(
            28,
            "FACCION.BANDIDOS",
            "Campamento de Bandidos",
            &["HUMANO", "CRIMEN", "REBELDIA"],
            &["HUMANO", "EXTERIOR"],
            vec![
                Army {
                    name: "Hermanas del Camino Polvoriento".to_string(),
                    composition: "Escaramuzadoras montadas, arqueras y tramperos".to_string(),
                    specialty: "Emboscadas, cortar suministros y capturar prisioneros por rescate".to_string(),
                    motto: "Nadie compra libertad sin deuda".to_string(),
                },
                Army {
                    name: "Mercenarios del Estandarte Roto".to_string(),
                    composition: "Veteranos deshonrados con armas mixtas".to_string(),
                    specialty: "Luchar sucio, rehacer líneas tras cada golpe, sembrar pánico".to_string(),
                    motto: "La paga manda, el honor espera".to_string(),
                },
            ],
        ),
        FactionMark::new(
            29,
            "FACCION.MONSTRUO",
            "Guarida de Monstruo",
            &["BESTIA_UNICA", "HORROR", "CAZA_MAYOR"],
            &["EXTERIOR"],
            vec![
                Army {
                    name: "Cacería de Lanzas Negras".to_string(),
                    composition: "Cazadores, rastreadores y arponeros con armaduras ligeras".to_string(),
                    specialty: "Cazar presas titánicas, fijar criaturas y rematar con venenos".to_string(),
                    motto: "Clavamos la noche para que amanezca".to_string(),
                },
                Army {
                    name: "Custodios del Foso".to_string(),
                    composition: "Guardianes del bastión que contienen o distraen la bestia".to_string(),
                    specialty: "Mantener líneas defensivas, retrasar golpes devastadores".to_string(),
                    motto: "Si cae el muro, caemos con él".to_string(),
                },
            ],
        ),
        FactionMark::new(
            30,
            "FACCION.BOSQUE",
            "Bosque Encantado / Oscuro",
            &["FEERICO", "NATURALEZA_VIVA", "LOCURA"],
            &["EXTERIOR"],
            vec![
                Army {
                    name: "Guarda de Corteza".to_string(),
                    composition: "Hostigadores con hojas curvas, druidas y espíritus menores".to_string(),
                    specialty: "Controlar terreno difícil, manipular raíces y niebla".to_string(),
                    motto: "El bosque recuerda quién lo hiere".to_string(),
                },
                Army {
                    name: "Ronda de Linternas Marchitas".to_string(),
                    composition: "Exploradores feéricos y vigías con luz rúnica".to_string(),
                    specialty: "Desorientar, aislar patrullas y guiar a la presa".to_string(),
                    motto: "Sigue la luz y perderás el camino".to_string(),
                },
            ],
        ),
        FactionMark::new(
            31,
            "FACCION.PLAGA",
            "Plaga Verde",
            &["ENFERMEDAD", "CONTAGIO", "CRISIS"],
            &["HUMANO", "EXTERIOR"],
            vec![
                Army {
                    name: "Monjes Pálidos".to_string(),
                    composition: "Sanadores, cirujanos de campo y portadores de antídotos".to_string(),
                    specialty: "Contención biológica, quemas controladas y cuarentenas rápidas".to_string(),
                    motto: "Purificar antes que lamentar".to_string(),
                },
                Army {
                    name: "Brigada de Cal viva".to_string(),
                    composition: "Zapadores y arqueros con munición incendiaria".to_string(),
                    specialty: "Sellar focos de infección, derribar madrigueras".to_string(),
                    motto: "La cal blanquea la culpa".to_string(),
                },
            ],
        ),
        FactionMark::new(
            32,
            "FACCION.TINTA",
            "Escuela del Silencio / Tinta",
            &["METATRAMA", "LOCURA", "CORRUPCION"],
            &["HUMANO", "EXTERIOR"],
            vec![
                Army {
                    name: "Alumnas del Silencio".to_string(),
                    composition: "Asesinas rituales, escribas y portadores de sellos".to_string(),
                    specialty: "Operaciones encubiertas, manipulación de memoria y sellos de Tinta".to_string(),
                    motto: "Lo que se escribe en Tinta no se olvida".to_string(),
                },
                Army {
                    name: "Custodios del Bastión Negro".to_string(),
                    composition: "Guardia élite con armaduras inscritas".to_string(),
                    specialty: "Proteger nodos rúnicos, contrarrestar plagas y herejías".to_string(),
                    motto: "Sellar, guardar, callar".to_string(),
                },
            ],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_all_core_marks_with_armies() {
        let marks = core_faction_marks();
        assert_eq!(marks.len(), 6);
        assert!(marks.iter().all(|m| !m.armies.is_empty()));
    }
}
