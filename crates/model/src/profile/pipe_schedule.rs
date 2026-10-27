/// ASME B36.10M carbon steel pipe wall thickness tables.
///
/// Maps DN (nominal diameter in mm) and schedule to wall thickness (mm).

pub struct PipeSchedule {
    pub dn: f64,
    pub schedule: &'static str,
    pub od: f64,
    pub wall: f64,
}

/// Look up wall thickness for a given DN and schedule.
/// Returns `None` if the combination is not found.
pub fn wall_thickness(dn: f64, schedule: &str) -> Option<f64> {
    let dn_key = dn.round() as i32;
    let s = schedule.to_uppercase();
    SCHEDULE_TABLE.iter()
        .find(|row| row.dn_key == dn_key && row.schedule.eq_ignore_ascii_case(&s))
        .map(|row| row.wall)
}

/// Outer diameter in mm for a given DN.
pub fn outer_diameter(dn: f64) -> f64 {
    let dn_key = dn.round() as i32;
    OD_TABLE.iter()
        .find(|row| row.dn_key == dn_key)
        .map(|row| row.od)
        .unwrap_or_else(|| dn_key as f64)
}

/// Look up wall thickness for stainless steel pipe via ASME B36.19M.
/// Uses the same outer diameter as B36.10M but a distinct wall-thickness
/// schedule (5S/10S/40S/80S). Returns `None` if the combination is not found.
pub fn wall_thickness_ss(dn: f64, schedule: &str) -> Option<f64> {
    let dn_key = dn.round() as i32;
    let s = schedule.to_uppercase();
    SS_SCHEDULE_TABLE.iter()
        .find(|row| row.dn_key == dn_key && row.schedule.eq_ignore_ascii_case(&s))
        .map(|row| row.wall)
}

struct ScheduleRow {
    dn_key: i32,
    schedule: &'static str,
    wall: f64,
}

struct OdRow {
    dn_key: i32,
    od: f64,
}

