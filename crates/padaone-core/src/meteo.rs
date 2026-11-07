//! Météo à deux axes : chaque participant pose un point dans un carré
//! humeur (horizontal) × énergie (vertical). L'écran montre le nuage de points.
//!
//! Les points sont triés avant d'être envoyés : l'ordre d'arrivée ne permet pas
//! de retrouver qui a posé quel point.

use serde::{Deserialize, Serialize};

use crate::activite::{
    moyenne, question_valide, Activite, ErreurConfiguration, ErreurReponse, Rang,
};

/// Coordonnées de 0 à 100.
pub const POINT_MAX: u8 = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meteo {
    pub question: String,
}

impl Meteo {
    pub fn validee(&self) -> Result<Self, ErreurConfiguration> {
        Ok(Meteo { question: question_valide(&self.question)? })
    }
}

/// Un point : `x` = humeur (0 maussade, 100 radieuse), `y` = énergie (0 à plat, 100 en forme).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ResultatMeteo {
    pub points: Vec<Point>,
    pub total: usize,
    /// Point moyen (arrondi au dixième).
    pub moyenne: Option<[f64; 2]>,
}

impl Activite for Meteo {
    type Reponse = Point;
    type Resultat = ResultatMeteo;

    fn valider(&self, p: Point) -> Result<Point, ErreurReponse> {
        if p.x <= POINT_MAX && p.y <= POINT_MAX {
            Ok(p)
        } else {
            Err(ErreurReponse::HorsEchelle)
        }
    }

    fn resultat<'a>(&self, reponses: impl Iterator<Item = (Rang, &'a Point)>) -> ResultatMeteo {
        let mut points: Vec<Point> = reponses.map(|(_, p)| *p).collect();
        points.sort_unstable();
        let xs: Vec<f64> = points.iter().map(|p| f64::from(p.x)).collect();
        let ys: Vec<f64> = points.iter().map(|p| f64::from(p.y)).collect();
        let moyenne = moyenne(&xs).zip(moyenne(&ys)).map(|(x, y)| [x, y]);
        ResultatMeteo { total: points.len(), points, moyenne }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activite::Collecte;

    #[test]
    fn points_dans_le_carre() {
        let m = Meteo { question: "Comment ça va ?".into() };
        assert!(m.valider(Point { x: 100, y: 0 }).is_ok());
        assert_eq!(m.valider(Point { x: 101, y: 0 }), Err(ErreurReponse::HorsEchelle));
    }

    #[test]
    fn points_tries_et_point_moyen() {
        let mut c = Collecte::new(Meteo { question: "Q".into() });
        c.repondre(1, Point { x: 80, y: 20 }).unwrap();
        c.repondre(2, Point { x: 10, y: 60 }).unwrap();
        let r = c.resultat();
        assert_eq!(r.points, vec![Point { x: 10, y: 60 }, Point { x: 80, y: 20 }]);
        assert_eq!(r.moyenne, Some([45.0, 40.0]));
    }
}
