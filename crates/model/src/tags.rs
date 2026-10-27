//! 标准 Tag 常量 — 替代 ObjectType 细分类爆炸
//!
//! 约定：ObjectType 只定义大类（FITTING、DUCT 等），
//! 细分靠 tags + ifc_class + properties 区分。
//!
//! 用法示例：
//! ```ignore
//! use omnim_model::tags;
//! obj.add_tags(&[tags::MEP::DUCT, tags::FITTING::RECTANGULAR_ELBOW]);
//! // 等价于用一个 DUCT_FITTING + tags 代替 DUCT_RECTANGULAR_ELBOW 枚举
//! ```

/// 管件子类型 — 配合 OBJECT_TYPE_FITTING 使用
pub mod fitting {
    pub const ELBOW: &str = "elbow";
    pub const TEE: &str = "tee";
    pub const REDUCER: &str = "reducer";
    pub const FLANGE: &str = "flange";
    pub const GASKET: &str = "gasket";
    pub const NOZZLE: &str = "nozzle";
    pub const COUPLING: &str = "coupling";

    // 弯头细分
    pub const ELBOW_90: &str = "elbow_90";
    pub const ELBOW_45: &str = "elbow_45";
    pub const ELBOW_LR: &str = "elbow_long_radius"; // 长半径
    pub const ELBOW_SR: &str = "elbow_short_radius"; // 短半径

    // 三通细分
    pub const TEE_EQUAL: &str = "tee_equal";
    pub const TEE_REDUCING: &str = "tee_reducing";

    // 大小头细分
    pub const REDUCER_CONCENTRIC: &str = "reducer_concentric";
    pub const REDUCER_ECCENTRIC: &str = "reducer_eccentric";
}

/// MEP 暖通/电气/给排水子类型
pub mod mep {
    // 风管相关
    pub const DUCT: &str = "duct";
    pub const RECTANGULAR_ELBOW: &str = "rectangular_elbow";
    pub const DUCT_TRANSITION: &str = "duct_transition";
    pub const DUCT_BRANCH: &str = "duct_branch";

    // 系统类型
    pub const SUPPLY_AIR: &str = "supply_air";
    pub const RETURN_AIR: &str = "return_air";
    pub const EXHAUST: &str = "exhaust";
    pub const FRESH_AIR: &str = "fresh_air";
    pub const CHILLED_WATER: &str = "chilled_water";
    pub const HOT_WATER: &str = "hot_water";
    pub const CONDENSER_WATER: &str = "condenser_water";
    pub const REFRIGERANT: &str = "refrigerant";
    pub const DRAINAGE: &str = "drainage";
    pub const WATER_SUPPLY: &str = "water_supply";

    // 桥架
    pub const CABLE_TRAY: &str = "cable_tray";
    pub const CABLE_LADDER: &str = "cable_ladder";

    // 配电
    pub const POWER_DISTRIBUTION: &str = "power_distribution";
    pub const LIGHTING: &str = "lighting";
    pub const EMERGENCY: &str = "emergency";
    pub const FIRE_ALARM: &str = "fire_alarm";

    // 喷淋/消防
    pub const SPRINKLER: &str = "sprinkler";
    pub const FIRE_HYDRANT: &str = "fire_hydrant";
    pub const PLUMBING_FIXTURE: &str = "plumbing_fixture";
}

/// 建筑子类型 — 配合建筑 ObjectType 使用
pub mod architectural {
    pub const CURTAIN_WALL: &str = "curtain_wall";
    pub const CEILING: &str = "ceiling";
    pub const RAMP: &str = "ramp";
    pub const FOUNDATION: &str = "foundation";
    pub const PILE: &str = "pile";
    pub const TRUSS: &str = "truss";
    pub const BRACE: &str = "brace";
    pub const REINFORCEMENT: &str = "reinforcement"; // 钢筋
}

/// 工房子类型 — 配合工厂 ObjectType 使用
pub mod industrial {
    // 结构
    pub const PLATFORM: &str = "platform";
    pub const LADDER: &str = "ladder";
    pub const HANDRAIL: &str = "handrail";
    pub const PIPE_RACK: &str = "pipe_rack";
    pub const PIPE_SUPPORT: &str = "pipe_support";

    // 设备
    pub const FILTER: &str = "filter";
    pub const CONTROL_VALVE: &str = "control_valve";
    pub const METER: &str = "meter";
    pub const SENSOR: &str = "sensor";
    pub const TRANSMITTER: &str = "transmitter";
    pub const GAUGE: &str = "gauge";

    // 罐体
    pub const VERTICAL_VESSEL: &str = "vertical_vessel";
    pub const HORIZONTAL_VESSEL: &str = "horizontal_vessel";
    pub const SPHERICAL_TANK: &str = "spherical_tank";
    pub const STORAGE_TANK: &str = "storage_tank";
    pub const PRESSURE_VESSEL: &str = "pressure_vessel";
}

/// 土木子类型 — 配合土木 ObjectType 使用
pub mod civil {
    // 道路子构件
    pub const LANE: &str = "lane";
    pub const SHOULDER: &str = "shoulder";
    pub const CURB: &str = "curb";
    pub const PAVEMENT_LAYER: &str = "pavement_layer";

    // 桥梁子构件
    pub const GIRDER: &str = "girder";
    pub const DECK: &str = "deck";
    pub const BEARING_PAD: &str = "bearing_pad";
    pub const EXPANSION_JOINT: &str = "expansion_joint";

    // 管井类型
    pub const INSPECTION_WELL: &str = "inspection_well";
    pub const CATCH_BASIN: &str = "catch_basin";
    pub const ELECTRICAL_WELL: &str = "electrical_well";

    // 场地/地形
    pub const TERRAIN: &str = "terrain";
    pub const CUT: &str = "cut";
    pub const FILL: &str = "fill";
    pub const EMBANKMENT: &str = "embankment";
}

/// 机械子类型 — 配合机械 ObjectType 使用
pub mod mechanical {
    pub const COUPLING: &str = "coupling";
    pub const KEY: &str = "key";
    pub const PULLEY: &str = "pulley";
    pub const SPROCKET: &str = "sprocket";
    pub const CAM: &str = "cam";
    pub const PISTON: &str = "piston";
    pub const CYLINDER: &str = "cylinder";
    pub const SEAL_RING: &str = "seal_ring";
    pub const O_RING: &str = "o_ring";
    pub const GASKET_MECH: &str = "gasket";
    pub const COMPRESSION_SPRING: &str = "compression_spring";
    pub const TENSION_SPRING: &str = "tension_spring";
    pub const TORSION_SPRING: &str = "torsion_spring";
}