const SCHEDULE_TABLE: &[ScheduleRow] = &[
    // DN 15 (1/2")
    ScheduleRow { dn_key: 15, schedule: "5S", wall: 1.65 },
    ScheduleRow { dn_key: 15, schedule: "10S", wall: 2.31 },
    ScheduleRow { dn_key: 15, schedule: "40",  wall: 2.77 },
    ScheduleRow { dn_key: 15, schedule: "80",  wall: 3.73 },
    ScheduleRow { dn_key: 15, schedule: "160", wall: 4.75 },
    // DN 20 (3/4")
    ScheduleRow { dn_key: 20, schedule: "5S", wall: 1.65 },
    ScheduleRow { dn_key: 20, schedule: "10S", wall: 2.31 },
    ScheduleRow { dn_key: 20, schedule: "40",  wall: 2.87 },
    ScheduleRow { dn_key: 20, schedule: "80",  wall: 3.91 },
    ScheduleRow { dn_key: 20, schedule: "160", wall: 5.56 },
    // DN 25 (1")
    ScheduleRow { dn_key: 25, schedule: "5S", wall: 1.65 },
    ScheduleRow { dn_key: 25, schedule: "10S", wall: 2.77 },
    ScheduleRow { dn_key: 25, schedule: "40",  wall: 3.38 },
    ScheduleRow { dn_key: 25, schedule: "80",  wall: 4.55 },
    ScheduleRow { dn_key: 25, schedule: "160", wall: 6.35 },
    // DN 32 (1.25")
    ScheduleRow { dn_key: 32, schedule: "5S", wall: 1.65 },
    ScheduleRow { dn_key: 32, schedule: "10S", wall: 2.77 },
    ScheduleRow { dn_key: 32, schedule: "40",  wall: 3.56 },
    ScheduleRow { dn_key: 32, schedule: "80",  wall: 4.85 },
    ScheduleRow { dn_key: 32, schedule: "160", wall: 6.35 },
    // DN 40 (1.5")
    ScheduleRow { dn_key: 40, schedule: "5S", wall: 1.65 },
    ScheduleRow { dn_key: 40, schedule: "10S", wall: 2.77 },
    ScheduleRow { dn_key: 40, schedule: "40",  wall: 3.68 },
    ScheduleRow { dn_key: 40, schedule: "80",  wall: 5.08 },
    ScheduleRow { dn_key: 40, schedule: "160", wall: 7.14 },
    // DN 50 (2")
    ScheduleRow { dn_key: 50, schedule: "5S", wall: 1.65 },
    ScheduleRow { dn_key: 50, schedule: "10S", wall: 2.77 },
    ScheduleRow { dn_key: 50, schedule: "40",  wall: 3.91 },
    ScheduleRow { dn_key: 50, schedule: "80",  wall: 5.54 },
    ScheduleRow { dn_key: 50, schedule: "160", wall: 8.74 },
    // DN 65 (2.5")
    ScheduleRow { dn_key: 65, schedule: "5S", wall: 2.11 },
    ScheduleRow { dn_key: 65, schedule: "10S", wall: 3.05 },
    ScheduleRow { dn_key: 65, schedule: "40",  wall: 5.16 },
    ScheduleRow { dn_key: 65, schedule: "80",  wall: 7.01 },
    ScheduleRow { dn_key: 65, schedule: "160", wall: 9.53 },
    // DN 80 (3")
    ScheduleRow { dn_key: 80, schedule: "5S", wall: 2.11 },
    ScheduleRow { dn_key: 80, schedule: "10S", wall: 3.05 },
    ScheduleRow { dn_key: 80, schedule: "40",  wall: 5.49 },
    ScheduleRow { dn_key: 80, schedule: "80",  wall: 7.62 },
    ScheduleRow { dn_key: 80, schedule: "160", wall: 11.13 },
    // DN 100 (4")
    ScheduleRow { dn_key: 100, schedule: "5S", wall: 2.11 },
    ScheduleRow { dn_key: 100, schedule: "10S", wall: 3.05 },
    ScheduleRow { dn_key: 100, schedule: "40",  wall: 6.02 },
    ScheduleRow { dn_key: 100, schedule: "80",  wall: 8.56 },
    ScheduleRow { dn_key: 100, schedule: "160", wall: 13.49 },
    // DN 150 (6")
    ScheduleRow { dn_key: 150, schedule: "5S", wall: 2.77 },
    ScheduleRow { dn_key: 150, schedule: "10S", wall: 3.40 },
    ScheduleRow { dn_key: 150, schedule: "40",  wall: 7.11 },
    ScheduleRow { dn_key: 150, schedule: "80",  wall: 10.97 },
    ScheduleRow { dn_key: 150, schedule: "160", wall: 18.26 },
    // DN 200 (8")
    ScheduleRow { dn_key: 200, schedule: "5S", wall: 2.77 },
    ScheduleRow { dn_key: 200, schedule: "10S", wall: 3.76 },
    ScheduleRow { dn_key: 200, schedule: "40",  wall: 8.18 },
    ScheduleRow { dn_key: 200, schedule: "80",  wall: 12.70 },
    ScheduleRow { dn_key: 200, schedule: "160", wall: 20.62 },
    // DN 250 (10")
    ScheduleRow { dn_key: 250, schedule: "5S", wall: 3.40 },
    ScheduleRow { dn_key: 250, schedule: "10S", wall: 4.19 },
    ScheduleRow { dn_key: 250, schedule: "40",  wall: 9.27 },
    ScheduleRow { dn_key: 250, schedule: "80",  wall: 15.09 },
    ScheduleRow { dn_key: 250, schedule: "160", wall: 25.40 },
    // DN 300 (12")
    ScheduleRow { dn_key: 300, schedule: "5S", wall: 3.96 },
    ScheduleRow { dn_key: 300, schedule: "10S", wall: 4.57 },
    ScheduleRow { dn_key: 300, schedule: "40",  wall: 10.31 },
    ScheduleRow { dn_key: 300, schedule: "80",  wall: 17.48 },
    ScheduleRow { dn_key: 300, schedule: "160", wall: 28.58 },
    // DN 350 (14")
    ScheduleRow { dn_key: 350, schedule: "5S", wall: 3.96 },
    ScheduleRow { dn_key: 350, schedule: "10S", wall: 4.78 },
    ScheduleRow { dn_key: 350, schedule: "40",  wall: 9.53 },
    ScheduleRow { dn_key: 350, schedule: "80",  wall: 15.09 },
    // DN 400 (16")
    ScheduleRow { dn_key: 400, schedule: "5S", wall: 4.19 },
    ScheduleRow { dn_key: 400, schedule: "10S", wall: 4.78 },
    ScheduleRow { dn_key: 400, schedule: "40",  wall: 9.53 },
    ScheduleRow { dn_key: 400, schedule: "80",  wall: 16.66 },
    // DN 450 (18")
    ScheduleRow { dn_key: 450, schedule: "5S", wall: 4.19 },
    ScheduleRow { dn_key: 450, schedule: "10S", wall: 4.78 },
    ScheduleRow { dn_key: 450, schedule: "40",  wall: 9.53 },
    ScheduleRow { dn_key: 450, schedule: "80",  wall: 19.05 },
    // DN 500 (20")
    ScheduleRow { dn_key: 500, schedule: "5S", wall: 4.78 },
    ScheduleRow { dn_key: 500, schedule: "10S", wall: 5.54 },
    ScheduleRow { dn_key: 500, schedule: "40",  wall: 9.53 },
    ScheduleRow { dn_key: 500, schedule: "80",  wall: 20.62 },
    // DN 600 (24")
    ScheduleRow { dn_key: 600, schedule: "5S", wall: 5.54 },
    ScheduleRow { dn_key: 600, schedule: "10S", wall: 6.35 },
    ScheduleRow { dn_key: 600, schedule: "40",  wall: 9.53 },
    ScheduleRow { dn_key: 600, schedule: "80",  wall: 24.61 },
    // DN 750 (30")
    ScheduleRow { dn_key: 750, schedule: "5S", wall: 6.35 },
    ScheduleRow { dn_key: 750, schedule: "10S", wall: 7.14 },
    ScheduleRow { dn_key: 750, schedule: "40",  wall: 9.53 },
    ScheduleRow { dn_key: 750, schedule: "80",  wall: 24.61 },
    // DN 900 (36")
    ScheduleRow { dn_key: 900, schedule: "5S", wall: 6.35 },
    ScheduleRow { dn_key: 900, schedule: "10S", wall: 7.14 },
    ScheduleRow { dn_key: 900, schedule: "40",  wall: 9.53 },
    ScheduleRow { dn_key: 900, schedule: "80",  wall: 24.61 },
];

