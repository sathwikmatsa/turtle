use super::battlestate::Cell;
use turtle::Color;
pub struct Config {}

impl Config {
    pub const EMPTY_COLOR: &'static str = "#55dde0";
    pub const UNATTACKED_COLOR: &'static str = "#55dde0";
    pub const CARRIER_COLOR: &'static str = "#f6ae2d";
    pub const BATTLESHIP_COLOR: &'static str = "#f48923";
    pub const CRUISER_COLOR: &'static str = "#947757";
    pub const SUBMARINE_COLOR: &'static str = "#2f4858";
    pub const DESTROYER_COLOR: &'static str = "#238cf4";
    pub const MISSED_COLOR: &'static str = "#33658a";
    pub const BOMBED_COLOR: &'static str = "#f26419";
    pub const DESTROYED_COLOR: &'static str = "#000000";
    pub const TARGET_COLOR: &'static str = "#f26419";

    pub const CELL_SIZE: f64 = 40.0;
    pub const SPACE_BETWEEN_GRIDS: f64 = 50.0;

    pub const SHIP_GRID_TOP_LEFT: (f64, f64) = (-Self::SPACE_BETWEEN_GRIDS / 2.0 - 10.0 * Self::CELL_SIZE, 5.0 * Self::CELL_SIZE);
    pub const ATTACK_GRID_TOP_LEFT: (f64, f64) = (Self::SPACE_BETWEEN_GRIDS / 2.0, 5.0 * Self::CELL_SIZE);

    pub const MISSED_CIRCLE_DIAMETER: f64 = 0.25 * Self::CELL_SIZE;
    pub const BOMBED_CIRCLE_DIAMETER: f64 = 0.75 * Self::CELL_SIZE;

    pub const CROSSHAIR_SIZE: f64 = 0.4 * Self::CELL_SIZE;
    pub const CROSSHAIR_PEN_SIZE: f64 = 4.0;

    pub fn cell_color(cell: &Cell) -> Color {
        match cell {
            Cell::Carrier => Self::CARRIER_COLOR.into(),
            Cell::Battleship => Self::BATTLESHIP_COLOR.into(),
            Cell::Cruiser => Self::CRUISER_COLOR.into(),
            Cell::Submarine => Self::SUBMARINE_COLOR.into(),
            Cell::Destroyer => Self::DESTROYER_COLOR.into(),
            Cell::Empty => Self::EMPTY_COLOR.into(),
            Cell::Unattacked => Self::UNATTACKED_COLOR.into(),
            Cell::Missed => Self::MISSED_COLOR.into(),
            Cell::Bombed => Self::BOMBED_COLOR.into(),
            Cell::Destroyed => Self::DESTROYED_COLOR.into(),
        }
    }
}