const OD_TABLE: &[OdRow] = &[
    OdRow { dn_key: 15,  od: 21.34 },
    OdRow { dn_key: 20,  od: 26.67 },
    OdRow { dn_key: 25,  od: 33.40 },
    OdRow { dn_key: 32,  od: 42.16 },
    OdRow { dn_key: 40,  od: 48.26 },
    OdRow { dn_key: 50,  od: 60.33 },
    OdRow { dn_key: 65,  od: 73.03 },
    OdRow { dn_key: 80,  od: 88.90 },
    OdRow { dn_key: 100, od: 114.30 },
    OdRow { dn_key: 150, od: 168.28 },
    OdRow { dn_key: 200, od: 219.08 },
    OdRow { dn_key: 250, od: 273.05 },
    OdRow { dn_key: 300, od: 323.85 },
    OdRow { dn_key: 350, od: 355.60 },
    OdRow { dn_key: 400, od: 406.40 },
    OdRow { dn_key: 450, od: 457.20 },
    OdRow { dn_key: 500, od: 508.00 },
    OdRow { dn_key: 600, od: 609.60 },
    OdRow { dn_key: 750, od: 762.00 },
    OdRow { dn_key: 900, od: 914.40 },
];

/// ASME B36.19M stainless steel pipe wall thickness table (SS304/SS316).
/// Uses schedules 5S, 10S, 40S, 80S. Outer diameter is identical to B36.10M.
const SS_SCHEDULE_TABLE: &[ScheduleRow] = &[
    ScheduleRow { dn_key: 15,  schedule: "5S",  wall: 1.65 },
    ScheduleRow { dn_key: 15,  schedule: "10S", wall: 2.11 },
    ScheduleRow { dn_key: 15,  schedule: "40S", wall: 2.77 },
    ScheduleRow { dn_key: 15,  schedule: "80S", wall: 3.73 },
    ScheduleRow { dn_key: 20,  schedule: "5S",  wall: 1.65 },
    ScheduleRow { dn_key: 20,  schedule: "10S", wall: 2.11 },
    ScheduleRow { dn_key: 20,  schedule: "40S", wall: 2.87 },
    ScheduleRow { dn_key: 20,  schedule: "80S", wall: 3.91 },
    ScheduleRow { dn_key: 25,  schedule: "5S",  wall: 1.65 },
    ScheduleRow { dn_key: 25,  schedule: "10S", wall: 2.11 },
    ScheduleRow { dn_key: 25,  schedule: "40S", wall: 3.38 },
    ScheduleRow { dn_key: 25,  schedule: "80S", wall: 4.55 },
    ScheduleRow { dn_key: 32,  schedule: "5S",  wall: 1.65 },
    ScheduleRow { dn_key: 32,  schedule: "10S", wall: 2.11 },
    ScheduleRow { dn_key: 32,  schedule: "40S", wall: 3.56 },
    ScheduleRow { dn_key: 32,  schedule: "80S", wall: 4.85 },
    ScheduleRow { dn_key: 40,  schedule: "5S",  wall: 1.65 },
    ScheduleRow { dn_key: 40,  schedule: "10S", wall: 2.11 },
    ScheduleRow { dn_key: 40,  schedule: "40S", wall: 3.68 },
    ScheduleRow { dn_key: 40,  schedule: "80S", wall: 5.08 },
    ScheduleRow { dn_key: 50,  schedule: "5S",  wall: 1.65 },
    ScheduleRow { dn_key: 50,  schedule: "10S", wall: 2.11 },
    ScheduleRow { dn_key: 50,  schedule: "40S", wall: 3.91 },
    ScheduleRow { dn_key: 50,  schedule: "80S", wall: 5.54 },
    ScheduleRow { dn_key: 65,  schedule: "5S",  wall: 1.65 },
    ScheduleRow { dn_key: 65,  schedule: "10S", wall: 2.11 },
    ScheduleRow { dn_key: 65,  schedule: "40S", wall: 5.16 },
    ScheduleRow { dn_key: 65,  schedule: "80S", wall: 7.01 },
    ScheduleRow { dn_key: 80,  schedule: "5S",  wall: 1.65 },
    ScheduleRow { dn_key: 80,  schedule: "10S", wall: 2.11 },
    ScheduleRow { dn_key: 80,  schedule: "40S", wall: 5.49 },
    ScheduleRow { dn_key: 80,  schedule: "80S", wall: 7.62 },
    ScheduleRow { dn_key: 100, schedule: "5S",  wall: 2.11 },
    ScheduleRow { dn_key: 100, schedule: "10S", wall: 2.11 },
    ScheduleRow { dn_key: 100, schedule: "40S", wall: 6.02 },
    ScheduleRow { dn_key: 100, schedule: "80S", wall: 8.56 },
    ScheduleRow { dn_key: 150, schedule: "5S",  wall: 2.11 },
    ScheduleRow { dn_key: 150, schedule: "10S", wall: 2.77 },
    ScheduleRow { dn_key: 150, schedule: "40S", wall: 7.11 },
    ScheduleRow { dn_key: 150, schedule: "80S", wall: 10.97 },
    ScheduleRow { dn_key: 200, schedule: "5S",  wall: 2.11 },
    ScheduleRow { dn_key: 200, schedule: "10S", wall: 2.77 },
    ScheduleRow { dn_key: 200, schedule: "40S", wall: 8.18 },
    ScheduleRow { dn_key: 200, schedule: "80S", wall: 12.70 },
    ScheduleRow { dn_key: 250, schedule: "5S",  wall: 2.11 },
    ScheduleRow { dn_key: 250, schedule: "10S", wall: 3.05 },
    ScheduleRow { dn_key: 250, schedule: "40S", wall: 9.27 },
    ScheduleRow { dn_key: 250, schedule: "80S", wall: 15.09 },
    ScheduleRow { dn_key: 300, schedule: "5S",  wall: 2.11 },
    ScheduleRow { dn_key: 300, schedule: "10S", wall: 3.05 },
    ScheduleRow { dn_key: 300, schedule: "40S", wall: 10.31 },
    ScheduleRow { dn_key: 300, schedule: "80S", wall: 17.48 },
    ScheduleRow { dn_key: 350, schedule: "5S",  wall: 2.11 },
    ScheduleRow { dn_key: 350, schedule: "10S", wall: 3.05 },
    ScheduleRow { dn_key: 350, schedule: "40S", wall: 9.53 },
    ScheduleRow { dn_key: 350, schedule: "80S", wall: 15.09 },
    ScheduleRow { dn_key: 400, schedule: "5S",  wall: 2.11 },
    ScheduleRow { dn_key: 400, schedule: "10S", wall: 3.05 },
    ScheduleRow { dn_key: 400, schedule: "40S", wall: 9.53 },
    ScheduleRow { dn_key: 400, schedule: "80S", wall: 16.66 },
    ScheduleRow { dn_key: 450, schedule: "5S",  wall: 2.11 },
    ScheduleRow { dn_key: 450, schedule: "10S", wall: 3.05 },
    ScheduleRow { dn_key: 450, schedule: "40S", wall: 9.53 },
    ScheduleRow { dn_key: 450, schedule: "80S", wall: 19.05 },
    ScheduleRow { dn_key: 500, schedule: "5S",  wall: 2.11 },
    ScheduleRow { dn_key: 500, schedule: "10S", wall: 3.05 },
    ScheduleRow { dn_key: 500, schedule: "40S", wall: 9.53 },
    ScheduleRow { dn_key: 500, schedule: "80S", wall: 20.62 },
    ScheduleRow { dn_key: 600, schedule: "5S",  wall: 2.77 },
    ScheduleRow { dn_key: 600, schedule: "10S", wall: 3.76 },
    ScheduleRow { dn_key: 600, schedule: "40S", wall: 9.53 },
    ScheduleRow { dn_key: 600, schedule: "80S", wall: 24.61 },
];

/// ASME B16.9 long-radius elbow center-to-face dimension (mm)
pub fn b16_9_elbow_center_to_face(dn: f64) -> f64 {
    let dn_key = dn.round() as i32;
    ELBOW_C2F_TABLE.iter()
        .find(|row| row.dn_key == dn_key)
        .map(|row| row.dim)
        .unwrap_or_else(|| 1.5 * dn)
}

struct ElbowDimRow {
    dn_key: i32,
    dim: f64,
}

const ELBOW_C2F_TABLE: &[ElbowDimRow] = &[
    ElbowDimRow { dn_key: 15,  dim: 38.0 },
    ElbowDimRow { dn_key: 20,  dim: 38.0 },
    ElbowDimRow { dn_key: 25,  dim: 38.0 },
    ElbowDimRow { dn_key: 32,  dim: 48.0 },
    ElbowDimRow { dn_key: 40,  dim: 57.0 },
    ElbowDimRow { dn_key: 50,  dim: 76.0 },
    ElbowDimRow { dn_key: 65,  dim: 95.0 },
    ElbowDimRow { dn_key: 80,  dim: 114.0 },
    ElbowDimRow { dn_key: 100, dim: 152.0 },
    ElbowDimRow { dn_key: 150, dim: 229.0 },
    ElbowDimRow { dn_key: 200, dim: 305.0 },
    ElbowDimRow { dn_key: 250, dim: 381.0 },
    ElbowDimRow { dn_key: 300, dim: 457.0 },
    ElbowDimRow { dn_key: 350, dim: 533.0 },
    ElbowDimRow { dn_key: 400, dim: 610.0 },
    ElbowDimRow { dn_key: 450, dim: 686.0 },
    ElbowDimRow { dn_key: 500, dim: 762.0 },
    ElbowDimRow { dn_key: 600, dim: 914.0 },
];

/// ASME B16.5 Class 150 weld neck flange dimensions (mm)
pub struct B16_5Flange {
    pub dn: f64,
    pub od: f64,
    pub bolt_circle: f64,
    pub bolt_holes: u32,
    pub bolt_diameter: f64,
    pub hub_length: f64,
    pub thickness: f64,
}

pub fn b16_5_flange_cl150(dn: f64) -> Option<B16_5Flange> {
    let dn_key = dn.round() as i32;
    FLANGE_CL150_TABLE.iter()
        .find(|row| row.dn_key == dn_key)
        .map(|row| B16_5Flange {
            dn: row.dn_key as f64,
            od: row.od,
            bolt_circle: row.bolt_circle,
            bolt_holes: row.bolt_holes,
            bolt_diameter: row.bolt_diameter,
            hub_length: row.hub_length,
            thickness: row.thickness,
        })
}

struct FlangeRow {
    dn_key: i32,
    od: f64,
    bolt_circle: f64,
    bolt_holes: u32,
    bolt_diameter: f64,
    hub_length: f64,
    thickness: f64,
}

const FLANGE_CL150_TABLE: &[FlangeRow] = &[
    FlangeRow { dn_key: 15,  od: 89.0,  bolt_circle: 60.3,  bolt_holes: 4,  bolt_diameter: 15.9, hub_length: 52.0,  thickness: 11.2 },
    FlangeRow { dn_key: 20,  od: 98.0,  bolt_circle: 69.9,  bolt_holes: 4,  bolt_diameter: 15.9, hub_length: 57.0,  thickness: 12.7 },
    FlangeRow { dn_key: 25,  od: 108.0, bolt_circle: 79.4,  bolt_holes: 4,  bolt_diameter: 15.9, hub_length: 62.0,  thickness: 14.2 },
    FlangeRow { dn_key: 32,  od: 117.0, bolt_circle: 88.9,  bolt_holes: 4,  bolt_diameter: 15.9, hub_length: 65.0,  thickness: 15.7 },
    FlangeRow { dn_key: 40,  od: 127.0, bolt_circle: 98.4,  bolt_holes: 4,  bolt_diameter: 15.9, hub_length: 68.0,  thickness: 17.5 },
    FlangeRow { dn_key: 50,  od: 152.0, bolt_circle: 120.7, bolt_holes: 4,  bolt_diameter: 19.1, hub_length: 74.0,  thickness: 19.1 },
    FlangeRow { dn_key: 65,  od: 178.0, bolt_circle: 139.7, bolt_holes: 4,  bolt_diameter: 19.1, hub_length: 84.0,  thickness: 22.4 },
    FlangeRow { dn_key: 80,  od: 191.0, bolt_circle: 152.4, bolt_holes: 4,  bolt_diameter: 19.1, hub_length: 91.0,  thickness: 23.9 },
    FlangeRow { dn_key: 100, od: 229.0, bolt_circle: 190.5, bolt_holes: 8,  bolt_diameter: 19.1, hub_length: 102.0, thickness: 25.4 },
    FlangeRow { dn_key: 150, od: 279.0, bolt_circle: 241.3, bolt_holes: 8,  bolt_diameter: 22.4, hub_length: 121.0, thickness: 28.6 },
    FlangeRow { dn_key: 200, od: 343.0, bolt_circle: 298.5, bolt_holes: 8,  bolt_diameter: 22.4, hub_length: 140.0, thickness: 31.8 },
    FlangeRow { dn_key: 250, od: 406.0, bolt_circle: 362.0, bolt_holes: 12, bolt_diameter: 25.4, hub_length: 159.0, thickness: 34.9 },
    FlangeRow { dn_key: 300, od: 483.0, bolt_circle: 431.8, bolt_holes: 12, bolt_diameter: 25.4, hub_length: 178.0, thickness: 38.1 },
    FlangeRow { dn_key: 350, od: 533.0, bolt_circle: 476.3, bolt_holes: 12, bolt_diameter: 28.6, hub_length: 184.0, thickness: 41.3 },
    FlangeRow { dn_key: 400, od: 597.0, bolt_circle: 539.8, bolt_holes: 16, bolt_diameter: 28.6, hub_length: 200.0, thickness: 44.5 },
    FlangeRow { dn_key: 450, od: 635.0, bolt_circle: 577.9, bolt_holes: 16, bolt_diameter: 31.8, hub_length: 216.0, thickness: 47.8 },
    FlangeRow { dn_key: 500, od: 699.0, bolt_circle: 635.0, bolt_holes: 20, bolt_diameter: 31.8, hub_length: 232.0, thickness: 50.8 },
    FlangeRow { dn_key: 600, od: 813.0, bolt_circle: 749.3, bolt_holes: 20, bolt_diameter: 35.1, hub_length: 252.0, thickness: 57.2 },
];

/// ASME B16.5 Class 300 weld neck flange dimensions (mm)
pub fn b16_5_flange_cl300(dn: f64) -> Option<B16_5Flange> {
    let dn_key = dn.round() as i32;
    FLANGE_CL300_TABLE.iter()
        .find(|row| row.dn_key == dn_key)
        .map(|row| B16_5Flange {
            dn: row.dn_key as f64,
            od: row.od,
            bolt_circle: row.bolt_circle,
            bolt_holes: row.bolt_holes,
            bolt_diameter: row.bolt_diameter,
            hub_length: row.hub_length,
            thickness: row.thickness,
        })
}

const FLANGE_CL300_TABLE: &[FlangeRow] = &[
    FlangeRow { dn_key: 15,  od: 95.0,  bolt_circle: 66.7,  bolt_holes: 4,  bolt_diameter: 15.9, hub_length: 52.0,  thickness: 14.3 },
    FlangeRow { dn_key: 20,  od: 120.0, bolt_circle: 82.6,  bolt_holes: 4,  bolt_diameter: 19.1, hub_length: 57.0,  thickness: 15.9 },
    FlangeRow { dn_key: 25,  od: 125.0, bolt_circle: 88.9,  bolt_holes: 4,  bolt_diameter: 19.1, hub_length: 62.0,  thickness: 17.5 },
    FlangeRow { dn_key: 32,  od: 135.0, bolt_circle: 98.4,  bolt_holes: 4,  bolt_diameter: 19.1, hub_length: 65.0,  thickness: 19.1 },
    FlangeRow { dn_key: 40,  od: 155.0, bolt_circle: 114.3, bolt_holes: 4,  bolt_diameter: 22.4, hub_length: 68.0,  thickness: 20.6 },
    FlangeRow { dn_key: 50,  od: 165.0, bolt_circle: 127.0, bolt_holes: 8,  bolt_diameter: 19.1, hub_length: 74.0,  thickness: 22.4 },
    FlangeRow { dn_key: 65,  od: 190.0, bolt_circle: 149.2, bolt_holes: 8,  bolt_diameter: 22.4, hub_length: 84.0,  thickness: 25.4 },
    FlangeRow { dn_key: 80,  od: 210.0, bolt_circle: 168.3, bolt_holes: 8,  bolt_diameter: 22.4, hub_length: 91.0,  thickness: 28.6 },
    FlangeRow { dn_key: 100, od: 255.0, bolt_circle: 200.0, bolt_holes: 8,  bolt_diameter: 22.4, hub_length: 102.0, thickness: 31.8 },
    FlangeRow { dn_key: 150, od: 320.0, bolt_circle: 269.9, bolt_holes: 12, bolt_diameter: 22.4, hub_length: 121.0, thickness: 36.6 },
    FlangeRow { dn_key: 200, od: 380.0, bolt_circle: 330.2, bolt_holes: 12, bolt_diameter: 25.4, hub_length: 140.0, thickness: 41.3 },
    FlangeRow { dn_key: 250, od: 445.0, bolt_circle: 387.4, bolt_holes: 16, bolt_diameter: 28.6, hub_length: 159.0, thickness: 47.8 },
    FlangeRow { dn_key: 300, od: 520.0, bolt_circle: 450.9, bolt_holes: 16, bolt_diameter: 31.8, hub_length: 178.0, thickness: 50.8 },
    FlangeRow { dn_key: 350, od: 585.0, bolt_circle: 514.4, bolt_holes: 20, bolt_diameter: 31.8, hub_length: 184.0, thickness: 53.9 },
    FlangeRow { dn_key: 400, od: 650.0, bolt_circle: 571.5, bolt_holes: 20, bolt_diameter: 35.1, hub_length: 200.0, thickness: 57.2 },
    FlangeRow { dn_key: 450, od: 710.0, bolt_circle: 628.7, bolt_holes: 24, bolt_diameter: 35.1, hub_length: 216.0, thickness: 60.3 },
    FlangeRow { dn_key: 500, od: 775.0, bolt_circle: 685.8, bolt_holes: 24, bolt_diameter: 35.1, hub_length: 232.0, thickness: 63.5 },
    FlangeRow { dn_key: 600, od: 915.0, bolt_circle: 812.8, bolt_holes: 24, bolt_diameter: 41.3, hub_length: 252.0, thickness: 69.9 },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dn100_sch40() {
        let w = wall_thickness(100.0, "40").unwrap();
        assert!((w - 6.02).abs() < 0.01);
    }

    #[test]
    fn test_dn50_sch80() {
        let w = wall_thickness(50.0, "80").unwrap();
        assert!((w - 5.54).abs() < 0.01);
    }

    #[test]
    fn test_dn150_od() {
        let od = outer_diameter(150.0);
        assert!((od - 168.28).abs() < 0.01);
    }

    #[test]
    fn test_unknown_schedule() {
        assert!(wall_thickness(100.0, "XXS").is_none());
    }

    #[test]
    fn test_unknown_dn() {
        assert!(wall_thickness(999.0, "40").is_none());
    }

    #[test]
    fn test_case_insensitive() {
        let w = wall_thickness(100.0, "40").unwrap();
        let w2 = wall_thickness(100.0, "40").unwrap();
        assert!((w - w2).abs() < 0.01);
    }

    #[test]
    fn test_dn750_sch40() {
        let w = wall_thickness(750.0, "40").unwrap();
        assert!((w - 9.53).abs() < 0.01);
    }

    #[test]
    fn test_dn900_od() {
        let od = outer_diameter(900.0);
        assert!((od - 914.40).abs() < 0.01);
    }

    #[test]
    fn test_b16_9_elbow_c2f() {
        let c2f = b16_9_elbow_center_to_face(100.0);
        assert!((c2f - 152.0).abs() < 0.1);
    }

    #[test]
    fn test_b16_9_elbow_fallback() {
        let c2f = b16_9_elbow_center_to_face(999.0);
        assert!((c2f - 1498.5).abs() < 0.1);
    }

    #[test]
    fn test_b16_5_flange_cl150() {
        let f = b16_5_flange_cl150(100.0).unwrap();
        assert!((f.od - 229.0).abs() < 0.1);
        assert_eq!(f.bolt_holes, 8);
    }

    #[test]
    fn test_b16_5_flange_unknown() {
        assert!(b16_5_flange_cl150(999.0).is_none());
    }

    #[test]
    fn test_dn750_od() {
        let od = outer_diameter(750.0);
        assert!((od - 762.0).abs() < 0.01);
    }
}